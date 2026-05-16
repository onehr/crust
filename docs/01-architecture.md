# Bridge — kernel architecture

> The intent of this document is to lock the protocol between the LLM
> proposer and the symbolic verifier *before* any kernel code is written.
> Schema, types, contracts, and failure modes are decided here; the test
> suite under `tests/` will then encode them as executable assertions; the
> kernel under `src/intent_bridge/` is the smallest implementation that
> makes those assertions pass.

## Verdict space

The kernel emits exactly one of three verdicts. The distinction between
the third and the other two is the most important design decision in the
whole project — see "Honest inconclusive" below.

| Verdict           | Operational meaning                                                                                 |
|-------------------|-----------------------------------------------------------------------------------------------------|
| `VERIFIED`        | A symbolic engine has *proved* that, for every input satisfying the precondition, the implementation's output satisfies the postcondition. Backed by a machine-checkable artifact (z3 UNSAT proof, exhaustive enumeration log). |
| `COUNTER_EXAMPLE` | A concrete input was found for which precondition holds but postcondition fails. Backed by a reproducible artifact (input, observed output, expected output, source of the example). |
| `INCONCLUSIVE`    | Neither of the above could be established within the budget. The implementation may or may not realize the intention; the kernel does not know.                                |

**`VERIFIED` is reserved for positive proof. Absence of counter-examples
is NOT verification.** A run that fuzzes 10,000 inputs without finding a
failure returns `INCONCLUSIVE`, not `VERIFIED`. This rule is what
distinguishes Bridge from LLM-as-judge tools — and from a long line of
test generators that quietly promote "no failures observed" to "passed".

## The three-stage protocol

```
   user --> [Intent]              [Implementation] <-- user
              |                         |
              v                         v
     +--------+-------------------------+--------+
     |             Stage 1: PROPOSE              |
     |        LLM emits a FormalContract         |
     |   (JSON, strict schema, validated)        |
     +--------+-------------------------+--------+
              |
              v
     +--------+-------------------------+--------+
     |             Stage 2: VALIDATE             |
     |   - pydantic schema check                 |
     |   - contract accepts every user example   |
     |   - contract is well-formed z3 / Python   |
     +--------+-------------------------+--------+
              |
              v
     +--------+-------------------------+--------+
     |             Stage 3: DISPOSE              |
     |   - try z3 (symbolic decision)            |
     |   - on UNKNOWN/timeout, try hypothesis    |
     |     (counter-example search)              |
     |   - never promote no-cex-found to VERIFIED|
     +--------+-------------------------+--------+
              |
              v
           [Verdict]
```

Three properties this protocol guarantees by construction:

1. **The LLM is a proposer, never a decider.** Stage 3 ignores the LLM's
   opinion and runs symbolic / property checks against the *contract*.
2. **No free-form LLM output reaches the verifier.** Stage 2 rejects any
   contract that does not parse, that contradicts the user's own
   examples, or that fails schema validation.
3. **The verdict carries an audit trail.** The contract that was checked,
   the engine that produced the verdict, and the concrete artifact
   (model, counter-example, or budget-exhausted log) all travel with
   the verdict for human inspection.

## Data model (pydantic v2)

```python
class Example(BaseModel):
    args: tuple[Any, ...]
    expected: Any

class Intent(BaseModel):
    docstring: str
    examples: list[Example]
    # Optional explicit precondition / postcondition. When absent the
    # LLM is asked to infer them; when present they are treated as
    # ground truth and the LLM is asked only to *translate* them to z3.
    pre_python: str | None = None    # Python lambda source
    post_python: str | None = None   # Python lambda over (args, result)

class Implementation(BaseModel):
    source: str        # the function's source code, importable
    function_name: str

class FormalContract(BaseModel):
    """The LLM's proposal, validated."""
    z3_precondition: str        # SMT-LIB v2 syntax over the function's args
    z3_postcondition: str       # SMT-LIB v2 syntax over args and result
    python_postcondition: str   # callable form: lambda args, result: bool
    arg_sorts: list[str]        # z3 sorts for each arg ("Int", "Real", "Bool")
    result_sort: str
    reasoning: str              # LLM's rationale, retained for audit

class CounterExample(BaseModel):
    args: tuple[Any, ...]
    observed: Any
    expected: Any | None
    discovered_by: Literal["z3", "hypothesis", "user_example"]

class Verdict(BaseModel):
    status: Literal["verified", "counter_example", "inconclusive"]
    contract: FormalContract
    counter_example: CounterExample | None = None
    proof_artifact: str | None = None       # z3 UNSAT trace or enumeration log
    inconclusive_reason: str | None = None
    trace: list[Step]                       # full audit trail
```

## Honest inconclusive — the design value

The single most likely failure mode of any LLM-driven verification tool
is to silently promote "the symbolic backend didn't disagree" to "the
implementation is correct". Bridge structurally prevents this:

- `VERIFIED` requires `engine.status == "unsat"` (no counter-example
  exists in the formal domain) AND the contract that was checked is
  recorded.
- Every other path — z3 returned `unknown`, timeout, function fell
  outside the supported fragment, hypothesis exhausted its budget
  without falsifying — produces `INCONCLUSIVE` with a `reason` string.
- The user is shown the contract that *was* checked, regardless of
  verdict. If they disagree with the contract, the verdict is moot.

This is the same posture taken by Coq's `Admitted`, Dafny's
`{:axiom}`, and CrossHair's `unable to prove` mode. We adopt it
explicitly rather than allow drift.

## Supported fragment for V1

The pilot restricts to functions with signature `(int, ...) -> int`,
i.e. pure integer arithmetic. Bodies may use:

- Arithmetic operators: `+ - * // % **(constant)`
- Comparison: `< <= > >= == !=`
- Conditional expressions and `if/else`
- Boolean ops: `and or not`

Bodies using strings, lists, dicts, recursion, loops, or any external
call are out of fragment. The kernel detects this in Stage 2 and emits
`INCONCLUSIVE` with reason `"out_of_fragment"`. Adding fragments is the
extension axis for V2+.

## SOPs (the hard-constraints layer)

Each layer below is a non-bypassable check. A bug in any of them is
treated as a P0.

| SOP                                | Enforcement                                                                 |
|------------------------------------|-----------------------------------------------------------------------------|
| LLM output schema                  | pydantic strict, refuses unknown keys                                       |
| LLM output sanity (accepts examples)| every `Intent.example` must satisfy `python_postcondition(args, expected)` |
| LLM output well-formedness          | both z3 strings must parse with `z3.parse_smt2_string`                     |
| Per-call LLM token budget          | `OpenAI.chat.completions.create(max_tokens=…)` enforced, retries cap=3      |
| Symbolic engine timeout            | `solver.set("timeout", milliseconds)`; UNKNOWN counts as inconclusive       |
| Fuzz budget                        | `max_examples` parameter; exhaustion counts as inconclusive                 |
| Determinism                        | All tests pin LLM via fixtures; live LLM is `pytest.mark.live` opt-in       |
| Forbidden imports                  | The implementation source is `compile`'d in a restricted globals dict       |

## Test plan (what the suite must establish)

For each property below the suite has at least one positive and one
adversarial test. The kernel passes when every test passes.

1. **Correct impl, decidable contract → `VERIFIED`** (e.g. `abs(x)` correct)
2. **Buggy impl, decidable contract → `COUNTER_EXAMPLE`** (e.g. `abs(x)` returning `x`; expected cex on any negative)
3. **Vague docstring with no examples → `INCONCLUSIVE`** with reason `proposer_uncertain` (LLM may not invent a contract from nothing)
4. **Examples that contradict the docstring → kernel refuses to run**, raises `IntentInconsistencyError`
5. **LLM proposes a contract that does not accept the given examples → `INCONCLUSIVE`** with reason `proposer_inconsistent`; kernel does NOT silently fix
6. **Function outside the V1 fragment (e.g. uses a list) → `INCONCLUSIVE`** with reason `out_of_fragment`
7. **Budget exhausted (z3 timeout AND hypothesis exhausted) → `INCONCLUSIVE`** with reason `budget_exhausted`
8. **Determinism**: two runs with the same fixed LLM fixture produce byte-identical verdicts
9. **Audit trail**: every verdict includes the `FormalContract` actually checked
10. **No `VERIFIED` without proof**: a synthetic run where the engine returns UNKNOWN must emit `INCONCLUSIVE`, never `VERIFIED`

Tests 5, 7, and 10 are the load-bearing ones — they are the structural
defences against the failure modes that destroy the value proposition.

## What V1 does NOT do

The following are deliberate non-goals for the pilot. Each is interesting
and tractable, but extending V1 to cover them widens the surface beyond
what one session can finish.

- Stateful functions, classes, side effects
- Bounded loops or recursion (would need loop invariants)
- Strings, lists, dicts as primary domain
- Multi-function modules
- Concurrency / async
- Performance contracts ("runs in O(n log n)")
- Live LLM in tests (mocks only; `pytest -m live` opts into live)
- A standalone proof checker for the audit trail

Each gets its own roadmap entry in `docs/02-roadmap.md` once V1 lands.
