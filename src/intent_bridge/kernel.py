"""The verification kernel — orchestrates Stages 1, 2, 3.

This module owns the one public entry point: :func:`verify`. Its job
is to walk the three-stage protocol from ``docs/01-architecture.md``,
emit a structured audit trail, and assemble the final
:class:`~intent_bridge.types.Verdict`. All non-trivial work delegates
to the helper modules:

* Stage 1 (PROPOSE) - :mod:`intent_bridge.proposer`
* Stage 2 (VALIDATE) - inline in :func:`_validate`
* Stage 3 (DISPOSE) - :mod:`intent_bridge._symbolic`, then
  :mod:`intent_bridge._fuzz` as fallback

The orchestrator is the only place that constructs a ``Verdict``.
This concentrates the verdict-shape invariants (every status carries
the right backing artifact) in a single file so they are auditable.
"""

from __future__ import annotations

from typing import Any

from intent_bridge._fuzz import fuzz
from intent_bridge._symbolic import verify_symbolically
from intent_bridge.proposer import (
    LLMProposer,
    ProposerInvalidOutputError,
    ProposerUncertainError,
)
from intent_bridge.types import (
    Budget,
    CounterExample,
    Example,
    FormalContract,
    Implementation,
    Intent,
    IntentInconsistencyError,
    Step,
    Verdict,
)


# ---------------------------------------------------------------------------
# Public entry point
# ---------------------------------------------------------------------------


def verify(
    intent: Intent,
    impl: Implementation,
    *,
    proposer: LLMProposer | None = None,
    budget: Budget | None = None,
) -> Verdict:
    """Decide whether ``impl`` realizes ``intent``.

    See ``docs/01-architecture.md`` for the protocol semantics. The
    main invariant is: VERIFIED is only returned when the symbolic
    engine produced a positive proof. Anything else is COUNTER_EXAMPLE
    or INCONCLUSIVE.

    ``proposer`` defaults to :class:`~intent_bridge.proposer.OpenRouterProposer`
    constructed from environment variables. Tests inject a mock.

    ``budget`` defaults to a small, safe envelope; production callers
    should size it for their workload.
    """
    budget = budget or Budget()
    trace: list[Step] = []

    # --- Pre-flight: intent self-consistency --------------------------
    _check_intent_consistency(intent)

    # --- Stage 1: PROPOSE ---------------------------------------------
    if proposer is None:
        # We import here to avoid forcing every caller to have an API
        # key set just to import this module.
        from intent_bridge.proposer import OpenRouterProposer

        proposer = OpenRouterProposer()

    trace.append(
        Step(stage="propose", event="calling_proposer", detail={"impl": impl.function_name})
    )
    try:
        contract = proposer.propose(intent, impl)
    except ProposerUncertainError as e:
        trace.append(
            Step(stage="propose", event="proposer_uncertain", detail={"reason": str(e)})
        )
        return Verdict(
            status="inconclusive",
            inconclusive_reason="proposer_uncertain",
            trace=trace,
        )
    except ProposerInvalidOutputError as e:
        trace.append(
            Step(
                stage="propose",
                event="proposer_invalid_output",
                detail={"reason": str(e)},
            )
        )
        return Verdict(
            status="inconclusive",
            inconclusive_reason="proposer_invalid_output",
            trace=trace,
        )

    trace.append(
        Step(
            stage="propose",
            event="contract_received",
            detail={
                "arg_sorts": contract.arg_sorts,
                "result_sort": contract.result_sort,
            },
        )
    )

    # --- Stage 2: VALIDATE --------------------------------------------
    validation_failure = _validate(intent, contract, trace)
    if validation_failure is not None:
        return validation_failure

    # --- Stage 3: DISPOSE ---------------------------------------------
    trace.append(Step(stage="dispose", event="symbolic_engine_start"))
    sym = verify_symbolically(impl, contract, budget)

    if sym.status == "verified":
        trace.append(Step(stage="dispose", event="verified_by_z3"))
        return Verdict(
            status="verified",
            contract=contract,
            proof_artifact=sym.proof_artifact,
            trace=trace,
        )

    if sym.status == "counter_example":
        trace.append(
            Step(
                stage="dispose",
                event="counter_example_from_z3",
                detail={
                    "args": list(sym.cex_args or ()),
                    "observed": sym.cex_observed,
                    "expected": sym.cex_expected,
                },
            )
        )
        cex = CounterExample(
            args=sym.cex_args or (),
            observed=sym.cex_observed,
            expected=sym.cex_expected,
            discovered_by="z3",
        )
        return Verdict(
            status="counter_example",
            contract=contract,
            counter_example=cex,
            trace=trace,
        )

    # Symbolic engine was inconclusive. Two sub-cases.
    sym_reason = sym.inconclusive_reason or "unknown"
    trace.append(
        Step(
            stage="dispose",
            event="symbolic_inconclusive",
            detail={"reason": sym_reason},
        )
    )

    # If the function is out of the supported fragment, fuzzing is
    # often pointless (the post lambda may rely on z3-only semantics)
    # but it can still catch obvious bugs on numeric domains. We try
    # it best-effort, then fall through with the original reason.
    if sym_reason.startswith("out_of_fragment"):
        # Out-of-fragment functions get one fuzz attempt; if it still
        # finds nothing, we surface the structural reason rather than
        # the generic "budget" excuse.
        fz = fuzz(impl, contract, budget)
        if fz.status == "counter_example":
            trace.append(
                Step(
                    stage="dispose",
                    event="counter_example_from_fuzz",
                    detail={
                        "args": list(fz.cex_args or ()),
                        "observed": fz.cex_observed,
                    },
                )
            )
            cex = CounterExample(
                args=fz.cex_args or (),
                observed=fz.cex_observed,
                expected=None,
                discovered_by="hypothesis",
            )
            return Verdict(
                status="counter_example",
                contract=contract,
                counter_example=cex,
                trace=trace,
            )
        return Verdict(
            status="inconclusive",
            inconclusive_reason="out_of_fragment",
            contract=contract,
            trace=trace,
        )

    # Symbolic timed out / returned UNKNOWN. Try fuzzing as a fallback.
    trace.append(Step(stage="dispose", event="fuzz_fallback_start"))
    fz = fuzz(impl, contract, budget)
    if fz.status == "counter_example":
        trace.append(
            Step(
                stage="dispose",
                event="counter_example_from_fuzz",
                detail={
                    "args": list(fz.cex_args or ()),
                    "observed": fz.cex_observed,
                },
            )
        )
        cex = CounterExample(
            args=fz.cex_args or (),
            observed=fz.cex_observed,
            expected=None,
            discovered_by="hypothesis",
        )
        return Verdict(
            status="counter_example",
            contract=contract,
            counter_example=cex,
            trace=trace,
        )

    # Neither engine could decide. Honest inconclusive: NEVER promote
    # to VERIFIED, no matter how many fuzz iterations passed.
    return Verdict(
        status="inconclusive",
        inconclusive_reason="budget_exhausted",
        contract=contract,
        trace=trace,
    )


# ---------------------------------------------------------------------------
# Stage 0: pre-flight consistency check
# ---------------------------------------------------------------------------


def _check_intent_consistency(intent: Intent) -> None:
    """Reject intent whose own pieces disagree.

    The check fires when both an explicit ``post_python`` and at least
    one example are present, and the example violates the
    postcondition. In that case no contract can possibly satisfy the
    intent and we crash loudly rather than emit a misleading verdict.
    """
    if intent.post_python is None or not intent.examples:
        return
    try:
        post_fn = eval(intent.post_python, {"__builtins__": {}})  # noqa: S307
    except SyntaxError as e:
        raise IntentInconsistencyError(
            f"explicit post_python does not parse: {e}"
        ) from e

    for ex in intent.examples:
        try:
            ok = bool(post_fn(ex.args, ex.expected))
        except Exception as e:  # noqa: BLE001
            raise IntentInconsistencyError(
                f"explicit post_python raised on example {ex}: {e}"
            ) from e
        if not ok:
            raise IntentInconsistencyError(
                f"example {ex.args!r} -> {ex.expected!r} contradicts "
                f"the explicit post_python"
            )


# ---------------------------------------------------------------------------
# Stage 2: validate the LLM's proposal
# ---------------------------------------------------------------------------


def _validate(
    intent: Intent, contract: FormalContract, trace: list[Step]
) -> Verdict | None:
    """Run schema + sanity checks on the proposed contract.

    Returns a verdict (always INCONCLUSIVE) if validation fails;
    returns None if the contract is acceptable to pass to Stage 3.
    """
    # Schema validation already happened in pydantic when the proposer
    # constructed the FormalContract; here we check the *semantic*
    # invariants that pydantic cannot enforce.

    # Lengths must agree across the three sort-shaped fields.
    if len(contract.arg_sorts) == 0:
        trace.append(
            Step(stage="validate", event="zero_args", detail={})
        )
        return Verdict(
            status="inconclusive",
            inconclusive_reason="proposer_invalid_output",
            contract=contract,
            trace=trace,
        )

    # The proposed contract must accept every user-supplied example.
    # This is the structural defence against the LLM hallucinating a
    # plausible-looking but wrong contract. If the LLM is right, the
    # check is free; if the LLM is wrong, we abort here rather than
    # let z3 silently verify against a bad spec.
    try:
        post_fn = eval(contract.python_postcondition, {"__builtins__": {}})  # noqa: S307
    except SyntaxError as e:
        trace.append(
            Step(
                stage="validate",
                event="postcondition_unparseable",
                detail={"error": str(e)},
            )
        )
        return Verdict(
            status="inconclusive",
            inconclusive_reason="proposer_invalid_output",
            contract=contract,
            trace=trace,
        )

    for ex in intent.examples:
        try:
            ok = bool(post_fn(ex.args, ex.expected))
        except Exception as e:  # noqa: BLE001
            trace.append(
                Step(
                    stage="validate",
                    event="postcondition_raised_on_example",
                    detail={"example": _example_to_jsonable(ex), "error": str(e)},
                )
            )
            return Verdict(
                status="inconclusive",
                inconclusive_reason="proposer_invalid_output",
                contract=contract,
                trace=trace,
            )
        if not ok:
            trace.append(
                Step(
                    stage="validate",
                    event="proposer_rejects_user_example",
                    detail={"example": _example_to_jsonable(ex)},
                )
            )
            return Verdict(
                status="inconclusive",
                inconclusive_reason="proposer_inconsistent",
                contract=contract,
                trace=trace,
            )

    trace.append(Step(stage="validate", event="contract_validated"))
    return None


def _example_to_jsonable(ex: Example) -> dict[str, Any]:
    return {"args": list(ex.args), "expected": ex.expected}
