"""Stage 3 (primary path): z3-based symbolic verification.

The pipeline here is:

  1. Build z3 constants for each function argument and a ``result``
     variable, using the sorts the LLM proposed.
  2. Translate the implementation's body to a z3 expression in terms
     of those constants (``intent_bridge._translator``).
  3. Parse the proposed precondition and postcondition as SMT-LIB
     boolean expressions.
  4. Ask z3: is there an assignment to the args satisfying the
     precondition, with ``result`` equal to the body's value, that
     violates the postcondition?
       - SAT   -> COUNTER_EXAMPLE (model.evaluate(...) gives concrete values)
       - UNSAT -> VERIFIED (no such assignment exists, for all inputs)
       - UNKNOWN / timeout -> INCONCLUSIVE (kick to the fuzz fallback)

The verifier returns a small dataclass; the kernel turns it into a
``Verdict``. We deliberately keep the dataclass internal so we can
evolve the symbolic interface without touching the public API.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Literal

import z3

from intent_bridge._translator import (
    OutOfFragmentError,
    extract_arg_names,
    extract_function,
    make_const,
    translate,
)
from intent_bridge.types import Budget, FormalContract, Implementation


# ---------------------------------------------------------------------------
# Result type
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class SymbolicResult:
    status: Literal["verified", "counter_example", "inconclusive"]
    # COUNTER_EXAMPLE fields:
    cex_args: tuple[Any, ...] | None = None
    cex_observed: Any | None = None
    cex_expected: Any | None = None
    # VERIFIED field: a string that documents what was proved.
    proof_artifact: str | None = None
    # INCONCLUSIVE field: machine-readable reason.
    inconclusive_reason: str | None = None


# ---------------------------------------------------------------------------
# SMT parsing
# ---------------------------------------------------------------------------


def _parse_smt_expr(
    expr_str: str,
    arg_names: list[str],
    arg_sorts: list[str],
    result_sort: str,
) -> z3.BoolRef:
    """Parse an SMT-LIB v2 boolean expression with declared free vars.

    The proposer emits the expression alone; this helper wraps it with
    the constant declarations the kernel knows about and lets z3 parse
    the result. Returns the conjunction of all parsed assertions (we
    only emit one, but defending against multi-assert input is cheap).
    """
    decls = "".join(f"(declare-const {n} {s}) " for n, s in zip(arg_names, arg_sorts))
    decls += f"(declare-const result {result_sort}) "
    script = decls + f"(assert {expr_str})"
    asserts = z3.parse_smt2_string(script)
    if len(asserts) == 0:
        return z3.BoolVal(True)
    if len(asserts) == 1:
        return asserts[0]
    return z3.And(*asserts)


# ---------------------------------------------------------------------------
# z3 value -> Python value extraction
#
# Models contain z3 AST nodes; we evaluate them to concrete Python
# values so that the audit trail and the counter-example can be
# manipulated without dragging z3 types around.
# ---------------------------------------------------------------------------


def _to_python(z3_val: z3.ExprRef) -> Any:
    if z3.is_int_value(z3_val):
        return z3_val.as_long()
    if z3.is_rational_value(z3_val):
        return float(z3_val.numerator_as_long()) / float(z3_val.denominator_as_long())
    if z3.is_bool(z3_val):
        return z3.is_true(z3_val)
    # Fallback: stringify; caller decides what to do.
    return str(z3_val)


# ---------------------------------------------------------------------------
# Main entry point
# ---------------------------------------------------------------------------


def verify_symbolically(
    impl: Implementation,
    contract: FormalContract,
    budget: Budget,
) -> SymbolicResult:
    """Try to decide ``impl`` against ``contract`` using z3.

    Never returns ``VERIFIED`` unless z3 produced UNSAT for the
    negated postcondition AND the function was successfully translated
    in full. Any other path produces ``INCONCLUSIVE`` with a reason.
    """

    # ------ 1. Resolve function and check arg count -------------------
    try:
        fn = extract_function(impl.source, impl.function_name)
        arg_names = extract_arg_names(fn)
    except OutOfFragmentError as e:
        return SymbolicResult(
            status="inconclusive", inconclusive_reason=f"out_of_fragment: {e}"
        )

    if len(arg_names) != len(contract.arg_sorts):
        return SymbolicResult(
            status="inconclusive",
            inconclusive_reason=(
                f"arg_count_mismatch: function declares {len(arg_names)} args, "
                f"contract declares {len(contract.arg_sorts)}"
            ),
        )

    # ------ 2. Build z3 constants -------------------------------------
    try:
        args_z3 = {
            name: make_const(name, sort)
            for name, sort in zip(arg_names, contract.arg_sorts)
        }
        result_z3 = make_const("result", contract.result_sort)
    except OutOfFragmentError as e:
        return SymbolicResult(
            status="inconclusive", inconclusive_reason=f"unsupported_sort: {e}"
        )

    # ------ 3. Translate function body --------------------------------
    try:
        body_expr = translate(impl.source, impl.function_name, args_z3)
    except OutOfFragmentError as e:
        return SymbolicResult(
            status="inconclusive", inconclusive_reason="out_of_fragment", proof_artifact=str(e)
        )

    # ------ 4. Parse contract pre / post ------------------------------
    try:
        pre = _parse_smt_expr(
            contract.z3_precondition, arg_names, contract.arg_sorts, contract.result_sort
        )
        post = _parse_smt_expr(
            contract.z3_postcondition, arg_names, contract.arg_sorts, contract.result_sort
        )
    except z3.Z3Exception as e:
        return SymbolicResult(
            status="inconclusive", inconclusive_reason=f"contract_unparseable: {e}"
        )

    # ------ 5. Ask z3 the verification question -----------------------
    #
    # We want: forall args, pre(args) implies post(args, body(args))
    # Equivalent: SAT(pre AND result == body AND NOT post) ?
    #   SAT   => witness violates post => COUNTER_EXAMPLE
    #   UNSAT => no witness exists      => VERIFIED
    #   UNKNOWN => z3 cannot decide     => INCONCLUSIVE
    solver = z3.Solver()
    solver.set("timeout", budget.z3_timeout_ms)
    solver.add(pre)
    solver.add(result_z3 == body_expr)
    solver.add(z3.Not(post))
    check = solver.check()

    if check == z3.unsat:
        return SymbolicResult(
            status="verified",
            proof_artifact=(
                f"z3 UNSAT for (pre AND result==body AND NOT post); "
                f"checked under timeout {budget.z3_timeout_ms} ms; "
                f"args={arg_names}; sorts={contract.arg_sorts}->{contract.result_sort}"
            ),
        )

    if check == z3.sat:
        model = solver.model()
        cex_args = tuple(_to_python(model.evaluate(args_z3[n], model_completion=True)) for n in arg_names)
        cex_observed = _to_python(model.evaluate(result_z3, model_completion=True))
        # Recover the "expected" output for the audit trail: the value
        # that WOULD satisfy the postcondition under these specific args.
        # If the postcondition is functional, this is unique; if not,
        # z3 picks one. If no value satisfies it (degenerate contract),
        # we report None.
        expected = _expected_under_post(
            args_z3, result_z3, post, cex_args, arg_names, budget
        )
        return SymbolicResult(
            status="counter_example",
            cex_args=cex_args,
            cex_observed=cex_observed,
            cex_expected=expected,
        )

    # z3 returned `unknown` — typically because of timeout.
    return SymbolicResult(
        status="inconclusive",
        inconclusive_reason="symbolic_undecided",
    )


# ---------------------------------------------------------------------------
# Expected-output recovery
# ---------------------------------------------------------------------------


def _expected_under_post(
    args_z3: dict[str, z3.ExprRef],
    result_z3: z3.ExprRef,
    post: z3.BoolRef,
    cex_args: tuple[Any, ...],
    arg_names: list[str],
    budget: Budget,
) -> Any | None:
    """Find a value of ``result`` that satisfies ``post`` for the given args.

    This is a courtesy for the audit trail; it lets the human reader
    see "the implementation returned X, but the contract says it
    should have returned Y". When the postcondition is non-functional
    (admits multiple results for these args), we surface whatever z3
    picks first; when it is unsatisfiable, we return None.
    """
    solver = z3.Solver()
    solver.set("timeout", budget.z3_timeout_ms)
    for name, val in zip(arg_names, cex_args):
        solver.add(args_z3[name] == val)
    solver.add(post)
    if solver.check() == z3.sat:
        return _to_python(solver.model().evaluate(result_z3, model_completion=True))
    return None
