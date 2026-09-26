"""Syntax-aware one-assertion policy for Python repository tests (standard library only)."""

import ast
from pathlib import Path
import subprocess
import sys


def assertion_count(function):
    """Count assert statements and unittest/mock assertion calls, excluding nested functions."""
    count = 0
    pending = list(function.body)
    while pending:
        node = pending.pop()
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            continue
        if isinstance(node, ast.Assert):
            count += 1
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute):
            if node.func.attr.startswith("assert"):
                count += 1
        pending.extend(ast.iter_child_nodes(node))
    return count


def diagnostics(source):
    """Compute diagnostics without reading files or executing the source."""
    tree = ast.parse(source)
    errors = []
    for node in ast.walk(tree):
        if not isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            continue
        count = assertion_count(node)
        expected = 1 if node.name.startswith("test_") else 0
        if count != expected:
            errors.append((node.lineno, node.col_offset + 1,
                           f"{node.name}: expected {expected} assertion(s), found {count}"))
    return errors


def source_paths():
    output = subprocess.check_output([
        "git", "ls-files", "--cached", "--others", "--exclude-standard", "-z", "--", "*.py"
    ])
    return sorted(set(Path(name) for name in output.decode().split("\0") if name))


def main():
    failures = []
    paths = source_paths()
    for path in paths:
        for line, column, message in diagnostics(path.read_text()):
            failures.append(f"{path}:{line}:{column}: {message}")
    for failure in failures:
        print(failure, file=sys.stderr)
    print(f"Checked {len(paths)} Python files; {len(failures)} policy violations")
    return bool(failures)


if __name__ == "__main__":
    sys.exit(main())
