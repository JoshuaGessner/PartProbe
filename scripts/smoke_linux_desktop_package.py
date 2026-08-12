#!/usr/bin/env python3
"""Exercise the extracted Linux desktop package through AT-SPI and keyboard input."""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import subprocess
import sys
import time
from typing import Any, Iterable


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--executable", required=True, type=Path)
    parser.add_argument("--fixture", required=True, type=Path)
    parser.add_argument("--timeout-seconds", type=float, default=60.0)
    return parser.parse_args()


def accessible_children(node: Any) -> Iterable[Any]:
    try:
        count = int(node.childCount)
    except Exception:
        return ()
    children = []
    for index in range(min(count, 512)):
        try:
            child = node.getChildAtIndex(index)
        except Exception:
            continue
        if child is not None:
            children.append(child)
    return children


def accessibility_nodes(pyatspi: Any) -> Iterable[Any]:
    desktop = pyatspi.Registry.getDesktop(0)
    pending = [desktop]
    visited = 0
    while pending and visited < 10_000:
        node = pending.pop()
        visited += 1
        yield node
        pending.extend(accessible_children(node))


def node_text(node: Any) -> str:
    values = []
    for attribute in ("name", "description"):
        try:
            value = getattr(node, attribute)
        except Exception:
            continue
        if value:
            values.append(str(value))
    return " ".join(values)


def role_name(node: Any) -> str:
    try:
        return str(node.getRoleName()).lower()
    except Exception:
        return ""


def find_node(pyatspi: Any, text: str, roles: set[str] | None = None) -> Any | None:
    expected = text.casefold()
    for node in accessibility_nodes(pyatspi):
        if roles is not None and role_name(node) not in roles:
            continue
        if expected in node_text(node).casefold():
            return node
    return None


def wait_for_node(
    pyatspi: Any,
    text: str,
    deadline: float,
    roles: set[str] | None = None,
) -> Any:
    while time.monotonic() < deadline:
        node = find_node(pyatspi, text, roles)
        if node is not None:
            return node
        time.sleep(0.25)
    raise RuntimeError(f"timed out waiting for accessible text: {text!r}")


def focus(node: Any, label: str) -> None:
    try:
        focused = bool(node.queryComponent().grabFocus())
    except Exception as error:
        raise RuntimeError(f"could not focus {label!r}") from error
    if not focused:
        raise RuntimeError(f"accessibility focus was rejected for {label!r}")


def key(*keys: str) -> None:
    subprocess.run(
        ["xdotool", "key", "--clearmodifiers", *keys],
        check=True,
        timeout=10,
    )


def type_text(value: str) -> None:
    subprocess.run(
        ["xdotool", "type", "--clearmodifiers", "--delay", "1", value],
        check=True,
        timeout=10,
    )


def dump_accessibility(pyatspi: Any) -> None:
    rows = []
    for node in accessibility_nodes(pyatspi):
        text = node_text(node).strip()
        if text:
            rows.append(f"{role_name(node)}: {text}")
        if len(rows) >= 120:
            break
    if rows:
        print("Accessible state at failure:", file=sys.stderr)
        print("\n".join(rows), file=sys.stderr)


def main() -> int:
    args = parse_args()
    executable = args.executable.resolve(strict=True)
    fixture = args.fixture.resolve(strict=True)
    if not executable.is_file() or not os.access(executable, os.X_OK):
        raise RuntimeError("packaged executable is not executable")
    if fixture.suffix.lower() not in {".step", ".stp"}:
        raise RuntimeError("interactive smoke fixture must be a STEP file")

    try:
        import pyatspi  # type: ignore[import-not-found]
    except ImportError as error:
        raise RuntimeError("the AT-SPI Python bindings are required") from error

    deadline = time.monotonic() + args.timeout_seconds
    process = subprocess.Popen([str(executable)])
    try:
        choose_button = wait_for_node(
            pyatspi,
            "Choose STEP model",
            deadline,
            {"push button", "button"},
        )
        focus(choose_button, "Choose STEP model")
        key("Return")

        wait_for_node(pyatspi, "Open File", deadline)
        key("ctrl+l")
        type_text(str(fixture))
        key("Return")

        wait_for_node(pyatspi, fixture.name, deadline)
        analyze_button = wait_for_node(
            pyatspi,
            "Analyze provisional geometry",
            deadline,
            {"push button", "button"},
        )
        focus(analyze_button, "Analyze provisional geometry")
        key("Return")

        for expected in (
            "Geometry evidence",
            "392",
            "480",
            "6, 4, 2.5",
            "OCCT 8.0.0",
        ):
            wait_for_node(pyatspi, expected, deadline)
        if find_node(pyatspi, "Analysis failed safely") is not None:
            raise RuntimeError("the interactive analysis reported a safe failure")
        if process.poll() is not None:
            raise RuntimeError(
                f"packaged executable exited during interactive smoke: {process.returncode}"
            )
        print("Linux packaged desktop accessibility smoke passed")
        return 0
    except Exception:
        dump_accessibility(pyatspi)
        raise
    finally:
        if process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as error:
        print(f"Linux packaged desktop accessibility smoke failed: {error}", file=sys.stderr)
        raise SystemExit(1) from error
