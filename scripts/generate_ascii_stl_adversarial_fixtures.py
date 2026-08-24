#!/usr/bin/env python3
"""Reproduce the governed malformed ASCII STL rejection corpus."""

from __future__ import annotations

import argparse
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
ADVERSARIAL_FIXTURES: tuple[tuple[str, Path], ...] = (
    (
        "invalid_utf8",
        ROOT / "fixtures" / "models" / "adversarial_ascii_stl_invalid_utf8.stl",
    ),
    (
        "malformed_facet",
        ROOT / "fixtures" / "models" / "adversarial_ascii_stl_malformed_facet.stl",
    ),
    (
        "empty_solid",
        ROOT / "fixtures" / "models" / "adversarial_ascii_stl_empty_solid.stl",
    ),
    (
        "degenerate_triangle",
        ROOT
        / "fixtures"
        / "models"
        / "adversarial_ascii_stl_degenerate_triangle.stl",
    ),
)


def build_adversarial_ascii_stl(case: str) -> bytes:
    """Return one deterministic malformed ASCII STL byte stream."""
    if case == "invalid_utf8":
        return b"solid invalid-utf8\n\xff\nendsolid invalid-utf8\n"
    if case == "malformed_facet":
        return b"""solid malformed-facet
facet normal 0 0 1
outer loop
vertex 0 0 0
vertex 1 0 0
endloop
endfacet
endsolid malformed-facet
"""
    if case == "empty_solid":
        return b"solid empty\nendsolid empty\n"
    if case == "degenerate_triangle":
        return b"""solid degenerate
facet normal 0 0 1
outer loop
vertex 0 0 0
vertex 1 0 0
vertex 2 0 0
endloop
endfacet
endsolid degenerate
"""
    raise ValueError(f"unsupported adversarial ASCII STL case: {case}")


def check_fixtures(
    fixtures: tuple[tuple[str, Path], ...] = ADVERSARIAL_FIXTURES,
) -> None:
    """Fail when any malformed ASCII STL fixture is not reproducible."""
    for case, output_path in fixtures:
        expected = build_adversarial_ascii_stl(case)
        if not output_path.is_file() or output_path.read_bytes() != expected:
            raise RuntimeError(
                "adversarial ASCII STL fixture is missing or not reproducible"
            )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--write",
        action="store_true",
        help="replace only the governed malformed ASCII STL outputs",
    )
    args = parser.parse_args()
    if args.write:
        for case, output_path in ADVERSARIAL_FIXTURES:
            output_path.write_bytes(build_adversarial_ascii_stl(case))
    else:
        check_fixtures()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
