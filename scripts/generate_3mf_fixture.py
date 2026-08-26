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
NESTED_COMPONENT_OUTPUT = (
    ROOT / "fixtures" / "models" / "cube_1cm_nested_component_chain.3mf"
)
METADATA_OUTPUT = ROOT / "fixtures" / "models" / "cube_10mm_3mf_metadata.3mf"
ADVERSARIAL_FIXTURES: tuple[tuple[str, Path], ...] = (
    (
        "branching_components",
        ROOT / "fixtures" / "models" / "adversarial_3mf_branching_components.3mf",
    ),
    (
        "non_immediate_reference",
        ROOT / "fixtures" / "models" / "adversarial_3mf_non_immediate_reference.3mf",
    ),
    (
        "object_metadata",
        ROOT / "fixtures" / "models" / "adversarial_3mf_object_metadata.3mf",
    ),
    (
        "relationship_traversal",
        ROOT / "fixtures" / "models" / "adversarial_3mf_relationship_traversal.3mf",
    ),
    (
        "case_ambiguous_part",
        ROOT / "fixtures" / "models" / "adversarial_3mf_case_ambiguous_part.3mf",
    ),
    (
        "build_union",
        ROOT / "fixtures" / "models" / "adversarial_3mf_build_union.3mf",
    ),
    (
        "item_metadata",
        ROOT / "fixtures" / "models" / "adversarial_3mf_item_metadata.3mf",
    ),
    (
        "vendor_metadata",
        ROOT / "fixtures" / "models" / "adversarial_3mf_vendor_metadata.3mf",
    ),
    (
        "high_compression_ratio",
        ROOT / "fixtures" / "models" / "adversarial_3mf_high_compression_ratio.3mf",
    ),
    (
        "unsupported_compression",
        ROOT / "fixtures" / "models" / "adversarial_3mf_unsupported_compression.3mf",
    ),
    (
        "forward_component_reference",
        ROOT / "fixtures" / "models" / "adversarial_3mf_forward_component_reference.3mf",
    ),
    (
        "unused_component_object",
        ROOT / "fixtures" / "models" / "adversarial_3mf_unused_component_object.3mf",
    ),
    (
        "material_attribute",
        ROOT / "fixtures" / "models" / "adversarial_3mf_material_attribute.3mf",
    ),
    (
        "required_extension",
        ROOT / "fixtures" / "models" / "adversarial_3mf_required_extension.3mf",
    ),
    (
        "encrypted_entry",
        ROOT / "fixtures" / "models" / "adversarial_3mf_encrypted_entry.3mf",
    ),
    (
        "absolute_entry_name",
        ROOT / "fixtures" / "models" / "adversarial_3mf_absolute_entry_name.3mf",
    ),
    (
        "backslash_entry_name",
        ROOT / "fixtures" / "models" / "adversarial_3mf_backslash_entry_name.3mf",
    ),
    (
        "directory_entry",
        ROOT / "fixtures" / "models" / "adversarial_3mf_directory_entry.3mf",
    ),
    (
        "malformed_model_xml",
        ROOT / "fixtures" / "models" / "adversarial_3mf_malformed_model_xml.3mf",
    ),
    (
        "document_type",
        ROOT / "fixtures" / "models" / "adversarial_3mf_document_type.3mf",
    ),
    (
        "entry_count_limit",
        ROOT / "fixtures" / "models" / "adversarial_3mf_entry_count_limit.3mf",
    ),
)
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
ALTERNATE_OPC_FIXTURES: tuple[tuple[str, Path], ...] = (
    (
        "default_content_type",
        ROOT / "fixtures" / "models" / "cube_10mm_3mf_default_content_type.3mf",
    ),
    (
        "alternate_model_part",
        ROOT / "fixtures" / "models" / "cube_10mm_3mf_alternate_model_part.3mf",
    ),
    (
        "stored_compression",
        ROOT / "fixtures" / "models" / "cube_10mm_3mf_stored_compression.3mf",
    ),
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


def build_nested_component_model_xml(source: bytes) -> bytes:
    """Wrap the governed mesh in a two-link linear component chain."""
    component = build_component_model_xml(source).decode("utf-8")
    outer_component_object = '''    <object id="3" type="model">
      <components>
        <component objectid="2" transform="1 0 0 0 3 0 0 0 1 1 0 2" />
      </components>
    </object>
'''
    model = component.replace(
        "  </resources>\n  <build>",
        f"{outer_component_object}  </resources>\n  <build>",
    ).replace(
        '<item objectid="2" transform="1 0 0 0 1 0 0 0 1 4 5 6" />',
        '<item objectid="3" transform="1 0 0 0 1 0 0 0 1 4 5 6" />',
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


def build_package(
    parts: tuple[tuple[str, bytes], ...],
    *,
    compression: int = zipfile.ZIP_DEFLATED,
) -> bytes:
    """Create deterministic compressed bytes from ordered OPC parts."""
    output = BytesIO()
    with zipfile.ZipFile(output, "w") as package:
        for name, contents in parts:
            info = zipfile.ZipInfo(name, FIXED_TIMESTAMP)
            info.compress_type = compression
            info.create_system = 3
            info.external_attr = 0o100644 << 16
            package.writestr(info, contents, compress_type=compression, compresslevel=9)
    return output.getvalue()


def declare_first_entry_compression(package: bytes, compression: int) -> bytes:
    """Rewrite one stored entry's local and central compression declarations."""
    declared = compression.to_bytes(2, "little")
    output = bytearray(package)
    if output[:4] != b"PK\x03\x04":
        raise RuntimeError("generated ZIP is missing its first local header")
    central_header = output.find(b"PK\x01\x02")
    if central_header < 0:
        raise RuntimeError("generated ZIP is missing its central directory")
    output[8:10] = declared
    output[central_header + 10 : central_header + 12] = declared
    return bytes(output)


def declare_first_entry_encrypted(package: bytes) -> bytes:
    """Set the encryption flag on one deterministic entry without adding secrets."""
    output = bytearray(package)
    if output[:4] != b"PK\x03\x04":
        raise RuntimeError("generated ZIP is missing its first local header")
    central_header = output.find(b"PK\x01\x02")
    if central_header < 0:
        raise RuntimeError("generated ZIP is missing its central directory")
    local_flags = int.from_bytes(output[6:8], "little") | 1
    central_flags = int.from_bytes(
        output[central_header + 8 : central_header + 10], "little"
    ) | 1
    output[6:8] = local_flags.to_bytes(2, "little")
    output[central_header + 8 : central_header + 10] = central_flags.to_bytes(
        2, "little"
    )
    return bytes(output)


def replace_entry_name_bytes(package: bytes, current: str, replacement: str) -> bytes:
    """Rewrite equal-length local/central ZIP names without host path semantics."""
    current_bytes = current.encode("ascii")
    replacement_bytes = replacement.encode("ascii")
    if len(current_bytes) != len(replacement_bytes):
        raise ValueError("ZIP entry-name replacement must preserve byte length")
    if package.count(current_bytes) != 2:
        raise RuntimeError("generated ZIP entry name is not present exactly twice")
    return package.replace(current_bytes, replacement_bytes)


def build_component_3mf(source: bytes) -> bytes:
    """Create deterministic bytes for the governed component-transform fixture."""
    return build_package(package_parts(source, component=True))


def build_nested_component_3mf(source: bytes) -> bytes:
    """Create deterministic bytes for the governed two-link component fixture."""
    parts = list(package_parts(source))
    parts[-1] = (parts[-1][0], build_nested_component_model_xml(source))
    return build_package(tuple(parts))


def build_adversarial_3mf(source: bytes, case: str) -> bytes:
    """Create one deterministic public package that the bounded parser must reject."""
    parts = list(package_parts(source))
    if case == "branching_components":
        model = build_component_model_xml(source).decode("utf-8").replace(
            "      </components>",
            '        <component objectid="1" />\n      </components>',
        )
        parts[-1] = (parts[-1][0], model.encode("utf-8"))
    elif case == "non_immediate_reference":
        model = build_nested_component_model_xml(source).decode("utf-8").replace(
            '<component objectid="2" transform="1 0 0 0 3 0 0 0 1 1 0 2" />',
            '<component objectid="1" transform="1 0 0 0 3 0 0 0 1 1 0 2" />',
        )
        parts[-1] = (parts[-1][0], model.encode("utf-8"))
    elif case == "object_metadata":
        model = build_unit_model_xml(source, "millimeter", "1").decode("utf-8").replace(
            "      <mesh>",
            '      <metadatagroup><metadata name="Title">Rejected object metadata</metadata></metadatagroup>\n      <mesh>',
        )
        parts[-1] = (parts[-1][0], model.encode("utf-8"))
    elif case == "relationship_traversal":
        relationships = parts[1][1].replace(
            b'Target="/3D/3dmodel.model"',
            b'Target="/../escape.model"',
        )
        parts[1] = (parts[1][0], relationships)
    elif case == "case_ambiguous_part":
        parts.append(("3d/3dmodel.model", parts[-1][1]))
    elif case == "build_union":
        model = build_unit_model_xml(source, "millimeter", "1").decode("utf-8").replace(
            "    <item objectid=\"1\" />",
            '    <item objectid="1" />\n    <item objectid="1" />',
        )
        parts[-1] = (parts[-1][0], model.encode("utf-8"))
    elif case == "item_metadata":
        model = build_unit_model_xml(source, "millimeter", "1").decode("utf-8").replace(
            '    <item objectid="1" />',
            '''    <item objectid="1">
      <metadatagroup><metadata name="Title">Rejected item metadata</metadata></metadatagroup>
    </item>''',
        )
        parts[-1] = (parts[-1][0], model.encode("utf-8"))
    elif case == "vendor_metadata":
        model = build_unit_model_xml(source, "millimeter", "1").decode("utf-8").replace(
            "  <resources>",
            '  <metadata name="VendorData">Rejected vendor metadata</metadata>\n  <resources>',
        )
        parts[-1] = (parts[-1][0], model.encode("utf-8"))
    elif case == "high_compression_ratio":
        parts.append(("Metadata/repeated-padding.bin", b"0" * (32 * 1024)))
    elif case == "unsupported_compression":
        stored = build_package(tuple(parts), compression=zipfile.ZIP_STORED)
        return declare_first_entry_compression(stored, zipfile.ZIP_BZIP2)
    elif case == "forward_component_reference":
        model = build_nested_component_model_xml(source).decode("utf-8").replace(
            '<component objectid="1" transform="2 0 0 0 1 0 0 0 1 1 2 3" />',
            '<component objectid="3" transform="2 0 0 0 1 0 0 0 1 1 2 3" />',
        )
        parts[-1] = (parts[-1][0], model.encode("utf-8"))
    elif case == "unused_component_object":
        model = build_component_model_xml(source).decode("utf-8").replace(
            '<item objectid="2" transform="1 0 0 0 1 0 0 0 1 4 5 6" />',
            '<item objectid="1" transform="1 0 0 0 1 0 0 0 1 4 5 6" />',
        )
        parts[-1] = (parts[-1][0], model.encode("utf-8"))
    elif case == "material_attribute":
        model = build_unit_model_xml(source, "millimeter", "1").decode("utf-8").replace(
            "<triangle ", '<triangle pid="2" ', 1
        )
        parts[-1] = (parts[-1][0], model.encode("utf-8"))
    elif case == "required_extension":
        model = build_model_xml(source).decode("utf-8").replace(
            '<model unit="centimeter"',
            '<model requiredextensions="foo" xmlns:foo="urn:example" unit="centimeter"',
        )
        parts[-1] = (parts[-1][0], model.encode("utf-8"))
    elif case == "encrypted_entry":
        return declare_first_entry_encrypted(build_package(tuple(parts)))
    elif case == "absolute_entry_name":
        parts.append(("/Metadata/absolute.xml", b"unsafe absolute entry name"))
    elif case == "backslash_entry_name":
        portable_name = "Metadata/backslash.xml"
        parts.append((portable_name, b"unsafe backslash entry name"))
        return replace_entry_name_bytes(
            build_package(tuple(parts)), portable_name, r"Metadata\backslash.xml"
        )
    elif case == "directory_entry":
        parts.append(("Metadata/", b""))
    elif case == "malformed_model_xml":
        parts[-1] = (parts[-1][0], parts[-1][1].replace(b"</model>", b"</broken>"))
    elif case == "document_type":
        parts[-1] = (
            parts[-1][0],
            parts[-1][1].replace(
                b'<?xml version="1.0" encoding="UTF-8"?>',
                b'<?xml version="1.0" encoding="UTF-8"?>\n<!DOCTYPE model>',
            ),
        )
    elif case == "entry_count_limit":
        for index in range(14):
            parts.append((f"Metadata/entry-{index:02d}.bin", b"bounded entry"))
    else:
        raise ValueError(f"unsupported adversarial 3MF case: {case}")
    return build_package(tuple(parts))


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


def build_alternate_opc_3mf(source: bytes, case: str) -> bytes:
    """Create one deterministic package-layout variant of the 10 mm cube."""
    parts = list(package_parts(source))
    if case == "default_content_type":
        parts[0] = (
            parts[0][0],
            b'''<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml" />
</Types>
''',
        )
    elif case == "alternate_model_part":
        parts[0] = (
            parts[0][0],
            parts[0][1].replace(b"/3D/3dmodel.model", b"/3D/primary.model"),
        )
        parts[1] = (
            parts[1][0],
            parts[1][1]
            .replace(b"/3D/3dmodel.model", b"/3D/primary.model")
            .replace(b" />", b' TargetMode="Internal" />'),
        )
        parts[2] = ("3D/primary.model", parts[2][1])
    elif case == "stored_compression":
        return build_package(tuple(parts), compression=zipfile.ZIP_STORED)
    else:
        raise ValueError(f"unsupported alternate OPC case: {case}")
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


def check_nested_component_fixture(
    source_path: Path = SOURCE, output_path: Path = NESTED_COMPONENT_OUTPUT
) -> None:
    """Fail when the committed nested-component fixture is not reproducible."""
    expected = build_nested_component_3mf(source_path.read_bytes())
    if not output_path.is_file() or output_path.read_bytes() != expected:
        raise RuntimeError("nested component 3MF fixture is missing or not reproducible")


def check_adversarial_fixtures(
    source_path: Path = SOURCE,
    fixtures: tuple[tuple[str, Path], ...] = ADVERSARIAL_FIXTURES,
) -> None:
    """Fail when any committed adversarial package is not reproducible."""
    source = source_path.read_bytes()
    for case, output_path in fixtures:
        expected = build_adversarial_3mf(source, case)
        if not output_path.is_file() or output_path.read_bytes() != expected:
            raise RuntimeError("adversarial 3MF fixture is missing or not reproducible")


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


def check_alternate_opc_fixtures(
    source_path: Path = SOURCE,
    fixtures: tuple[tuple[str, Path], ...] = ALTERNATE_OPC_FIXTURES,
) -> None:
    """Fail when any committed alternate OPC fixture is not reproducible."""
    source = source_path.read_bytes()
    for case, output_path in fixtures:
        expected = build_alternate_opc_3mf(source, case)
        if not output_path.is_file() or output_path.read_bytes() != expected:
            raise RuntimeError("alternate OPC fixture is missing or not reproducible")


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
    nested_component_expected = build_nested_component_3mf(source)
    metadata_expected = build_metadata_3mf(source)
    adversarial_expected = tuple(
        (output_path, build_adversarial_3mf(source, case))
        for case, output_path in ADVERSARIAL_FIXTURES
    )
    unit_expected = tuple(
        (
            output_path,
            build_unit_3mf(source, unit, millimeters_per_unit),
        )
        for unit, millimeters_per_unit, output_path in UNIT_FIXTURES
    )
    alternate_opc_expected = tuple(
        (output_path, build_alternate_opc_3mf(source, case))
        for case, output_path in ALTERNATE_OPC_FIXTURES
    )
    if args.write:
        OUTPUT.write_bytes(expected)
        COMPONENT_OUTPUT.write_bytes(component_expected)
        NESTED_COMPONENT_OUTPUT.write_bytes(nested_component_expected)
        METADATA_OUTPUT.write_bytes(metadata_expected)
        for output_path, contents in adversarial_expected:
            output_path.write_bytes(contents)
        for output_path, contents in unit_expected:
            output_path.write_bytes(contents)
        for output_path, contents in alternate_opc_expected:
            output_path.write_bytes(contents)
    else:
        check_fixture()
        check_component_fixture()
        check_nested_component_fixture()
        check_metadata_fixture()
        check_adversarial_fixtures()
        check_unit_fixtures()
        check_alternate_opc_fixtures()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
