"""Command-line surface for Bridge.

Two subcommands ship with V1:

* ``bridge verify`` — verify a function against an intent file
* ``bridge demo``   — run the canonical abs/buggy-abs demo

The CLI is a thin wrapper around :func:`intent_bridge.verify`. It
does no business logic; if you find yourself reaching for a flag
that shapes a verdict, that probably belongs in the kernel, not here.
"""

from __future__ import annotations

import json
import os
import sys
from pathlib import Path
from typing import Annotated

import typer
from rich.console import Console
from rich.panel import Panel
from rich.syntax import Syntax
from rich.table import Table

from intent_bridge import Budget, Example, Implementation, Intent, verify
from intent_bridge.proposer import OpenRouterProposer

app = typer.Typer(
    name="bridge",
    help=(
        "Neurosymbolic intent-to-implementation verification. The LLM "
        "proposes a formal contract; the SMT engine disposes."
    ),
    no_args_is_help=True,
    add_completion=False,
)

_console = Console()


# ---------------------------------------------------------------------------
# `bridge verify`
# ---------------------------------------------------------------------------


@app.command()
def verify_cmd(
    intent_path: Annotated[Path, typer.Argument(help="Path to intent JSON file.")],
    impl_path: Annotated[Path, typer.Argument(help="Path to implementation .py file.")],
    function_name: Annotated[
        str, typer.Option("--function", "-f", help="Name of the function to verify.")
    ],
    model: Annotated[
        str,
        typer.Option(
            "--model",
            "-m",
            help="OpenRouter / OpenAI-compatible model identifier.",
        ),
    ] = "anthropic/claude-sonnet-4.5",
    z3_timeout_ms: Annotated[
        int, typer.Option(help="z3 solver timeout in milliseconds.")
    ] = 5000,
    fuzz_max_examples: Annotated[
        int, typer.Option(help="Maximum hypothesis examples in the fuzz fallback.")
    ] = 200,
    json_output: Annotated[
        bool, typer.Option("--json", help="Emit the verdict as JSON only.")
    ] = False,
) -> None:
    """Verify that the function in IMPL_PATH realizes the intent in INTENT_PATH."""

    intent_data = json.loads(intent_path.read_text())
    intent = Intent.model_validate(intent_data)
    impl = Implementation(
        source=impl_path.read_text(),
        function_name=function_name,
    )
    budget = Budget(z3_timeout_ms=z3_timeout_ms, fuzz_max_examples=fuzz_max_examples)

    proposer = OpenRouterProposer(model=model)
    verdict = verify(intent, impl, proposer=proposer, budget=budget)

    if json_output:
        print(verdict.model_dump_json(indent=2))
        raise typer.Exit(code=_exit_code_for(verdict.status))

    _render_verdict_rich(impl, intent, verdict)
    raise typer.Exit(code=_exit_code_for(verdict.status))


# ---------------------------------------------------------------------------
# `bridge demo`
# ---------------------------------------------------------------------------


@app.command()
def demo(
    use_live_llm: Annotated[
        bool,
        typer.Option(
            "--live",
            help=(
                "Call a real LLM via OpenRouter (requires OPENROUTER_API_KEY). "
                "Off by default; the demo runs against a hard-coded contract "
                "so it is deterministic and offline."
            ),
        ),
    ] = False,
    model: Annotated[
        str, typer.Option("--model", "-m", help="LLM model id for --live mode.")
    ] = "anthropic/claude-sonnet-4.5",
) -> None:
    """Walk through the canonical abs() verification.

    First runs the verifier on a correct ``abs`` implementation
    (expected: VERIFIED). Then runs it on a buggy implementation that
    forgets the negative branch (expected: COUNTER_EXAMPLE with a
    negative integer as the witness).
    """

    intent = Intent(
        docstring="Return the absolute value of an integer x.",
        examples=[
            Example(args=(0,), expected=0),
            Example(args=(5,), expected=5),
            Example(args=(-3,), expected=3),
        ],
    )

    correct_impl = Implementation(
        source="def abs_(x):\n    return -x if x < 0 else x\n",
        function_name="abs_",
    )
    buggy_impl = Implementation(
        source="def abs_(x):\n    return x\n",
        function_name="abs_",
    )

    if use_live_llm:
        if not (os.environ.get("OPENROUTER_API_KEY") or os.environ.get("OPENAI_API_KEY")):
            _console.print(
                "[red]--live requires OPENROUTER_API_KEY or OPENAI_API_KEY[/red]"
            )
            raise typer.Exit(code=2)
        proposer = OpenRouterProposer(model=model)
    else:
        proposer = _DemoMockProposer()

    for label, impl in [("correct abs", correct_impl), ("buggy abs (returns x)", buggy_impl)]:
        _console.rule(f"[bold]demo: {label}")
        verdict = verify(intent, impl, proposer=proposer)
        _render_verdict_rich(impl, intent, verdict)


# ---------------------------------------------------------------------------
# Demo proposer (mock; pinned to the correct abs contract)
# ---------------------------------------------------------------------------


class _DemoMockProposer:
    """A proposer that always returns the canonical abs() contract.

    Used by ``bridge demo`` when ``--live`` is not set so the demo is
    deterministic and offline. The contract here is hand-written; in
    live mode the LLM produces the same shape from the docstring.
    """

    def propose(self, intent, impl):  # type: ignore[no-untyped-def]
        from intent_bridge.types import FormalContract

        return FormalContract(
            z3_precondition="true",
            z3_postcondition="(= result (ite (< x 0) (- x) x))",
            python_postcondition=(
                "lambda args, result: result == (-args[0] if args[0] < 0 else args[0])"
            ),
            python_precondition="lambda args: True",
            arg_sorts=["Int"],
            result_sort="Int",
            reasoning="abs(x) returns -x for negative x, x otherwise.",
        )


# ---------------------------------------------------------------------------
# Pretty-printing
# ---------------------------------------------------------------------------


def _render_verdict_rich(impl: Implementation, intent: Intent, verdict) -> None:  # type: ignore[no-untyped-def]
    status_style = {
        "verified": "bold green",
        "counter_example": "bold red",
        "inconclusive": "bold yellow",
    }[verdict.status]

    _console.print()
    _console.print(Panel(
        Syntax(impl.source.strip(), "python", theme="ansi_dark"),
        title=f"implementation: {impl.function_name}",
        border_style="dim",
    ))
    _console.print(Panel(intent.docstring, title="intent", border_style="dim"))

    _console.print(f"\nverdict: [{status_style}]{verdict.status.upper()}[/]")

    if verdict.contract is not None:
        ct = Table(show_header=False, padding=(0, 1))
        ct.add_row("z3 pre", verdict.contract.z3_precondition)
        ct.add_row("z3 post", verdict.contract.z3_postcondition)
        ct.add_row("python post", verdict.contract.python_postcondition)
        ct.add_row("reasoning", verdict.contract.reasoning)
        _console.print(Panel(ct, title="contract (what was actually checked)", border_style="dim"))

    if verdict.counter_example is not None:
        cex = verdict.counter_example
        ct = Table(show_header=False, padding=(0, 1))
        ct.add_row("args", repr(cex.args))
        ct.add_row("observed", repr(cex.observed))
        ct.add_row("expected", repr(cex.expected))
        ct.add_row("discovered by", cex.discovered_by)
        _console.print(Panel(ct, title="counter-example", border_style="red"))

    if verdict.proof_artifact is not None:
        _console.print(Panel(verdict.proof_artifact, title="proof artifact", border_style="green"))

    if verdict.inconclusive_reason is not None:
        _console.print(Panel(verdict.inconclusive_reason, title="inconclusive reason", border_style="yellow"))

    # Audit trail (compact).
    trace_table = Table(title="audit trail", show_lines=False)
    trace_table.add_column("stage")
    trace_table.add_column("event")
    trace_table.add_column("detail", overflow="fold")
    for step in verdict.trace:
        trace_table.add_row(step.stage, step.event, json.dumps(step.detail) if step.detail else "")
    _console.print(trace_table)


def _exit_code_for(status: str) -> int:
    """Map a verdict status to a CLI exit code."""
    return {
        "verified": 0,
        "counter_example": 1,
        "inconclusive": 2,
    }[status]


if __name__ == "__main__":
    app()  # pragma: no cover
