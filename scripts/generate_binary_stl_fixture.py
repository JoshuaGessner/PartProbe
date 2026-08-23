#!/usr/bin/env python3
"""Reproduce the governed binary STL cube from its reviewed ASCII source."""

from __future__ import annotations

import argparse
from pathlib import Path
import struct


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "fixtures" / "models" / "cube_10mm_ascii.stl"
OUTPUT = ROOT / "fixtures" / "models" / "cube_10mm_binary.stl"
HEADER = b"PartProbe FIX-MESH-003 derived from governed FIX-MESH-001"


def parse_ascii_triangles(source: bytes) -> list[tuple[tuple[float, float, float], ...]]:
    """Extract exactly three vertices per facet from the governed ASCII fixture."""
    text = source.decode("ascii")
    triangles: list[tuple[tuple[float, float, float], ...]] = []
    current: list[tuple[float, float, float]] = []
    for raw_line in text.splitlines():
        fields = raw_line.split()
        if fields[:1] == ["vertex"]:
            if len(fields) != 4:
                raise ValueError("fixture vertex must have exactly three coordinates")
            current.append(tuple(float(value) for value in fields[1:]))
        elif fields[:1] == ["endfacet"]:
            if len(current) != 3:
                raise ValueError("fixture facet must have exactly three vertices")
            triangles.append(tuple(current))
            current = []
    if current or not triangles:
        raise ValueError("fixture must contain complete nonempty facets")
    return triangles


def facet_normal(
    triangle: tuple[tuple[float, float, float], ...],
) -> tuple[float, float, float]:
    """Return the unit normal established by the fixture's vertex winding."""
    a, b, c = triangle
    ab = tuple(b[index] - a[index] for index in range(3))
    ac = tuple(c[index] - a[index] for index in range(3))
    cross = (
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    )
    magnitude = sum(component * component for component in cross) ** 0.5
    if magnitude == 0.0:
        raise ValueError("fixture triangle must not be degenerate")
    return tuple(component / magnitude for component in cross)


def build_binary_stl(source: bytes) -> bytes:
    """Encode the governed triangles using fixed little-endian binary STL records."""
    triangles = parse_ascii_triangles(source)
    if len(HEADER) > 80:
        raise ValueError("binary STL header exceeds 80 bytes")
    output = bytearray(HEADER.ljust(80, b"\0"))
    output.extend(struct.pack("<I", len(triangles)))
    for triangle in triangles:
        coordinates = tuple(value for vertex in triangle for value in vertex)
        output.extend(struct.pack("<12fH", *facet_normal(triangle), *coordinates, 0))
    return bytes(output)


def check_fixture(source_path: Path = SOURCE, output_path: Path = OUTPUT) -> None:
    """Fail when the committed binary fixture differs from deterministic output."""
    expected = build_binary_stl(source_path.read_bytes())
    if not output_path.is_file() or output_path.read_bytes() != expected:
        raise RuntimeError("binary STL fixture is missing or not reproducible")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--write",
        action="store_true",
        help="replace only the governed binary STL output with deterministic bytes",
    )
    args = parser.parse_args()
    expected = build_binary_stl(SOURCE.read_bytes())
    if args.write:
        OUTPUT.write_bytes(expected)
    else:
        check_fixture()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
