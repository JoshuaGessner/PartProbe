#!/usr/bin/env python3
"""Reproduce governed ASCII STL topology-warning fixtures."""

from __future__ import annotations

import argparse
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "fixtures" / "models" / "cube_10mm_ascii.stl"
TOPOLOGY_FIXTURES: tuple[tuple[str, Path], ...] = (
    (
        "reversed_facet",
        ROOT / "fixtures" / "models" / "cube_10mm_ascii_reversed_facet.stl",
    ),
    (
        "non_manifold_edge",
        ROOT / "fixtures" / "models" / "two_tetrahedra_shared_edge_ascii.stl",
    ),
    (
        "coplanar_overlap",
        ROOT / "fixtures" / "models" / "coplanar_overlap_ascii.stl",
    ),
)

FIRST_FACET = b"""  facet normal 0 0 -1
    outer loop
      vertex 0 0 0
      vertex 10 10 0
      vertex 10 0 0
    endloop
  endfacet
"""
REVERSED_FIRST_FACET = b"""  facet normal 0 0 1
    outer loop
      vertex 0 0 0
      vertex 10 0 0
      vertex 10 10 0
    endloop
  endfacet
"""

NON_MANIFOLD_SHARED_EDGE = b"""solid two_tetrahedra_shared_edge
  facet normal 0 0 -1
    outer loop
      vertex 0 0 0
      vertex 0 2 0
      vertex 2 0 0
    endloop
  endfacet
  facet normal 0 -1 0
    outer loop
      vertex 0 0 0
      vertex 2 0 0
      vertex 0 0 2
    endloop
  endfacet
  facet normal -1 0 0
    outer loop
      vertex 0 0 0
      vertex 0 0 2
      vertex 0 2 0
    endloop
  endfacet
  facet normal 1 1 1
    outer loop
      vertex 2 0 0
      vertex 0 2 0
      vertex 0 0 2
    endloop
  endfacet
  facet normal 0 -2 4
    outer loop
      vertex 0 0 0
      vertex 1 -2 -1
      vertex 2 0 0
    endloop
  endfacet
  facet normal 0 4 -2
    outer loop
      vertex 0 0 0
      vertex 2 0 0
      vertex 1 -1 -2
    endloop
  endfacet
  facet normal -3 -1 -1
    outer loop
      vertex 0 0 0
      vertex 1 -1 -2
      vertex 1 -2 -1
    endloop
  endfacet
  facet normal 3 -1 -1
    outer loop
      vertex 2 0 0
      vertex 1 -2 -1
      vertex 1 -1 -2
    endloop
  endfacet
endsolid two_tetrahedra_shared_edge
"""

COPLANAR_OVERLAP = b"""solid coplanar_overlap
  facet normal 0 0 1
    outer loop
      vertex 0 0 0
      vertex 2 0 0
      vertex 0 2 0
    endloop
  endfacet
  facet normal 0 0 1
    outer loop
      vertex 0.5 0.5 0
      vertex 2.5 0.5 0
      vertex 0.5 2.5 0
    endloop
  endfacet
endsolid coplanar_overlap
"""


def build_ascii_stl_topology_fixture(source: bytes, case: str) -> bytes:
    """Return one deterministic topology-warning ASCII STL byte stream."""
    if case == "reversed_facet":
        if source.count(FIRST_FACET) != 1:
            raise RuntimeError("governed source must contain the first facet exactly once")
        return source.replace(FIRST_FACET, REVERSED_FIRST_FACET, 1)
    if case == "non_manifold_edge":
        return NON_MANIFOLD_SHARED_EDGE
    if case == "coplanar_overlap":
        return COPLANAR_OVERLAP
    raise ValueError(f"unsupported ASCII STL topology case: {case}")


def check_fixtures(
    source_path: Path = SOURCE,
    fixtures: tuple[tuple[str, Path], ...] = TOPOLOGY_FIXTURES,
) -> None:
    """Fail when any topology-warning ASCII STL fixture is not reproducible."""
    source = source_path.read_bytes()
    for case, output_path in fixtures:
        expected = build_ascii_stl_topology_fixture(source, case)
        if not output_path.is_file() or output_path.read_bytes() != expected:
            raise RuntimeError(
                "ASCII STL topology fixture is missing or not reproducible"
            )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--write",
        action="store_true",
        help="replace only the governed ASCII STL topology-warning outputs",
    )
    args = parser.parse_args()
    if args.write:
        source = SOURCE.read_bytes()
        for case, output_path in TOPOLOGY_FIXTURES:
            output_path.write_bytes(build_ascii_stl_topology_fixture(source, case))
    else:
        check_fixtures()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
