#!/usr/bin/env python3
"""Reproduce the governed 3MF cube from the reviewed ASCII mesh source."""

from __future__ import annotations

import argparse
from decimal import Decimal, localcontext
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
METADATA_OUTPUT = ROOT / "fixtures" / "models" / "cube_10mm_3mf_metadata.3mf"
UNIT_FIXTURES: tuple[tuple[str | None, str, Path], ...] = (
    ("micron", "0.001", ROOT / "fixtures" / "models" / "cube_10mm_3mf_micron.3mf"),
    (
        "millimeter",
        "1",
        ROOT / "fixtures" / "models" / "cube_10mm_3mf_millimeter.3mf",
    ),
    ("meter", "1000", ROOT / "fixtures" / "models" / "cube_10mm_3mf_meter.3mf"),
    ("inch", "25.4", ROOT / "fixtures" / "models" / "cube_10mm_3mf_inch.3mf"),
    ("foot", "304.8", ROOT / "fixtures" / "models" / "cube_10mm_3mf_foot.3mf"),
    (None, "1", ROOT / "fixtures" / "models" / "cube_10mm_3mf_default_mm.3mf"),
)
FIXED_TIMESTAMP = (2000, 1, 1, 0, 0, 0)


def _format_source_coordinate(millimeters: float, millimeters_per_unit: str) -> str:
    with localcontext() as context:
        context.prec = 28
        value = Decimal(str(millimeters)) / Decimal(millimeters_per_unit)
    if value == 0:
        return "0"
    return format(value.normalize(), "f")


def build_unit_model_xml(
    source: bytes,
    unit: str | None,
    millimeters_per_unit: str,
    *,
    build_transform: str | None = None,
) -> bytes:
    """Convert the governed 10 mm cube into one unit-specific 3MF model."""
    triangles = parse_ascii_triangles(source)
    vertices: list[tuple[str, str, str]] = []
    indices: dict[tuple[str, str, str], int] = {}
    indexed_triangles: list[tuple[int, int, int]] = []
    for triangle in triangles:
        triangle_indices = []
        for millimeter_vertex in triangle:
            source_vertex = tuple(
                _format_source_coordinate(value, millimeters_per_unit)
                for value in millimeter_vertex
            )
            if source_vertex not in indices:
                indices[source_vertex] = len(vertices)
                vertices.append(source_vertex)
            triangle_indices.append(indices[source_vertex])
        indexed_triangles.append(tuple(triangle_indices))

    vertex_xml = "\n".join(
        f'          <vertex x="{x}" y="{y}" z="{z}" />'
        for x, y, z in vertices
    )
    triangle_xml = "\n".join(
        f'          <triangle v1="{v1}" v2="{v2}" v3="{v3}" />'
        for v1, v2, v3 in indexed_triangles
    )
    unit_attribute = f' unit="{unit}"' if unit is not None else ""
    transform_attribute = (
        f' transform="{build_transform}"' if build_transform is not None else ""
    )
    return f'''<?xml version="1.0" encoding="UTF-8"?>
<model{unit_attribute} xml:lang="en-US" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
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
    <item objectid="1"{transform_attribute} />
  </build>
</model>
'''.encode("utf-8")


def build_model_xml(source: bytes) -> bytes:
    """Convert the governed 10 mm STL cube to one translated 1 cm model."""
    return build_unit_model_xml(
        source,
        "centimeter",
        "10",
        build_transform="1 0 0 0 1 0 0 0 1 2 3 4",
    )


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


def build_metadata_model_xml(source: bytes) -> bytes:
    """Add bounded public Core metadata without changing the governed cube."""
    direct = build_unit_model_xml(source, "millimeter", "1").decode("utf-8")
    metadata = '''  <metadata name="Title">Governed 10 mm cube</metadata>
  <metadata name="Application">PartProbe fixture generator</metadata>
  <metadata name="Description" preserve="true">Public synthetic metadata fixture</metadata>
'''
    return direct.replace("  <resources>", f"{metadata}  <resources>").encode("utf-8")


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
    return build_package(package_parts(source))


def build_package(parts: tuple[tuple[str, bytes], ...]) -> bytes:
    """Create deterministic Deflate-compressed bytes from ordered OPC parts."""
    output = BytesIO()
    with zipfile.ZipFile(output, "w") as package:
        for name, contents in parts:
            info = zipfile.ZipInfo(name, FIXED_TIMESTAMP)
            info.compress_type = zipfile.ZIP_DEFLATED
            info.create_system = 3
            info.external_attr = 0o100644 << 16
            package.writestr(info, contents, compress_type=zipfile.ZIP_DEFLATED, compresslevel=9)
    return output.getvalue()


def build_component_3mf(source: bytes) -> bytes:
    """Create deterministic bytes for the governed component-transform fixture."""
    return build_package(package_parts(source, component=True))


def build_metadata_3mf(source: bytes) -> bytes:
    """Create deterministic bytes for the governed Core-metadata fixture."""
    parts = list(package_parts(source))
    parts[-1] = (parts[-1][0], build_metadata_model_xml(source))
    return build_package(tuple(parts))


def build_unit_3mf(source: bytes, unit: str | None, millimeters_per_unit: str) -> bytes:
    """Create deterministic bytes for one governed direct-mesh unit fixture."""
    model = build_unit_model_xml(source, unit, millimeters_per_unit)
    parts = list(package_parts(source))
    parts[-1] = (parts[-1][0], model)
    return build_package(tuple(parts))


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


def check_metadata_fixture(
    source_path: Path = SOURCE, output_path: Path = METADATA_OUTPUT
) -> None:
    """Fail when the committed metadata fixture differs from deterministic output."""
    expected = build_metadata_3mf(source_path.read_bytes())
    if not output_path.is_file() or output_path.read_bytes() != expected:
        raise RuntimeError("metadata 3MF fixture is missing or not reproducible")


def check_unit_fixtures(
    source_path: Path = SOURCE,
    fixtures: tuple[tuple[str | None, str, Path], ...] = UNIT_FIXTURES,
) -> None:
    """Fail when any committed Core-unit fixture differs from deterministic output."""
    source = source_path.read_bytes()
    for unit, millimeters_per_unit, output_path in fixtures:
        expected = build_unit_3mf(source, unit, millimeters_per_unit)
        if not output_path.is_file() or output_path.read_bytes() != expected:
            raise RuntimeError("3MF unit fixture is missing or not reproducible")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--write",
        action="store_true",
        help="replace only the governed 3MF outputs with deterministic bytes",
    )
    args = parser.parse_args()
    source = SOURCE.read_bytes()
    expected = build_3mf(source)
    component_expected = build_component_3mf(source)
    metadata_expected = build_metadata_3mf(source)
    unit_expected = tuple(
        (
            output_path,
            build_unit_3mf(source, unit, millimeters_per_unit),
        )
        for unit, millimeters_per_unit, output_path in UNIT_FIXTURES
    )
    if args.write:
        OUTPUT.write_bytes(expected)
        COMPONENT_OUTPUT.write_bytes(component_expected)
        METADATA_OUTPUT.write_bytes(metadata_expected)
        for output_path, contents in unit_expected:
            output_path.write_bytes(contents)
    else:
        check_fixture()
        check_component_fixture()
        check_metadata_fixture()
        check_unit_fixtures()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
