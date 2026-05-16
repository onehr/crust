"""Stage 3 (fallback path): hypothesis-driven counter-example search.

When the symbolic engine returns INCONCLUSIVE (e.g. timeout, function
uses a feature outside the V1 fragment but is still pure), we try
property-based fuzzing as a second chance to find a counter-example.
This path can ONLY produce COUNTER_EXAMPLE or INCONCLUSIVE — it never
produces VERIFIED. Absence of falsifications in N random tries is
not a proof.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Literal

from hypothesis import HealthCheck, given, settings, strategies as st

from intent_bridge._translator import extract_arg_names, extract_function
from intent_bridge.types import Budget, FormalContract, Implementation


@dataclass(frozen=True)
class FuzzResult:
    status: Literal["counter_example", "inconclusive"]
    cex_args: tuple[Any, ...] | None = None
    cex_observed: Any | None = None


# Integer strategy is deliberately bounded. The fuzz path runs Python
# *concretely*, so an unbounded strategy can generate inputs whose
# evaluation cost dwarfs the entire fuzz budget (e.g. `range(10**9)` in
# a loop). The bound is large enough to surface most semantic bugs and
# small enough that hypothesis can exhaust ``fuzz_max_examples`` in
# reasonable time. Functions with bugs only on truly large inputs are
# out of scope for V1's fuzz fallback; symbolic reasoning is where
# those belong.
_INT_FUZZ_BOUND = 10_000

_SORT_TO_STRATEGY = {
    "Int": st.integers(min_value=-_INT_FUZZ_BOUND, max_value=_INT_FUZZ_BOUND),
    "Real": st.floats(allow_nan=False, allow_infinity=False, width=64),
    "Bool": st.booleans(),
}


def _compile_impl(impl: Implementation):
    """Compile the implementation source and return a callable.

    We intentionally do NOT advertise sandbox semantics on this path.
    A proper sandbox needs subprocess isolation (or `multiprocessing`
    with seccomp); restricting ``__builtins__`` in-process gives a
    false sense of safety. The kernel is a research tool; treat
    untrusted source the same way you would treat any other code you
    are about to ``exec`` — by running it under OS isolation if you
    cannot trust it.
    """
    namespace: dict[str, Any] = {}
    exec(compile(impl.source, f"<impl:{impl.function_name}>", "exec"), namespace)
    fn = namespace.get(impl.function_name)
    if fn is None:
        raise RuntimeError(
            f"function {impl.function_name!r} not defined by source"
        )
    return fn


def _compile_postcondition(python_post: str):
    """Compile ``contract.python_postcondition`` to a callable.

    The lambda must have shape ``lambda args, result: bool``. We do
    not restrict its environment as aggressively as the implementation,
    because the lambda is the LLM's translation of the user's intent;
    if it is malicious, the kernel has bigger problems than sandboxing.
    """
    return eval(python_post, {"__builtins__": {}})  # noqa: S307


def fuzz(
    impl: Implementation,
    contract: FormalContract,
    budget: Budget,
) -> FuzzResult:
    """Look for a counter-example via property-based testing.

    The implementation is invoked on hypothesis-generated inputs in
    the input domain implied by ``contract.arg_sorts``. Each output is
    checked against ``contract.python_postcondition``. The first
    falsifying input is returned; otherwise INCONCLUSIVE.
    """

    fn = _compile_impl(impl)
    post_fn = _compile_postcondition(contract.python_postcondition)
    pre_fn = _compile_postcondition(contract.python_precondition)

    fn_args_ast = extract_function(impl.source, impl.function_name)
    arg_names = extract_arg_names(fn_args_ast)
    if len(arg_names) != len(contract.arg_sorts):
        return FuzzResult(status="inconclusive")

    try:
        strategies = [_SORT_TO_STRATEGY[s] for s in contract.arg_sorts]
    except KeyError:
        return FuzzResult(status="inconclusive")

    found: list[tuple[Any, ...]] = []
    falsifying_output: list[Any] = []

    @given(st.tuples(*strategies))
    @settings(
        max_examples=budget.fuzz_max_examples,
        deadline=None,
        suppress_health_check=[HealthCheck.function_scoped_fixture],
        database=None,  # avoid hypothesis writing to .hypothesis/ during runs
        derandomize=True,  # makes determinism easier to preserve
    )
    def property_check(args: tuple[Any, ...]) -> None:
        # Inputs outside the contract's precondition are not a fair
        # test. Skip them silently rather than report them as
        # counter-examples, otherwise a function that diverges from
        # the contract only on its excluded domain would be wrongly
        # falsified.
        try:
            if not pre_fn(args):
                return
        except Exception:
            # A precondition that raises is itself broken — surface
            # this as INCONCLUSIVE rather than COUNTER_EXAMPLE.
            return

        try:
            observed = fn(*args)
        except Exception:
            # An implementation that raises on a precondition-passing
            # input violates the contract just as much as one returning
            # the wrong value. Surface the input as the counter-example.
            if not found:
                found.append(args)
                falsifying_output.append(None)
            assert False, "implementation raised on valid input"

        if not post_fn(args, observed):
            if not found:
                found.append(args)
                falsifying_output.append(observed)
            assert False, f"postcondition failed on {args} -> {observed}"

    try:
        property_check()
    except AssertionError:
        # hypothesis raises through the assertion; the falsifying args
        # are in ``found`` thanks to the side channel above. Hypothesis
        # also shrinks; we may want to use ``find`` instead for cleaner
        # output, but ``given`` + assertion is the standard recipe.
        pass

    if found:
        return FuzzResult(
            status="counter_example",
            cex_args=tuple(found[0]),
            cex_observed=falsifying_output[0],
        )
    return FuzzResult(status="inconclusive")
