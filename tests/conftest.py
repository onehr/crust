"""Shared pytest fixtures for the Bridge test suite.

The most important fixture is ``mock_proposer``: a deterministic
LLM-proposer stand-in that lets the rest of the test suite assert on
kernel behavior without depending on a live LLM. The architecture
document requires that tests be deterministic; live LLM calls happen
only under ``pytest -m live``.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Callable

import pytest

from intent_bridge.proposer import LLMProposer
from intent_bridge.types import FormalContract, Implementation, Intent


@dataclass
class MockProposer:
    """A LLMProposer-shaped object that returns canned contracts.

    ``responder`` maps an (intent.docstring, impl.function_name) pair to
    the FormalContract that this mock will return. Unknown keys raise,
    so a test that forgets to register its response fails loudly rather
    than silently falling through.
    """

    responder: Callable[[Intent, Implementation], FormalContract]
    calls: list[tuple[Intent, Implementation]]

    def propose(self, intent: Intent, impl: Implementation) -> FormalContract:
        self.calls.append((intent, impl))
        return self.responder(intent, impl)


@pytest.fixture
def mock_proposer() -> Callable[[Callable[[Intent, Implementation], FormalContract]], MockProposer]:
    """Returns a factory: pass a responder, get a MockProposer."""

    def factory(responder: Callable[[Intent, Implementation], FormalContract]) -> MockProposer:
        return MockProposer(responder=responder, calls=[])

    return factory


@pytest.fixture
def abs_contract() -> FormalContract:
    """A correct, well-formed contract for `abs(x: int) -> int`.

    Reused across multiple tests. The z3 form is in SMT-LIB v2 syntax;
    the Python form is a lambda string that the validator can `exec`.
    """
    # SMT-LIB expressions only; declarations are added by the kernel
    # using the arg_sorts / result_sort fields. This keeps the LLM
    # contract small and disentangles "what variables exist" from
    # "what holds about them".
    return FormalContract(
        z3_precondition="true",
        z3_postcondition="(= result (ite (< x 0) (- x) x))",
        python_postcondition="lambda args, result: result == (-args[0] if args[0] < 0 else args[0])",
        arg_sorts=["Int"],
        result_sort="Int",
        reasoning="abs(x) returns -x for negative x, x otherwise.",
    )


@pytest.fixture
def abs_correct_impl() -> Implementation:
    return Implementation(
        source="def abs_(x):\n    return -x if x < 0 else x\n",
        function_name="abs_",
    )


@pytest.fixture
def abs_buggy_impl() -> Implementation:
    """An impl that ignores the negative branch — must yield COUNTER_EXAMPLE."""
    return Implementation(
        source="def abs_(x):\n    return x\n",
        function_name="abs_",
    )


@pytest.fixture
def abs_intent() -> Intent:
    from intent_bridge.types import Example

    return Intent(
        docstring="Return the absolute value of an integer x.",
        examples=[
            Example(args=(0,), expected=0),
            Example(args=(5,), expected=5),
            Example(args=(-3,), expected=3),
        ],
    )
