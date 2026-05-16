"""Bridge — neurosymbolic intent-to-implementation verification.

Public surface:

    from intent_bridge import verify, Intent, Implementation, Example
    from intent_bridge import Verdict, VerdictStatus

The kernel's contract is documented in ``docs/01-architecture.md``.
Read that first; do not infer the protocol from the code alone.
"""

from intent_bridge.kernel import verify
from intent_bridge.types import (
    Budget,
    CounterExample,
    Example,
    FormalContract,
    Implementation,
    Intent,
    Verdict,
    VerdictStatus,
)

__all__ = [
    "Budget",
    "CounterExample",
    "Example",
    "FormalContract",
    "Implementation",
    "Intent",
    "Verdict",
    "VerdictStatus",
    "verify",
]
