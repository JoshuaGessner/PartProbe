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


PORTAL_CHOOSER_NAME = "File Chooser Widget"
PORTAL_FILES_LABEL = "Files"
PORTAL_WINDOW_TITLE = "Select STEP model"
SELECTED_SOURCE_LABEL = "Selected model source"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--executable", required=True, type=Path)
    parser.add_argument("--fixture", required=True, type=Path)
    parser.add_argument("--timeout-seconds", type=float, default=60.0)
    parser.add_argument("--selection-only", action="store_true")
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
    pending = [pyatspi.Registry.getDesktop(0)]
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
    matches = matching_nodes(
        pyatspi,
        text,
        roles,
        exact=exact,
        required_states=required_states,
    )
    return matches[0] if matches else None


def matching_nodes(
    pyatspi: Any,
    text: str,
    roles: set[str] | None = None,
    *,
    exact: bool = False,
    required_states: tuple[Any, ...] = (),
) -> list[Any]:
    expected = text.casefold()
    matches = []
    for node in accessibility_nodes(pyatspi):
        if roles is not None and role_name(node) not in roles:
            continue
        candidate = node_text(node).casefold()
        text_matches = candidate == expected if exact else expected in candidate
        if text_matches and node_has_states(node, required_states):
            matches.append(node)
    return matches


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


def wait_for_unique_node(
    pyatspi: Any,
    text: str,
    deadline: float,
    roles: set[str] | None = None,
    *,
    exact: bool = False,
    required_states: tuple[Any, ...] = (),
) -> Any:
    while time.monotonic() < deadline:
        matches = matching_nodes(
            pyatspi,
            text,
            roles,
            exact=exact,
            required_states=required_states,
        )
        if len(matches) == 1:
            return matches[0]
        if len(matches) > 1:
            raise RuntimeError(
                f"ambiguous accessible target {text!r}: {len(matches)} live matches"
            )
        time.sleep(0.25)
    raise RuntimeError(f"timed out waiting for unique accessible text: {text!r}")


def wait_for_unique_role_node(
    pyatspi: Any,
    roles: set[str],
    deadline: float,
    *,
    label: str,
    required_states: tuple[Any, ...] = (),
) -> Any:
    while time.monotonic() < deadline:
        matches = [
            node
            for node in accessibility_nodes(pyatspi)
            if role_name(node) in roles and node_has_states(node, required_states)
        ]
        if len(matches) == 1:
            return matches[0]
        if len(matches) > 1:
            raise RuntimeError(
                f"ambiguous accessible role target {label!r}: {len(matches)} live matches"
            )
        time.sleep(0.05)
    raise RuntimeError(f"timed out waiting for unique accessible role: {label!r}")


def accessible_text_value(node: Any) -> str:
    try:
        text = node.queryText()
        return str(text.getText(0, int(text.characterCount)))
    except Exception as error:
        raise RuntimeError("accessible node has no text interface") from error


def wait_for_text_value(node: Any, expected: str, timeout_seconds: float) -> None:
    deadline = time.monotonic() + timeout_seconds
    while time.monotonic() < deadline:
        if accessible_text_value(node) == expected:
            return
        time.sleep(0.05)
    actual = accessible_text_value(node)
    raise RuntimeError(
        f"accessible location entry did not contain the expected text: {actual!r}"
    )


def portal_source_entry_text(source: Path) -> str:
    if not source.is_absolute():
        raise ValueError("portal source must be absolute")
    return str(source)


def submit_source_from_location_entry(
    location_entry: Any,
    source_text: str,
    focused_state: Any,
) -> None:
    wait_for_text_value(location_entry, "/", 2.0)
    type_text(source_text.removeprefix("/"))
    wait_for_text_value(location_entry, source_text, 2.0)
    if not node_has_states(location_entry, (focused_state,)):
        raise RuntimeError("portal location entry lost focus before source submission")
    key("Return")


def selected_source_accessible_label(display_name: str) -> str:
    return f"{SELECTED_SOURCE_LABEL}: {display_name}"


def wait_for_node_absent(
    pyatspi: Any,
    text: str,
    deadline: float,
    roles: set[str] | None = None,
    *,
    exact: bool = False,
) -> None:
    while time.monotonic() < deadline:
        if find_node(pyatspi, text, roles, exact=exact) is None:
            return
        time.sleep(0.05)
    raise RuntimeError(f"timed out waiting for accessible text to clear: {text!r}")


def focus(
    node: Any,
    label: str,
    *,
    focused_state: Any | None = None,
) -> None:
    try:
        focused = bool(node.queryComponent().grabFocus())
    except Exception as error:
        raise RuntimeError(f"could not focus {label!r}") from error
    if not focused:
        raise RuntimeError(f"accessibility focus was rejected for {label!r}")
    if focused_state is None:
        return
    focus_deadline = time.monotonic() + 2.0
    while time.monotonic() < focus_deadline:
        if node_has_states(node, (focused_state,)):
            return
        time.sleep(0.05)
    raise RuntimeError(f"accessible control did not acquire focus for {label!r}")


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


def open_location_entry(
    focus_anchor: Any,
    focus_label: str,
    focused_state: Any,
) -> None:
    focus_window(PORTAL_WINDOW_TITLE)
    focus(
        focus_anchor,
        focus_label,
        focused_state=focused_state,
    )
    key("slash")


def wait_for_expected_source_label(
    pyatspi: Any,
    expected_source_label: str,
    deadline: float,
    showing_state: Any,
) -> Any:
    while time.monotonic() < deadline:
        exact_matches = matching_nodes(
            pyatspi,
            expected_source_label,
            exact=True,
            required_states=(showing_state,),
        )
        if len(exact_matches) == 1:
            return exact_matches[0]
        if len(exact_matches) > 1:
            raise RuntimeError(
                f"ambiguous accepted source label: {expected_source_label!r}"
            )
        any_source = matching_nodes(
            pyatspi,
            f"{SELECTED_SOURCE_LABEL}:",
            required_states=(showing_state,),
        )
        if any_source:
            raise RuntimeError(
                "native picker accepted a source other than the governed fixture"
            )
        time.sleep(0.05)
    raise RuntimeError(
        f"timed out waiting for exact accepted source label: {expected_source_label!r}"
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
    expected_source_text = portal_source_entry_text(fixture)
    expected_source_label = selected_source_accessible_label(fixture.name)

    try:
        import pyatspi  # type: ignore[import-not-found]
    except ImportError as error:
        raise RuntimeError("the AT-SPI Python bindings are required") from error

    deadline = time.monotonic() + args.timeout_seconds
    process = subprocess.Popen([str(executable)])
    try:
        choose_button = wait_for_unique_node(
            pyatspi,
            "Choose STEP model",
            deadline,
            {"push button", "button"},
            exact=True,
            required_states=(
                pyatspi.STATE_SHOWING,
                pyatspi.STATE_ENABLED,
                pyatspi.STATE_FOCUSABLE,
            ),
        )
        focus_window("PartProbe")
        focus(
            choose_button,
            "Choose STEP model",
            focused_state=pyatspi.STATE_FOCUSED,
        )
        key("Return")

        wait_for_unique_node(
            pyatspi,
            PORTAL_CHOOSER_NAME,
            deadline,
            {"file chooser"},
            exact=True,
            required_states=(pyatspi.STATE_SHOWING,),
        )
        print("Found exact showing XDG portal file chooser", flush=True)
        location_focus_anchor = wait_for_unique_node(
            pyatspi,
            PORTAL_FILES_LABEL,
            deadline,
            {"table"},
            exact=True,
            required_states=(
                pyatspi.STATE_SHOWING,
                pyatspi.STATE_ENABLED,
                pyatspi.STATE_FOCUSABLE,
            ),
        )
        open_location_entry(
            location_focus_anchor,
            "Portal file list",
            pyatspi.STATE_FOCUSED,
        )
        location_entry = wait_for_unique_role_node(
            pyatspi,
            {"text", "entry"},
            deadline,
            label="focused portal location entry",
            required_states=(
                pyatspi.STATE_SHOWING,
                pyatspi.STATE_ENABLED,
                pyatspi.STATE_FOCUSABLE,
                pyatspi.STATE_FOCUSED,
            ),
        )
        submit_source_from_location_entry(
            location_entry,
            expected_source_text,
            pyatspi.STATE_FOCUSED,
        )
        print("Submitted exact portal source entry text", flush=True)
        wait_for_expected_source_label(
            pyatspi,
            expected_source_label,
            deadline,
            pyatspi.STATE_SHOWING,
        )
        wait_for_node_absent(
            pyatspi,
            "Picker open",
            deadline,
            {"push button", "button"},
            exact=True,
        )
        wait_for_unique_node(
            pyatspi,
            "Analyze provisional geometry",
            deadline,
            {"push button", "button"},
            exact=True,
            required_states=(
                pyatspi.STATE_SHOWING,
                pyatspi.STATE_ENABLED,
                pyatspi.STATE_FOCUSABLE,
            ),
        )
        if args.selection_only:
            print("Linux packaged desktop native selection smoke passed")
            return 0
        analyze_button = wait_for_unique_node(
            pyatspi,
            "Analyze provisional geometry",
            deadline,
            {"push button", "button"},
            exact=True,
            required_states=(
                pyatspi.STATE_SHOWING,
                pyatspi.STATE_ENABLED,
                pyatspi.STATE_FOCUSABLE,
            ),
        )
        focus_window("PartProbe")
        focus(
            analyze_button,
            "Analyze provisional geometry",
            focused_state=pyatspi.STATE_FOCUSED,
        )
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
