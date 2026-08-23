#!/usr/bin/env python3
"""Reproduce the governed 3MF cube from the reviewed ASCII mesh source."""

from __future__ import annotations

import argparse
from io import BytesIO
from pathlib import Path
import zipfile

from generate_binary_stl_fixture import parse_ascii_triangles


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "fixtures" / "models" / "cube_10mm_ascii.stl"
OUTPUT = ROOT / "fixtures" / "models" / "cube_1cm_translated.3mf"
COMPONENT_OUTPUT = (
    ROOT / "fixtures" / "models" / "cube_1cm_component_scaled_translated.3mf"
)
FIXED_TIMESTAMP = (2000, 1, 1, 0, 0, 0)


def build_model_xml(source: bytes) -> bytes:
    """Convert the governed 10 mm STL cube to one 1 cm 3MF mesh object."""
    triangles = parse_ascii_triangles(source)
    vertices: list[tuple[float, float, float]] = []
    indices: dict[tuple[float, float, float], int] = {}
    indexed_triangles: list[tuple[int, int, int]] = []
    for triangle in triangles:
        triangle_indices = []
        for millimeter_vertex in triangle:
            centimeter_vertex = tuple(value / 10.0 for value in millimeter_vertex)
            if centimeter_vertex not in indices:
                indices[centimeter_vertex] = len(vertices)
                vertices.append(centimeter_vertex)
            triangle_indices.append(indices[centimeter_vertex])
        indexed_triangles.append(tuple(triangle_indices))

    vertex_xml = "\n".join(
        f'          <vertex x="{x:g}" y="{y:g}" z="{z:g}" />'
        for x, y, z in vertices
    )
    triangle_xml = "\n".join(
        f'          <triangle v1="{v1}" v2="{v2}" v3="{v3}" />'
        for v1, v2, v3 in indexed_triangles
    )
    return f'''<?xml version="1.0" encoding="UTF-8"?>
<model unit="centimeter" xml:lang="en-US" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
  <resources>
    <object id="1" type="model">
      <mesh>
        <vertices>
{vertex_xml}
        </vertices>
        <triangles>
{triangle_xml}
        </triangles>
      </mesh>
    </object>
  </resources>
  <build>
    <item objectid="1" transform="1 0 0 0 1 0 0 0 1 2 3 4" />
  </build>
</model>
'''.encode("utf-8")


def build_component_model_xml(source: bytes) -> bytes:
    """Wrap the governed mesh in one scaled/translated component object."""
    direct = build_model_xml(source).decode("utf-8")
    component_object = '''    <object id="2" type="model">
      <components>
        <component objectid="1" transform="2 0 0 0 1 0 0 0 1 1 2 3" />
      </components>
    </object>
'''
    model = direct.replace(
        "  </resources>\n  <build>",
        f"{component_object}  </resources>\n  <build>",
    ).replace(
        '<item objectid="1" transform="1 0 0 0 1 0 0 0 1 2 3 4" />',
        '<item objectid="2" transform="1 0 0 0 1 0 0 0 1 4 5 6" />',
    )
    return model.encode("utf-8")


def package_parts(
    source: bytes, *, component: bool = False
) -> tuple[tuple[str, bytes], ...]:
    """Return the exact ordered OPC parts used by the governed fixture."""
    content_types = b'''<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Override PartName="/3D/3dmodel.model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml" />
</Types>
'''
    relationships = b'''<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rel0" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel" Target="/3D/3dmodel.model" />
</Relationships>
'''
    return (
        ("[Content_Types].xml", content_types),
        ("_rels/.rels", relationships),
        (
            "3D/3dmodel.model",
            build_component_model_xml(source) if component else build_model_xml(source),
        ),
    )


def build_3mf(source: bytes) -> bytes:
    """Create deterministic Deflate-compressed 3MF bytes."""
    output = BytesIO()
    with zipfile.ZipFile(output, "w") as package:
        for name, contents in package_parts(source):
            info = zipfile.ZipInfo(name, FIXED_TIMESTAMP)
            info.compress_type = zipfile.ZIP_DEFLATED
            info.create_system = 3
            info.external_attr = 0o100644 << 16
            package.writestr(info, contents, compress_type=zipfile.ZIP_DEFLATED, compresslevel=9)
    return output.getvalue()


def build_component_3mf(source: bytes) -> bytes:
    """Create deterministic bytes for the governed component-transform fixture."""
    output = BytesIO()
    with zipfile.ZipFile(output, "w") as package:
        for name, contents in package_parts(source, component=True):
            info = zipfile.ZipInfo(name, FIXED_TIMESTAMP)
            info.compress_type = zipfile.ZIP_DEFLATED
            info.create_system = 3
            info.external_attr = 0o100644 << 16
            package.writestr(
                info,
                contents,
                compress_type=zipfile.ZIP_DEFLATED,
                compresslevel=9,
            )
    return output.getvalue()


def check_fixture(source_path: Path = SOURCE, output_path: Path = OUTPUT) -> None:
    """Fail when the committed 3MF fixture differs from deterministic output."""
    expected = build_3mf(source_path.read_bytes())
    if not output_path.is_file() or output_path.read_bytes() != expected:
        raise RuntimeError("3MF fixture is missing or not reproducible")


def check_component_fixture(
    source_path: Path = SOURCE, output_path: Path = COMPONENT_OUTPUT
) -> None:
    """Fail when the committed component fixture differs from deterministic output."""
    expected = build_component_3mf(source_path.read_bytes())
    if not output_path.is_file() or output_path.read_bytes() != expected:
        raise RuntimeError("component 3MF fixture is missing or not reproducible")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--write",
        action="store_true",
        help="replace only the governed 3MF output with deterministic bytes",
    )
    args = parser.parse_args()
    expected = build_3mf(SOURCE.read_bytes())
    component_expected = build_component_3mf(SOURCE.read_bytes())
    if args.write:
        OUTPUT.write_bytes(expected)
        COMPONENT_OUTPUT.write_bytes(component_expected)
    else:
        check_fixture()
        check_component_fixture()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
