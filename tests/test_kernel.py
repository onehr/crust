"""End-to-end kernel tests.

These are the load-bearing tests for Bridge V1. The architecture
document lists ten properties the kernel must obey; each one is at
least one assertion below. The tests are written *before* the kernel
implementation — they are the executable spec of what the kernel does,
not a check that what we wrote happens to behave a particular way.

If a test in this file fails, the kernel is broken; do not "loosen"
the test to make it pass.
"""

from __future__ import annotations

import pytest

from intent_bridge import (
    Budget,
    Example,
    Implementation,
    Intent,
    verify,
)
from intent_bridge.types import (
    FormalContract,
    IntentInconsistencyError,
    Verdict,
)


# ---------------------------------------------------------------------------
# Property 1: correct impl + decidable contract → VERIFIED
# ---------------------------------------------------------------------------


def test_correct_abs_yields_verified(
    abs_intent: Intent,
    abs_correct_impl: Implementation,
    abs_contract: FormalContract,
    mock_proposer,
) -> None:
    proposer = mock_proposer(lambda _i, _f: abs_contract)
    verdict = verify(abs_intent, abs_correct_impl, proposer=proposer)

    assert verdict.status == "verified"
    assert verdict.contract == abs_contract
    assert verdict.proof_artifact is not None  # "unsat: forall x ..." trace
    assert verdict.counter_example is None


# ---------------------------------------------------------------------------
# Property 2: buggy impl + decidable contract → COUNTER_EXAMPLE
# ---------------------------------------------------------------------------


def test_buggy_abs_yields_counter_example(
    abs_intent: Intent,
    abs_buggy_impl: Implementation,
    abs_contract: FormalContract,
    mock_proposer,
) -> None:
    proposer = mock_proposer(lambda _i, _f: abs_contract)
    verdict = verify(abs_intent, abs_buggy_impl, proposer=proposer)

    assert verdict.status == "counter_example"
    assert verdict.counter_example is not None
    # The bug is in the negative branch; any negative input is a witness.
    cex = verdict.counter_example
    assert len(cex.args) == 1
    assert isinstance(cex.args[0], int)
    assert cex.args[0] < 0
    # Observed output equals the input (the bug), not its absolute value.
    assert cex.observed == cex.args[0]
    # The counter-example must come from the symbolic engine for this case.
    assert cex.discovered_by == "z3"


# ---------------------------------------------------------------------------
# Property 3: vague intent (no examples, vague docstring) → INCONCLUSIVE
# ---------------------------------------------------------------------------


def test_vague_intent_yields_inconclusive(
    abs_correct_impl: Implementation, mock_proposer
) -> None:
    """If the proposer signals it cannot translate the intent, the kernel
    must surface that as INCONCLUSIVE — never invent a contract.
    """

    def refusing_proposer(_i: Intent, _f: Implementation) -> FormalContract:
        from intent_bridge.proposer import ProposerUncertainError

        raise ProposerUncertainError("docstring too vague, no examples to anchor")

    proposer = mock_proposer(refusing_proposer)
    intent = Intent(docstring="Do something with x.")
    verdict = verify(intent, abs_correct_impl, proposer=proposer)

    assert verdict.status == "inconclusive"
    assert verdict.inconclusive_reason == "proposer_uncertain"


# ---------------------------------------------------------------------------
# Property 4: examples that contradict the docstring → IntentInconsistencyError
#
# This must be detected before the proposer is even called. The kernel
# cannot verify intent that already disagrees with itself.
# ---------------------------------------------------------------------------


def test_examples_contradicting_explicit_postcondition_raise(
    abs_correct_impl: Implementation, mock_proposer
) -> None:
    """Two of the supplied examples can never simultaneously match the
    explicit ``post_python`` the user also supplied. The kernel must
    refuse to run.
    """
    intent = Intent(
        docstring="Return the absolute value of x.",
        examples=[
            Example(args=(-3,), expected=99),  # impossible under any abs interpretation
        ],
        post_python="lambda args, result: result == (-args[0] if args[0] < 0 else args[0])",
    )

    with pytest.raises(IntentInconsistencyError):
        verify(intent, abs_correct_impl, proposer=mock_proposer(lambda _i, _f: None))  # type: ignore[arg-type]


# ---------------------------------------------------------------------------
# Property 5: LLM proposes a contract that disagrees with examples → INCONCLUSIVE
#
# This is the structural defence against LLM hallucination. The proposer
# might confidently emit a contract; if that contract fails to accept the
# user's own examples, we abort.
# ---------------------------------------------------------------------------


def test_proposer_contract_inconsistent_with_examples(
    abs_intent: Intent,
    abs_correct_impl: Implementation,
    mock_proposer,
) -> None:
    # This contract claims abs(x) == x always, which fails on the
    # negative example the user provided.
    wrong_contract = FormalContract(
        z3_precondition="(declare-const x Int) (assert true)",
        z3_postcondition=(
            "(declare-const x Int) (declare-const result Int) (assert (= result x))"
        ),
        python_postcondition="lambda args, result: result == args[0]",
        arg_sorts=["Int"],
        result_sort="Int",
        reasoning="(intentionally wrong for the test)",
    )

    proposer = mock_proposer(lambda _i, _f: wrong_contract)
    verdict = verify(abs_intent, abs_correct_impl, proposer=proposer)

    assert verdict.status == "inconclusive"
    assert verdict.inconclusive_reason == "proposer_inconsistent"


# ---------------------------------------------------------------------------
# Property 6: function outside the V1 fragment → INCONCLUSIVE / out_of_fragment
# ---------------------------------------------------------------------------


def test_function_outside_fragment_is_inconclusive(mock_proposer) -> None:
    """An int -> int function that nevertheless uses control flow outside
    the V1 fragment (here: a ``for`` loop and a local assignment) must be
    flagged as out_of_fragment, not silently mis-verified.

    The contract here is well-typed (int args, int result, postcondition
    that accepts the supplied examples) so validation passes; the failure
    must come from the symbolic engine's translation step.
    """
    impl = Implementation(
        source=(
            "def f(x):\n"
            "    total = 0\n"
            "    for i in range(x):\n"
            "        total = total + 1\n"
            "    return total\n"
        ),
        function_name="f",
    )
    intent = Intent(
        docstring="Return x, computed by counting up.",
        examples=[Example(args=(0,), expected=0), Example(args=(3,), expected=3)],
    )
    contract = FormalContract(
        z3_precondition="(>= x 0)",
        z3_postcondition="(= result x)",
        python_postcondition="lambda args, result: result == args[0]",
        python_precondition="lambda args: args[0] >= 0",
        arg_sorts=["Int"],
        result_sort="Int",
        reasoning="iterative count up to x",
    )

    proposer = mock_proposer(lambda _i, _f: contract)
    verdict = verify(intent, impl, proposer=proposer)

    assert verdict.status == "inconclusive"
    assert verdict.inconclusive_reason == "out_of_fragment"


# ---------------------------------------------------------------------------
# Property 7: under a tight budget, verdict invariants still hold.
#
# Note: an earlier draft of this test asserted that a 1 ms z3 budget
# would force INCONCLUSIVE. That is empirically false — abs(x) is
# decided in microseconds, so 1 ms is plenty. The honest invariant is
# structural: WHATEVER the budget, every verdict must carry the
# accompanying artifact its status requires. False VERIFIED-without-
# proof is impossible at the Verdict-constructor level (test 10);
# this test confirms the kernel does not bypass that on tight budgets.
# ---------------------------------------------------------------------------


def test_tight_budget_preserves_verdict_invariants(
    abs_intent: Intent,
    abs_correct_impl: Implementation,
    abs_contract: FormalContract,
    mock_proposer,
) -> None:
    budget = Budget(z3_timeout_ms=1, fuzz_max_examples=1)
    proposer = mock_proposer(lambda _i, _f: abs_contract)
    verdict = verify(abs_intent, abs_correct_impl, proposer=proposer, budget=budget)

    match verdict.status:
        case "verified":
            assert verdict.proof_artifact is not None
            assert verdict.contract is not None
        case "counter_example":
            assert verdict.counter_example is not None
        case "inconclusive":
            assert verdict.inconclusive_reason in (
                "budget_exhausted",
                "symbolic_undecided",
                "out_of_fragment",
                "proposer_inconsistent",
                "proposer_uncertain",
                "proposer_invalid_output",
            )


# ---------------------------------------------------------------------------
# Property 8: determinism — same fixture twice → byte-identical verdicts
# ---------------------------------------------------------------------------


def test_determinism_same_fixture(
    abs_intent: Intent,
    abs_correct_impl: Implementation,
    abs_contract: FormalContract,
    mock_proposer,
) -> None:
    p1 = mock_proposer(lambda _i, _f: abs_contract)
    p2 = mock_proposer(lambda _i, _f: abs_contract)
    v1 = verify(abs_intent, abs_correct_impl, proposer=p1)
    v2 = verify(abs_intent, abs_correct_impl, proposer=p2)

    # We do not compare ``trace`` byte-for-byte because timing fields
    # may differ; we compare the structural verdict.
    assert v1.status == v2.status
    assert v1.contract == v2.contract
    assert v1.counter_example == v2.counter_example
    assert v1.inconclusive_reason == v2.inconclusive_reason


# ---------------------------------------------------------------------------
# Property 9: audit trail is always present and contains the contract
# ---------------------------------------------------------------------------


def test_audit_trail_includes_contract_on_all_verdicts(
    abs_intent: Intent,
    abs_correct_impl: Implementation,
    abs_buggy_impl: Implementation,
    abs_contract: FormalContract,
    mock_proposer,
) -> None:
    for impl in (abs_correct_impl, abs_buggy_impl):
        proposer = mock_proposer(lambda _i, _f: abs_contract)
        verdict: Verdict = verify(abs_intent, abs_correct_impl, proposer=proposer)
        # Either it's verified (correct) or counter_example (buggy); both
        # must include the contract that was checked.
        assert verdict.contract is not None
        assert verdict.trace, "trace must never be empty"
        stages_seen = {step.stage for step in verdict.trace}
        assert {"propose", "validate", "dispose"} <= stages_seen


# ---------------------------------------------------------------------------
# Property 10: structural — VERIFIED requires a proof artifact
#
# This is enforced by the Verdict type itself; if any code path
# constructs a Verdict(status="verified", proof_artifact=None) the
# constructor raises. The test pins this behavior so the invariant
# stays load-bearing even if Verdict is refactored.
# ---------------------------------------------------------------------------


def test_verified_status_requires_proof_artifact(
    abs_contract: FormalContract,
) -> None:
    with pytest.raises(ValueError, match="verified verdict requires a proof artifact"):
        Verdict(status="verified", contract=abs_contract, proof_artifact=None)
