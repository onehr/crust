"""Stage 1 of the kernel protocol: LLM proposes a FormalContract.

A proposer's only job is to translate an ``Intent`` plus an
``Implementation`` into a ``FormalContract`` that the rest of the
kernel can validate and verify against. The proposer is a *trusted*
component for translation only; its output is treated as an
adversarial input to validation (Stage 2).

V1 ships two proposers:

* ``OpenRouterProposer`` — production proposer, talks to an
  OpenAI-compatible API endpoint (defaults to OpenRouter). Requires
  ``OPENROUTER_API_KEY`` (or ``OPENAI_API_KEY``) in the environment.
* The test suite uses a ``MockProposer`` (see ``tests/conftest.py``)
  that returns canned contracts.

Adding a new proposer means writing a class with a ``propose`` method
matching the ``LLMProposer`` protocol. The kernel does not depend on
any concrete proposer class.
"""

from __future__ import annotations

import json
import os
from typing import Protocol

from pydantic import ValidationError

from intent_bridge.types import FormalContract, Implementation, Intent


class ProposerUncertainError(Exception):
    """A proposer signals it cannot translate the intent into a contract.

    Raise this when:

    * The docstring is too vague to determine what the function does.
    * The examples are insufficient to disambiguate between candidate
      contracts.
    * The function falls outside the proposer's competence.

    The kernel catches this and emits an INCONCLUSIVE verdict with
    reason ``proposer_uncertain``. *Never* return a guessed contract
    in this situation; the entire point of the protocol is that the
    proposer abstains when it is not confident.
    """


class ProposerInvalidOutputError(Exception):
    """The LLM returned something that does not match the FormalContract schema.

    Raised after exhausting the retry budget. The kernel treats this
    the same as ``ProposerUncertainError`` for verdict purposes, but
    the distinction is preserved in the audit trail so that downstream
    tooling can tell "the LLM gave up" apart from "the LLM is broken".
    """


class LLMProposer(Protocol):
    """Structural type for proposers — a single ``propose`` method.

    Implementations should be deterministic for a fixed seed if they
    intend to be used in tests; the production OpenRouter proposer is
    deliberately non-deterministic, so the test suite uses mocks.
    """

    def propose(self, intent: Intent, impl: Implementation) -> FormalContract:
        ...


# ---------------------------------------------------------------------------
# OpenRouter (OpenAI-compatible) proposer
# ---------------------------------------------------------------------------


_SYSTEM_PROMPT = """\
You are the symbolic-contract proposer in a neurosymbolic verification
tool. You will be given a user's INTENT (natural-language docstring
plus optional input/output examples) and a candidate IMPLEMENTATION
(a Python function). Your job is to emit a JSON object describing the
contract the implementation must satisfy.

You are NOT being asked to judge whether the implementation is correct.
A downstream SMT solver will do that. Your job is purely to translate
the user's intent into formal language.

Output schema (strict — extra keys will be rejected):

{
  "z3_precondition": "<SMT-LIB v2 boolean expression>",
  "z3_postcondition": "<SMT-LIB v2 boolean expression>",
  "python_postcondition": "lambda args, result: <bool>",
  "arg_sorts": ["Int" | "Real" | "Bool", ...],
  "result_sort": "Int" | "Real" | "Bool",
  "reasoning": "<one or two sentences justifying the contract>"
}

Rules:

1. The SMT expressions reference free variables named after the
   function's arguments and a free variable named "result". Do NOT
   include "(declare-const ...)" — declarations are added by the
   kernel.

2. The Python postcondition is a lambda taking (args: tuple, result)
   and returning bool. It must agree with the SMT postcondition on
   every input.

3. If the intent is too vague to translate, OR the function is outside
   the integer-arithmetic fragment, return exactly the JSON object
   {"abstain": "<one-sentence reason>"} and nothing else. Do not guess.

4. The postcondition MUST accept every (args, expected) pair in the
   user's examples. If it would reject any example, you have
   misinterpreted the intent — abstain instead.
"""


_USER_PROMPT_TEMPLATE = """\
INTENT:
- Docstring: {docstring}
- Examples: {examples_json}
- Explicit precondition (Python): {pre_python}
- Explicit postcondition (Python): {post_python}

IMPLEMENTATION:
```python
{source}
```
Function name: {function_name}

Emit ONLY the JSON object. No prose, no markdown fences.
"""


class OpenRouterProposer:
    """Production proposer using the OpenAI-compatible API.

    Defaults to OpenRouter (https://openrouter.ai) but works against
    any OpenAI-compatible endpoint by overriding ``base_url``. The
    constructor is intentionally permissive about model name — the
    caller picks the routing.

    This proposer is NOT used in unit tests (live LLM calls violate
    the determinism requirement); see ``MockProposer`` in
    ``tests/conftest.py``.
    """

    def __init__(
        self,
        model: str = "anthropic/claude-sonnet-4.5",
        api_key: str | None = None,
        base_url: str = "https://openrouter.ai/api/v1",
        max_tokens: int = 2048,
        max_retries: int = 2,
        request_timeout_s: float = 30.0,
    ) -> None:
        api_key = api_key or os.environ.get("OPENROUTER_API_KEY") or os.environ.get(
            "OPENAI_API_KEY"
        )
        if api_key is None:
            raise RuntimeError(
                "OpenRouterProposer needs an API key in OPENROUTER_API_KEY "
                "or OPENAI_API_KEY, or pass api_key= to the constructor."
            )
        # Lazy import so the rest of the package works without `openai`
        # installed (in particular, the test suite must run with mocks).
        from openai import OpenAI

        self._client = OpenAI(api_key=api_key, base_url=base_url, timeout=request_timeout_s)
        self._model = model
        self._max_tokens = max_tokens
        self._max_retries = max_retries

    def propose(self, intent: Intent, impl: Implementation) -> FormalContract:
        prompt = _USER_PROMPT_TEMPLATE.format(
            docstring=intent.docstring,
            examples_json=json.dumps([ex.model_dump() for ex in intent.examples]),
            pre_python=intent.pre_python or "(none)",
            post_python=intent.post_python or "(none)",
            source=impl.source,
            function_name=impl.function_name,
        )

        last_error: Exception | None = None
        for attempt in range(self._max_retries + 1):
            try:
                response = self._client.chat.completions.create(
                    model=self._model,
                    messages=[
                        {"role": "system", "content": _SYSTEM_PROMPT},
                        {"role": "user", "content": prompt},
                    ],
                    max_tokens=self._max_tokens,
                    temperature=0.0,
                    response_format={"type": "json_object"},
                )
                content = response.choices[0].message.content
                if content is None:
                    raise ProposerInvalidOutputError("empty response")
                payload = json.loads(content)
                if "abstain" in payload:
                    raise ProposerUncertainError(str(payload["abstain"]))
                return FormalContract.model_validate(payload)
            except ProposerUncertainError:
                raise
            except (json.JSONDecodeError, ValidationError, ProposerInvalidOutputError) as e:
                last_error = e
                continue
        raise ProposerInvalidOutputError(
            f"LLM did not produce a valid contract after {self._max_retries + 1} attempts: {last_error}"
        )
