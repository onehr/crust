"""Python AST -> z3 expression translator (V1 fragment).

The V1 fragment is small on purpose: pure integer arithmetic, no loops,
no recursion, no calls. The translator's job is to map a Python
function body to a single z3 expression whose value equals what the
function returns on every input.

Anything outside the fragment raises :class:`OutOfFragmentError`. The
kernel catches that and emits ``INCONCLUSIVE / out_of_fragment``.
"""

from __future__ import annotations

import ast
from typing import cast

import z3


class OutOfFragmentError(Exception):
    """Raised when the function uses Python features V1 does not handle."""


# ---------------------------------------------------------------------------
# Sort handling
# ---------------------------------------------------------------------------


_SORT_FACTORIES = {
    "Int": z3.Int,
    "Real": z3.Real,
    "Bool": z3.Bool,
}


def make_const(name: str, sort: str) -> z3.ExprRef:
    """Build a z3 constant of the named sort. Used for arg / result vars."""
    try:
        factory = _SORT_FACTORIES[sort]
    except KeyError as e:
        raise OutOfFragmentError(f"unsupported sort: {sort!r}") from e
    return factory(name)


# ---------------------------------------------------------------------------
# Function extraction
# ---------------------------------------------------------------------------


def extract_function(source: str, function_name: str) -> ast.FunctionDef:
    """Return the top-level FunctionDef matching ``function_name``.

    Top-level only: we deliberately do not look inside classes or
    nested defs. Bringing those in widens the fragment without buying
    much for the integer-arithmetic V1.
    """
    module = ast.parse(source)
    for node in module.body:
        if isinstance(node, ast.FunctionDef) and node.name == function_name:
            return node
    raise OutOfFragmentError(
        f"no top-level function named {function_name!r} in source"
    )


def extract_arg_names(fn: ast.FunctionDef) -> list[str]:
    """Return positional argument names. Disallow defaults / *args / **kwargs."""
    args = fn.args
    if args.vararg or args.kwarg or args.kwonlyargs or args.posonlyargs:
        raise OutOfFragmentError("only simple positional args are supported")
    if args.defaults:
        raise OutOfFragmentError("default arguments are not supported in V1")
    return [a.arg for a in args.args]


# ---------------------------------------------------------------------------
# Translator core
# ---------------------------------------------------------------------------


class _Translator:
    """Convert a function body to a single z3 expression.

    The strategy is path-aware: every ``return`` statement contributes
    a (path_condition, value) pair, and the final z3 expression is a
    nested ``If(...)`` that picks the value whose path condition holds.

    This handles the common Python patterns we care about for V1:

        def f(x):
            if x < 0:
                return -x
            return x

        def g(a, b):
            return a if a > b else b
    """

    def __init__(self, args_z3: dict[str, z3.ExprRef]) -> None:
        self._args = args_z3

    # ------------------------------------------------------------------
    # Top-level: translate a list of statements (the function body)
    # ------------------------------------------------------------------

    def translate_body(self, stmts: list[ast.stmt]) -> z3.ExprRef:
        return self._translate_seq(stmts)

    def _translate_seq(self, stmts: list[ast.stmt]) -> z3.ExprRef:
        """Reduce a sequence of statements to the expression they yield.

        Every path through the sequence MUST end in a ``return``. A
        sequence with no return on some path is rejected as outside
        the fragment because we cannot ascribe a value to that path.
        """
        if not stmts:
            raise OutOfFragmentError("statement sequence does not end in a return")

        first, rest = stmts[0], stmts[1:]

        if isinstance(first, ast.Return):
            if first.value is None:
                raise OutOfFragmentError("bare `return` (returns None) is unsupported")
            if rest:
                # Code after a return is dead; we accept the function
                # but warn by raising — V1 is strict about cleanliness.
                raise OutOfFragmentError("unreachable code after `return`")
            return self._translate_expr(first.value)

        if isinstance(first, ast.If):
            cond = self._translate_expr(first.test)
            then_expr = self._translate_seq(first.body)
            # If the `if` has an explicit `else`, use it; otherwise
            # the subsequent statements form the fall-through branch.
            else_stmts = first.orelse if first.orelse else rest
            else_expr = self._translate_seq(else_stmts)
            return z3.If(cond, then_expr, else_expr)

        # Anything else (Assign, For, While, Expr, etc.) is out of fragment.
        raise OutOfFragmentError(
            f"statement of type {type(first).__name__} is not in the V1 fragment"
        )

    # ------------------------------------------------------------------
    # Expressions
    # ------------------------------------------------------------------

    def _translate_expr(self, expr: ast.expr) -> z3.ExprRef:
        if isinstance(expr, ast.Constant):
            return self._translate_constant(expr)
        if isinstance(expr, ast.Name):
            return self._translate_name(expr)
        if isinstance(expr, ast.UnaryOp):
            return self._translate_unary(expr)
        if isinstance(expr, ast.BinOp):
            return self._translate_binop(expr)
        if isinstance(expr, ast.BoolOp):
            return self._translate_boolop(expr)
        if isinstance(expr, ast.Compare):
            return self._translate_compare(expr)
        if isinstance(expr, ast.IfExp):
            return self._translate_ifexp(expr)
        raise OutOfFragmentError(
            f"expression of type {type(expr).__name__} is not in the V1 fragment"
        )

    def _translate_constant(self, expr: ast.Constant) -> z3.ExprRef:
        v = expr.value
        if isinstance(v, bool):
            return z3.BoolVal(v)
        if isinstance(v, int):
            return z3.IntVal(v)
        if isinstance(v, float):
            return z3.RealVal(v)
        raise OutOfFragmentError(f"constant of type {type(v).__name__} unsupported")

    def _translate_name(self, expr: ast.Name) -> z3.ExprRef:
        if expr.id in self._args:
            return self._args[expr.id]
        raise OutOfFragmentError(
            f"free variable {expr.id!r} (V1 only supports function args)"
        )

    def _translate_unary(self, expr: ast.UnaryOp) -> z3.ExprRef:
        operand = self._translate_expr(expr.operand)
        match expr.op:
            case ast.USub():
                return cast(z3.ExprRef, -operand)
            case ast.UAdd():
                return operand
            case ast.Not():
                return z3.Not(operand)
            case _:
                raise OutOfFragmentError(f"unary op {type(expr.op).__name__} unsupported")

    def _translate_binop(self, expr: ast.BinOp) -> z3.ExprRef:
        left = self._translate_expr(expr.left)
        right = self._translate_expr(expr.right)
        match expr.op:
            case ast.Add():
                return cast(z3.ExprRef, left + right)
            case ast.Sub():
                return cast(z3.ExprRef, left - right)
            case ast.Mult():
                return cast(z3.ExprRef, left * right)
            case ast.FloorDiv():
                # z3's `/` on Int sort is integer division. Python's
                # `//` floors toward negative infinity; z3 truncates
                # toward zero. The two agree on non-negative operands;
                # they disagree on negative dividends. We accept the
                # divergence in V1 and document it as a limitation;
                # functions that rely on negative floor-div will get
                # a false counter-example or false verification.
                return cast(z3.ExprRef, left / right)
            case ast.Mod():
                return cast(z3.ExprRef, left % right)
            case _:
                raise OutOfFragmentError(f"binop {type(expr.op).__name__} unsupported")

    def _translate_boolop(self, expr: ast.BoolOp) -> z3.ExprRef:
        operands = [self._translate_expr(v) for v in expr.values]
        match expr.op:
            case ast.And():
                return z3.And(*operands)
            case ast.Or():
                return z3.Or(*operands)
            case _:
                raise OutOfFragmentError(f"boolop {type(expr.op).__name__} unsupported")

    def _translate_compare(self, expr: ast.Compare) -> z3.ExprRef:
        if len(expr.ops) != 1 or len(expr.comparators) != 1:
            # Python allows `a < b < c`; we do not, to keep semantics
            # explicit. The user can rewrite as `a < b and b < c`.
            raise OutOfFragmentError("chained comparisons (a < b < c) are unsupported")
        left = self._translate_expr(expr.left)
        right = self._translate_expr(expr.comparators[0])
        match expr.ops[0]:
            case ast.Lt():
                return cast(z3.ExprRef, left < right)
            case ast.LtE():
                return cast(z3.ExprRef, left <= right)
            case ast.Gt():
                return cast(z3.ExprRef, left > right)
            case ast.GtE():
                return cast(z3.ExprRef, left >= right)
            case ast.Eq():
                return cast(z3.ExprRef, left == right)
            case ast.NotEq():
                return cast(z3.ExprRef, left != right)
            case _:
                raise OutOfFragmentError(
                    f"comparison {type(expr.ops[0]).__name__} unsupported"
                )

    def _translate_ifexp(self, expr: ast.IfExp) -> z3.ExprRef:
        test = self._translate_expr(expr.test)
        body = self._translate_expr(expr.body)
        orelse = self._translate_expr(expr.orelse)
        return z3.If(test, body, orelse)


# ---------------------------------------------------------------------------
# Public entry point
# ---------------------------------------------------------------------------


def translate(
    source: str,
    function_name: str,
    args_z3: dict[str, z3.ExprRef],
) -> z3.ExprRef:
    """Translate the function ``function_name`` in ``source`` to a z3 expression.

    Raises :class:`OutOfFragmentError` if any part of the function is
    outside the V1 fragment.
    """
    fn = extract_function(source, function_name)
    arg_names = extract_arg_names(fn)
    # Sanity: every arg the function declares must have a z3 binding.
    for name in arg_names:
        if name not in args_z3:
            raise OutOfFragmentError(
                f"no z3 binding for parameter {name!r}; "
                f"check arg_sorts in the contract"
            )
    return _Translator(args_z3).translate_body(fn.body)
