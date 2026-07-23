#!/usr/bin/env python3
"""Validate the documentation-first planning baseline without external packages."""

from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOCS = ROOT / "docs"
REQUIRED_ROOT = ["README.md", "AGENTS.md", "CHANGELOG.md", "docs/INDEX.md", "docs/PROJECT_STATE.md"]
METADATA_FIELDS = {
    "status": ("Status:",),
    "last updated": ("Last updated:",),
    "requirements": ("Related requirements:", "Related requirement IDs:"),
    "ADRs": ("Related ADRs:", "Related architecture decision IDs:"),
    "open questions": ("Open questions:",),
    "dependencies": ("Dependencies:",),
    "supersession": ("Supersedes:", "Supersedes / superseded by:"),
}
LINK = re.compile(r"(?<!!)\[[^\]]+\]\(([^)]+)\)")


def main() -> int:
    errors: list[str] = []
    for relative in REQUIRED_ROOT:
        if not (ROOT / relative).is_file():
            errors.append(f"missing required file: {relative}")

    docs = sorted(DOCS.rglob("*.md"))
    for path in docs:
        text = path.read_text(encoding="utf-8")
        prefix = "\n".join(text.splitlines()[:20])
        for field, accepted_labels in METADATA_FIELDS.items():
            if not any(label in prefix for label in accepted_labels):
                errors.append(f"{path.relative_to(ROOT)}: metadata missing {field}")

        for target in LINK.findall(text):
            target = target.strip()
            if target.startswith(("http://", "https://", "mailto:", "#")):
                continue
            target_path = target.split("#", 1)[0]
            if not target_path:
                continue
            resolved = (path.parent / target_path).resolve()
            try:
                resolved.relative_to(ROOT.resolve())
            except ValueError:
                errors.append(f"{path.relative_to(ROOT)}: link escapes repository: {target}")
                continue
            if not resolved.exists():
                errors.append(f"{path.relative_to(ROOT)}: broken local link: {target}")

    agents = (ROOT / "AGENTS.md").read_text(encoding="utf-8") if (ROOT / "AGENTS.md").exists() else ""
    for rule_number in range(1, 29):
        if f"{rule_number}." not in agents:
            errors.append(f"AGENTS.md: mandatory rule {rule_number} not found")

    if errors:
        print("Planning validation failed:")
        print("\n".join(f"- {error}" for error in errors))
        return 1

    print(f"Planning validation passed: {len(docs)} documentation files checked.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
