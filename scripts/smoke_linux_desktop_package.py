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
PORTAL_ACCEPT_LABEL = "Select"
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


def canonical_portal_directory_text(directory: str) -> str:
    if not directory.startswith("/"):
        raise ValueError("portal directory must be absolute")
    if directory == "/":
        return directory
    return f"{directory.rstrip('/')}/"


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


def activate_click_action(node: Any, label: str) -> None:
    try:
        actions = node.queryAction()
        action_count = int(actions.nActions)
    except Exception as error:
        raise RuntimeError(f"accessible control has no action interface: {label!r}") from error

    available = []
    for index in range(action_count):
        try:
            action_name = str(actions.getName(index))
        except Exception:
            continue
        available.append(action_name)
        if action_name.casefold() == "click":
            try:
                accepted = bool(actions.doAction(index))
            except Exception as error:
                raise RuntimeError(f"accessible click failed for {label!r}") from error
            if not accepted:
                raise RuntimeError(f"accessible click was rejected for {label!r}")
            return
    raise RuntimeError(
        f"accessible control has no exact click action for {label!r}: {available!r}"
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


def activate_accept_button(
    accept_button: Any,
    accept_label: str,
    focused_state: Any,
) -> None:
    focus_window(PORTAL_WINDOW_TITLE)
    focus(
        accept_button,
        f"{accept_label} selected STEP model",
        focused_state=focused_state,
    )
    key("Return")


def click_accept_button(
    accept_button: Any,
    accept_label: str,
    focused_state: Any,
) -> None:
    focus_window(PORTAL_WINDOW_TITLE)
    focus(
        accept_button,
        f"{accept_label} selected STEP model",
        focused_state=focused_state,
    )
    activate_click_action(accept_button, f"{accept_label} selected STEP model")


def activate_selected_file(fixture_row: Any, focused_state: Any) -> None:
    focus_window(PORTAL_WINDOW_TITLE)
    focus(
        fixture_row,
        "Selected STEP model",
        focused_state=focused_state,
    )
    key("Return")


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


def picker_open_observed(pyatspi: Any) -> bool:
    return (
        find_node(
            pyatspi,
            "Picker open",
            {"push button", "button"},
            exact=True,
        )
        is not None
    )


def selection_acceptance_observed(pyatspi: Any, expected_source_label: str) -> bool:
    source_summary = (
        find_node(
            pyatspi,
            expected_source_label,
            exact=True,
            required_states=(pyatspi.STATE_SHOWING,),
        )
        is not None
    )
    return source_summary and not picker_open_observed(pyatspi)


def wait_for_selection_acceptance(
    pyatspi: Any,
    expected_source_label: str,
    timeout_seconds: float,
) -> bool:
    deadline = time.monotonic() + timeout_seconds
    while time.monotonic() < deadline:
        if selection_acceptance_observed(pyatspi, expected_source_label):
            return True
        time.sleep(0.05)
    return selection_acceptance_observed(pyatspi, expected_source_label)


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
        wait_for_text_value(location_entry, "/", 2.0)
        fixture_directory = str(fixture.parent)
        expected_directory_text = canonical_portal_directory_text(fixture_directory)
        type_text(fixture_directory.removeprefix("/"))
        wait_for_text_value(location_entry, expected_directory_text, 2.0)
        print("Confirmed exact canonical portal location entry text", flush=True)
        key("Return")
        fixture_row = wait_for_unique_node(
            pyatspi,
            fixture.name,
            deadline,
            exact=True,
            required_states=(pyatspi.STATE_SHOWING,),
        )
        select_node(fixture_row, fixture.name)
        print(f"Selected exact fixture row: {fixture.name}", flush=True)
        accept_label = PORTAL_ACCEPT_LABEL
        accept_button = wait_for_unique_node(
            pyatspi,
            accept_label,
            deadline,
            {"push button", "button"},
            exact=True,
            required_states=(
                pyatspi.STATE_SHOWING,
                pyatspi.STATE_ENABLED,
                pyatspi.STATE_FOCUSABLE,
            ),
        )
        activate_accept_button(
            accept_button,
            accept_label,
            pyatspi.STATE_FOCUSED,
        )
        print("Activated exact portal Select button with Return", flush=True)

        selection_accepted = wait_for_selection_acceptance(
            pyatspi,
            expected_source_label,
            2.0,
        )
        if not selection_accepted and picker_open_observed(pyatspi):
            if node_has_states(
                fixture_row,
                (pyatspi.STATE_SHOWING, pyatspi.STATE_FOCUSABLE),
            ):
                activate_selected_file(fixture_row, pyatspi.STATE_FOCUSED)
                print("Activated selected fixture with Return", flush=True)
            else:
                print(
                    "Selected fixture row is not focusable; skipping Return activation",
                    flush=True,
                )

        selection_accepted = wait_for_selection_acceptance(
            pyatspi,
            expected_source_label,
            2.0,
        )
        if not selection_accepted and picker_open_observed(pyatspi):
            accept_button = wait_for_unique_node(
                pyatspi,
                accept_label,
                deadline,
                {"push button", "button"},
                exact=True,
                required_states=(
                    pyatspi.STATE_SHOWING,
                    pyatspi.STATE_ENABLED,
                    pyatspi.STATE_FOCUSABLE,
                ),
            )
            click_accept_button(
                accept_button,
                accept_label,
                pyatspi.STATE_FOCUSED,
            )
            print("Invoked exact portal Select button click action", flush=True)

        wait_for_unique_node(
            pyatspi,
            expected_source_label,
            deadline,
            exact=True,
            required_states=(pyatspi.STATE_SHOWING,),
        )
        wait_for_node_absent(
            pyatspi,
            "Picker open",
            deadline,
            {"push button", "button"},
            exact=True,
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
