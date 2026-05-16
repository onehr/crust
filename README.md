# Bridge

> Neurosymbolic intent-to-implementation verification.
> An LLM proposes a formal contract; an SMT solver disposes.
> When the symbolic backend cannot decide, the verdict is
> `INCONCLUSIVE` — *never* a false `VERIFIED`.

## What this is

A small kernel that answers, for a candidate implementation, one hard
question:

> Does this implementation actually realize the intention it claims to
> realize, and what is the evidence?

with confidence strictly higher than "an LLM said so", and cost
strictly lower than "a human wrote a Coq proof".

The bet: **LLMs are good *proposers* of formal contracts and bad
*deciders* of correctness; SMT solvers are good deciders and bad
proposers. A disciplined composition can outperform either alone.**

## Quickstart

```bash
# clone + install (uv is recommended; pip also works)
uv sync --extra dev
uv pip install -e .

# the kernel ships with a deterministic offline demo
uv run bridge demo

# run the full test suite (10 tests; mock LLM, no network)
uv run pytest

# live verification with a real LLM (requires OPENROUTER_API_KEY)
export OPENROUTER_API_KEY=sk-or-...
uv run bridge demo --live
```

## What a verdict looks like

For a correct `abs(x)`:

```
verdict: VERIFIED
proof artifact: z3 UNSAT for (pre AND result==body AND NOT post);
                checked under timeout 5000 ms; args=['x']; sorts=['Int']->Int
```

For a buggy `abs(x): return x`:

```
verdict: COUNTER_EXAMPLE
args:        (-1,)
observed:    -1
expected:    1
discovered:  z3
```

Both verdicts also surface the *contract that was checked* — the
LLM-proposed formal interpretation of the user's intent — so a human
auditor can confirm the verdict matches what they actually wanted to
verify.

## The three verdicts

| Verdict           | Meaning                                                                                                |
|-------------------|--------------------------------------------------------------------------------------------------------|
| `VERIFIED`        | The symbolic engine *proved* the implementation satisfies the contract on every input. Backed by a machine-checkable artifact. |
| `COUNTER_EXAMPLE` | A concrete input was found for which the implementation violates the contract. Backed by reproducible (args, observed, expected). |
| `INCONCLUSIVE`    | Neither could be established within budget. The implementation may or may not realize the intention; the kernel does not know. |

`VERIFIED` is reserved for *positive proof*. Absence of counter-examples
is **not** verification. A run that fuzzes 10 000 inputs without finding
a failure returns `INCONCLUSIVE`, never `VERIFIED`. This rule is what
distinguishes Bridge from LLM-as-judge wrappers.

## Architecture

```
   user --> [Intent]                       [Implementation] <-- user
              |                                   |
              v                                   v
              +-------- Stage 1: PROPOSE ---------+
              |   LLM emits a FormalContract      |
              |   (JSON, strict schema)           |
              +-----------------+-----------------+
                                |
              +--------- Stage 2: VALIDATE -------+
              |   schema check + accepts examples |
              |   + well-formed z3 / Python       |
              +-----------------+-----------------+
                                |
              +--------- Stage 3: DISPOSE --------+
              |   z3 symbolic decision            |
              |   then hypothesis fuzz fallback   |
              |   never promotes "no cex" to     |
              |   "verified"                      |
              +-----------------+-----------------+
                                |
                                v
                            [Verdict]
```

See [`docs/01-architecture.md`](docs/01-architecture.md) for the
detailed protocol, supported fragment, and SOP layer.

## Documentation

| File                          | What it covers                                                  |
|-------------------------------|-----------------------------------------------------------------|
| [`docs/00-prior-art.md`](docs/00-prior-art.md)        | Compressed survey of existing tools and the gap argument |
| [`docs/01-architecture.md`](docs/01-architecture.md) | Kernel design, types, SOPs, supported fragment, test plan |

## Status

V1 pilot. The supported fragment is intentionally small:

* Pure Python functions of integer arguments returning an integer
* Bodies using arithmetic, comparison, conditional expressions, and
  `if/else` returns
* No loops, recursion, classes, or external calls

Functions outside the fragment are reported as `INCONCLUSIVE` with
reason `out_of_fragment`. Extending the fragment is the V2 roadmap.

## What this is not

* Not an LLM coding agent. Cursor, Aider, Claude Code, Devin already exist.
* Not an LLM-as-judge wrapper. The LLM proposes; it never decides.
* Not a UX layer over Coq. The contract is auto-proposed, not user-written.
* Not a test generator. The output is a verdict with a proof artifact, not more tests.

## Acknowledgements / prior art

Bridge sits in a crowded space. The closest direct neighbours are
**CrossHair** (Python symbolic execution against hand-written
contracts) and the **Draft-Sketch-Prove** family of LLM-formal hybrids.
See [`docs/00-prior-art.md`](docs/00-prior-art.md) for the honest
positioning.

## Legacy

This repository previously hosted `crust`, a toy C compiler written
in Rust while the author was learning Rust. That project lives at
[`legacy/crust/`](legacy/crust/) and is preserved as a historical
learning artifact; it is no longer under active development. Bridge
is a fresh project that shares the repository, not the codebase.

## License

Apache 2.0 — see [`LICENSE`](LICENSE).
