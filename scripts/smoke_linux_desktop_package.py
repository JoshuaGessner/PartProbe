#!/usr/bin/env python3
"""Exercise the extracted Linux desktop package through AT-SPI and keyboard input."""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import re
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


def find_node(
    pyatspi: Any,
    text: str,
    roles: set[str] | None = None,
    *,
    exact: bool = False,
    required_states: tuple[Any, ...] = (),
) -> Any | None:
    expected = text.casefold()
    for node in accessibility_nodes(pyatspi):
        if roles is not None and role_name(node) not in roles:
            continue
        candidate = node_text(node).casefold()
        matches = candidate == expected if exact else expected in candidate
        if matches and node_has_states(node, required_states):
            return node
    return None


def node_has_states(node: Any, required_states: tuple[Any, ...]) -> bool:
    if not required_states:
        return True
    try:
        states = node.getState()
        return all(bool(states.contains(state)) for state in required_states)
    except Exception:
        return False


def wait_for_node(
    pyatspi: Any,
    text: str,
    deadline: float,
    roles: set[str] | None = None,
    *,
    exact: bool = False,
    required_states: tuple[Any, ...] = (),
) -> Any:
    while time.monotonic() < deadline:
        node = find_node(
            pyatspi,
            text,
            roles,
            exact=exact,
            required_states=required_states,
        )
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


def select_node(node: Any, label: str) -> None:
    current = node
    for _ in range(16):
        try:
            parent = current.parent
        except Exception:
            parent = None
        if parent is None:
            break
        try:
            child_index = int(current.getIndexInParent())
            selection = parent.querySelection()
            if bool(selection.selectChild(child_index)):
                selection_deadline = time.monotonic() + 2.0
                while time.monotonic() < selection_deadline:
                    if bool(selection.isChildSelected(child_index)):
                        return
                    time.sleep(0.05)
        except Exception:
            pass
        current = parent
    raise RuntimeError(f"could not select accessible item {label!r}")


def activate_button(node: Any, label: str) -> None:
    try:
        actions = node.queryAction()
        action_names = [
            str(actions.getName(index)).casefold()
            for index in range(int(actions.nActions))
        ]
    except Exception as error:
        raise RuntimeError(
            f"could not inspect accessibility actions for {label!r}"
        ) from error
    for preferred_name in ("click", "press", "activate"):
        for index, name in enumerate(action_names):
            if name == preferred_name:
                if not bool(actions.doAction(index)):
                    raise RuntimeError(
                        f"accessibility activation was rejected for {label!r}"
                    )
                return
    raise RuntimeError(
        f"accessible button {label!r} has no activation action: {action_names!r}"
    )


def focus_window(title: str) -> None:
    subprocess.run(
        [
            "xdotool",
            "search",
            "--sync",
            "--onlyvisible",
            "--name",
            f"^{re.escape(title)}$",
            "windowfocus",
            "--sync",
        ],
        check=True,
        timeout=10,
    )


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


def accept_open_dialog(open_button: Any) -> None:
    focus_window("Open File")
    activate_button(open_button, "Open selected STEP model")


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
        focus_window("PartProbe")
        focus(choose_button, "Choose STEP model")
        key("Return")

        wait_for_node(pyatspi, "Open File", deadline)
        focus_window("Open File")
        key("ctrl+l")
        type_text(str(fixture.parent))
        key("Return")
        fixture_row = wait_for_node(
            pyatspi,
            fixture.name,
            deadline,
            exact=True,
            required_states=(pyatspi.STATE_SHOWING,),
        )
        select_node(fixture_row, fixture.name)
        open_button = wait_for_node(
            pyatspi,
            "Open",
            deadline,
            {"push button", "button"},
            exact=True,
            required_states=(pyatspi.STATE_SHOWING, pyatspi.STATE_ENABLED),
        )
        accept_open_dialog(open_button)

        wait_for_node(pyatspi, "Model selected", deadline)
        wait_for_node(pyatspi, fixture.name, deadline)
        analyze_button = wait_for_node(
            pyatspi,
            "Analyze provisional geometry",
            deadline,
            {"push button", "button"},
        )
        focus_window("PartProbe")
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
