"""Bridge — public data types.

All types are pydantic v2 models. They are the *only* shapes that cross
the kernel's interfaces. Free-form dicts, **kwargs, and Any are not
allowed at module boundaries; the schema is the contract.

Read `docs/01-architecture.md` for the design rationale.
"""

from __future__ import annotations

from typing import Any, Literal

from pydantic import BaseModel, ConfigDict, Field, field_validator


# ---------------------------------------------------------------------------
# Inputs
# ---------------------------------------------------------------------------


class Example(BaseModel):
    """A single input-output pair the user offers as evidence of intent.

    The example is treated as ground truth: any LLM-proposed contract
    that fails to accept all examples is rejected as inconsistent
    (Stage 2 in the protocol). This is what stops the proposer from
    silently inventing a contract that disagrees with what the user
    explicitly said.
    """

    model_config = ConfigDict(frozen=True)

    args: tuple[Any, ...]
    expected: Any


class Intent(BaseModel):
    """What the user wants the implementation to do.

    The docstring is the natural-language statement; the examples pin
    it down on specific inputs. Optional ``pre_python`` / ``post_python``
    let a user supply an exact formal contract — when present, the LLM
    is asked only to translate them to z3, never to invent them.
    """

    model_config = ConfigDict(frozen=True)

    docstring: str = Field(..., min_length=1)
    examples: list[Example] = Field(default_factory=list)
    pre_python: str | None = None
    post_python: str | None = None

    @field_validator("docstring")
    @classmethod
    def _strip_docstring(cls, v: str) -> str:
        v = v.strip()
        if not v:
            raise ValueError("docstring may not be empty after stripping")
        return v


class Implementation(BaseModel):
    """A Python function under test, supplied as source text.

    The kernel re-compiles the source in a restricted namespace; raw
    callables are not accepted because they would defeat the audit-trail
    invariant (we need the *bytes* of the function that was verified,
    not a closure over arbitrary state).
    """

    model_config = ConfigDict(frozen=True)

    source: str = Field(..., min_length=1)
    function_name: str = Field(..., min_length=1)


class Budget(BaseModel):
    """Bounds on every external resource the kernel may spend.

    Exceeding any of these produces an INCONCLUSIVE verdict with reason
    ``budget_exhausted``. There is no "best-effort, keep trying" mode;
    the budget is non-negotiable.
    """

    model_config = ConfigDict(frozen=True)

    llm_max_tokens: int = Field(default=2048, gt=0)
    llm_max_retries: int = Field(default=2, ge=0)
    z3_timeout_ms: int = Field(default=5000, gt=0)
    fuzz_max_examples: int = Field(default=200, gt=0)


# ---------------------------------------------------------------------------
# Internal intermediates
# ---------------------------------------------------------------------------


class FormalContract(BaseModel):
    """The LLM's proposed contract, after schema validation.

    Two surface forms travel together:
    * ``z3_*`` strings in SMT-LIB v2 syntax for the symbolic engine;
    * ``python_postcondition`` as a Python lambda source for the
      property-test fallback and for the example-consistency check.

    The two must agree semantically. Disagreement is a proposer bug
    that the validator will catch when the contract fails to accept
    the user's own examples.
    """

    model_config = ConfigDict(frozen=True)

    z3_precondition: str
    z3_postcondition: str
    python_postcondition: str
    # The Python-callable form of the precondition. Used by the fuzz
    # fallback to filter inputs that fall outside the contract's input
    # domain; ``true`` if the contract has no nontrivial precondition.
    # Keep this in sync with ``z3_precondition`` — disagreement between
    # the two surfaces in Stage 2 validation.
    python_precondition: str = "lambda args: True"
    arg_sorts: list[str]
    result_sort: str
    reasoning: str


class CounterExample(BaseModel):
    """A concrete falsifying input.

    ``discovered_by`` records which engine produced this counter-example
    so that downstream tooling (and humans) can judge how to react. A
    counter-example from ``user_example`` means the user's own examples
    contradict the proposed contract — a different failure shape than
    a counter-example mined by the symbolic engine.
    """

    model_config = ConfigDict(frozen=True)

    args: tuple[Any, ...]
    observed: Any
    expected: Any | None
    discovered_by: Literal["z3", "hypothesis", "user_example"]


class Step(BaseModel):
    """A single line in the audit trail.

    The trail must be sufficient for a third party to reconstruct *why*
    the kernel reached the verdict it did. We pin to small structured
    events rather than free text.
    """

    model_config = ConfigDict(frozen=True)

    stage: Literal["propose", "validate", "dispose"]
    event: str
    detail: dict[str, Any] = Field(default_factory=dict)


# ---------------------------------------------------------------------------
# Output
# ---------------------------------------------------------------------------


VerdictStatus = Literal["verified", "counter_example", "inconclusive"]


class Verdict(BaseModel):
    """The single output of ``intent_bridge.verify``.

    The status is the headline; the rest is the audit trail. A verdict
    of VERIFIED is meaningful only in the context of the contract that
    was checked, which is why the contract travels with the verdict
    even on the happy path.
    """

    model_config = ConfigDict(frozen=True)

    status: VerdictStatus
    contract: FormalContract | None = None
    counter_example: CounterExample | None = None
    proof_artifact: str | None = None
    inconclusive_reason: str | None = None
    trace: list[Step] = Field(default_factory=list)

    @field_validator("status")
    @classmethod
    def _verified_requires_contract(cls, v: VerdictStatus) -> VerdictStatus:
        # The full cross-field check runs in ``model_post_init`` because
        # field_validator does not see other fields. This is just a
        # placeholder for the type-narrowing case below.
        return v

    def model_post_init(self, __ctx: Any) -> None:
        # Structural invariants. Any breach is a kernel bug — we crash
        # loudly here rather than emit a malformed Verdict that downstream
        # tooling might trust.
        match self.status:
            case "verified":
                if self.contract is None:
                    raise ValueError("verified verdict requires a contract")
                if self.counter_example is not None:
                    raise ValueError("verified verdict must not carry a counter-example")
                if self.proof_artifact is None:
                    raise ValueError("verified verdict requires a proof artifact")
            case "counter_example":
                if self.counter_example is None:
                    raise ValueError("counter_example verdict requires a CounterExample")
            case "inconclusive":
                if self.inconclusive_reason is None:
                    raise ValueError("inconclusive verdict requires a reason")


# ---------------------------------------------------------------------------
# Errors
# ---------------------------------------------------------------------------


class IntentInconsistencyError(ValueError):
    """The user's own inputs contradict each other.

    Raised when the user-supplied examples disagree with a user-supplied
    explicit postcondition, or when two examples contradict each other.
    The kernel will not run on inconsistent intent because no contract
    can satisfy it; surfacing the inconsistency is more useful than any
    verdict.
    """
