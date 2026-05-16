# Bridge — compressed prior-art and gap argument

> This is a *compressed* survey, deliberately not a literature review.
> Its purpose is to anchor the kernel design in the existing landscape
> and to surface the single gap V1 actually fills. A full survey is
> deferred to V2. Where a claim is dated post the author's knowledge
> cutoff (2026-01) and not verified live, the entry is marked
> `[VERIFY]` and should be confirmed before being cited downstream.

## The landscape, in five tiers

### 1. Proof assistants (decider, expert input)

- **Lean 4 + mathlib** — interactive theorem prover with a large
  formalized maths library. LLM ecosystem is the most mature here:
  **LeanDojo** provides a programmatic interface, **Lean Copilot**
  injects LLM tactic suggestions, and Google DeepMind's
  **AlphaProof** [VERIFY] achieved IMO 2024 silver-medal-level results.
  *Leaves to Bridge:* nothing for theorem-proving over maths; everything
  for "translate vague natural-language intent into a contract".
- **Coq / Rocq, Isabelle, Agda, F\*** — same shape, smaller LLM tooling.

### 2. SMT-backed program verifiers (decider, programmer input)

- **Dafny** — write a method and its `requires`/`ensures` in one
  language, the verifier (Boogie + Z3) discharges proof obligations.
  Mature, taught in undergrad. *Leaves to Bridge:* Dafny demands the
  *user* write the contract; Bridge's bet is that an LLM can propose
  the contract from looser inputs (docstring, examples).
- **Why3, Viper, Boogie, Z3 directly** — building blocks beneath Dafny.

### 3. Refinement types (lightweight inline contracts)

- **Liquid Haskell, F\* refinement types, Refined TypeScript** —
  annotations like `{ v: Int | v > 0 }` checked by SMT at type-check
  time. *Leaves to Bridge:* requires user-written refinements;
  doesn't ingest natural-language intent.

### 4. Counter-example searchers (find bugs, never prove)

- **Hypothesis** (Python), **QuickCheck** (Haskell), **proptest**
  (Rust), **Fast-Check** (JS) — property-based testing. *Leaves to
  Bridge:* properties are user-written; framework cannot decide
  whether the property captures the user's intent.
- **CrossHair** — symbolic execution of Python via Z3, finds
  counter-examples to `# pre:` / `# post:` contracts written as
  comments. Closest neighbour to Bridge V1; their contract is human-
  written, ours is LLM-proposed and machine-validated.
- **KLEE, angr, Manticore, CBMC, SeaHorn** — symbolic / bounded model
  checkers for C/C++/binary. Out of Bridge V1 scope (we target Python
  pure functions).

### 5. LLM-in-loop formal tools (the frontier)

This tier is moving fast and is where Bridge sits. Honest assessment of
2024–2025 prior work:

- **Draft-Sketch-Prove** (Jiang et al., 2023) — LLM drafts informal
  proof, sketches Isabelle skeleton, automated solver fills holes.
  Pattern Bridge inherits: LLM proposes, formal disposes.
- **Baldur** (Microsoft / First, 2023) — LLM generates whole proofs
  for Isabelle; uses repair loop on failure.
- **Copra** (Thakur et al., 2023) — Coq proof search agent that
  queries LLM at each step, validates with the kernel.
- **LeanDojo + ReProver** (Yang et al., 2023) — RAG-augmented Lean
  proof search, MIT-licensed dataset.
- **AlphaProof / DeepSeek-Prover-V1.5 / Goedel-Prover** [VERIFY] —
  RL-trained proof models on Lean, increasingly competitive with
  expert human provers.
- **LLMSTEP, ntp-toolkit, LeanAgent** — incremental tactic
  prediction tools.

What is conspicuously absent from this list: **a tool that takes a
docstring + examples for a pure Python function and emits a
machine-checkable verdict**. The frontier is concentrated on theorem
proving in mathlib-scale formal libraries, *not* on the practical
"does this 20-line function do what the comment says" question.

## Gap argument

Existing tools partition cleanly along two axes:

```
        user writes formal spec      user writes informal spec
       +-----------------------+   +-----------------------+
  formal| Dafny, Liquid Haskell |   | (empty)              |
 verdict| F*, CrossHair         |   |                      |
       +-----------------------+   +-----------------------+
        | Hypothesis, KLEE       |   | LLM-as-judge, MT-Bench|
  weak  | (finds bugs, never     |   | (no verdict you can  |
verdict | proves)                |   | check)               |
       +-----------------------+   +-----------------------+
```

The top-right quadrant is empty: *informal spec in, formal verdict out*.
Bridge V1 occupies exactly this cell, with the deliberate posture that
when the symbolic engine cannot produce a formal verdict the tool
returns `INCONCLUSIVE` rather than fall back into the bottom-right.

The LLM's role is purely **translation between quadrants** (informal
intent → formal contract). Verdict comes from the formal engine alone.

## Why not just contribute to CrossHair / Dafny / LeanDojo

Honest answers:

- **CrossHair** is the closest fit. The right question is whether
  Bridge V1 is "CrossHair plus an LLM contract proposer". Yes — and
  if V1 converges, a contributing relationship with the CrossHair
  maintainers is the obvious next step. Building the proposer +
  validator + audit trail as a separate kernel first lets us iterate
  on the *LLM protocol* without coupling to a moving target. The
  symbolic engine could be swapped for CrossHair in V2.
- **Dafny** expects users who can read pre/post conditions. Bridge's
  thesis is that LLMs let us widen the user base; that requires the
  proposer to be first-class, not glued on.
- **LeanDojo** is theorem-proving, not code verification. Different
  problem domain.

## What this gap is worth

A working Bridge would let a reviewer (human or AI agent) answer the
single question "does this PR's implementation match its commit
message / docstring / linked ticket?" with a machine-checkable
artifact, on the subset of code where the symbolic engine can decide,
and with an *honest* "inconclusive" elsewhere.

The honest "inconclusive" is the whole point. The tool's value comes
from being trustable when it says VERIFIED or COUNTER_EXAMPLE, not
from a high VERIFIED rate.

## Open risks to this framing

1. **The LLM cannot reliably translate informal intent to formal
   contracts on the long tail.** Mitigation: make the contract
   visible in the audit trail, let the user reject and re-prompt.
2. **The supported fragment is too narrow to be useful.** V1 is pure
   integer arithmetic; if the tool can't generalize, value is small.
   Test: pick three "real" functions from open-source projects in V2
   and see if they fall in or out of fragment.
3. **CrossHair plus a thin LLM wrapper is the right product**, and
   building a separate kernel is wasted motion. Mitigation: keep
   the LLM-symbolic boundary clean enough that the kernel can be
   re-targeted at CrossHair, KLEE, etc., without reshaping the
   protocol.
