"""Regression tests for the deterministic governed 3MF fixture generator."""

import hashlib
import json
import sys
import tempfile
import unittest
import zipfile
from io import BytesIO
from pathlib import Path


SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))

import generate_3mf_fixture  # noqa: E402


class ThreeMfFixtureToolingTests(unittest.TestCase):
    def test_generated_package_has_exact_governed_parts(self) -> None:
        generated = generate_3mf_fixture.build_3mf(
            generate_3mf_fixture.SOURCE.read_bytes()
        )
        with zipfile.ZipFile(BytesIO(generated)) as package:
            self.assertEqual(
                package.namelist(),
                ["[Content_Types].xml", "_rels/.rels", "3D/3dmodel.model"],
            )
            model = package.read("3D/3dmodel.model")
        self.assertIn(b'unit="centimeter"', model)
        self.assertIn(b'transform="1 0 0 0 1 0 0 0 1 2 3 4"', model)
        self.assertEqual(model.count(b"<vertex "), 8)
        self.assertEqual(model.count(b"<triangle "), 12)

    def test_committed_package_is_reproducible(self) -> None:
        generate_3mf_fixture.check_fixture()
        generate_3mf_fixture.check_component_fixture()
        generate_3mf_fixture.check_nested_component_fixture()
        generate_3mf_fixture.check_metadata_fixture()
        generate_3mf_fixture.check_adversarial_fixtures()
        generate_3mf_fixture.check_unit_fixtures()

    def test_metadata_package_is_bounded_public_core_evidence(self) -> None:
        generated = generate_3mf_fixture.build_metadata_3mf(
            generate_3mf_fixture.SOURCE.read_bytes()
        )
        self.assertEqual(generated, generate_3mf_fixture.METADATA_OUTPUT.read_bytes())
        with zipfile.ZipFile(BytesIO(generated)) as package:
            model = package.read("3D/3dmodel.model")
        self.assertEqual(model.count(b"<metadata "), 3)
        self.assertIn(b'<metadata name="Title">Governed 10 mm cube</metadata>', model)
        self.assertIn(b'<metadata name="Description" preserve="true">', model)
        self.assertNotIn(b"customer", model.lower())
        self.assertEqual(model.count(b"<vertex "), 8)
        self.assertEqual(model.count(b"<triangle "), 12)

    def test_unit_corpus_has_every_remaining_declaration_and_default(self) -> None:
        source = generate_3mf_fixture.SOURCE.read_bytes()
        seen_units = []
        for unit, millimeters_per_unit, output_path in generate_3mf_fixture.UNIT_FIXTURES:
            generated = generate_3mf_fixture.build_unit_3mf(
                source,
                unit,
                millimeters_per_unit,
            )
            self.assertEqual(generated, output_path.read_bytes())
            with zipfile.ZipFile(BytesIO(generated)) as package:
                model = package.read("3D/3dmodel.model")
            if unit is None:
                self.assertNotIn(b" unit=", model)
            else:
                self.assertIn(f'unit="{unit}"'.encode(), model)
            self.assertIn(b'<item objectid="1" />', model)
            self.assertEqual(model.count(b"<vertex "), 8)
            self.assertEqual(model.count(b"<triangle "), 12)
            seen_units.append(unit)
        self.assertEqual(
            seen_units,
            ["micron", "millimeter", "meter", "inch", "foot", None],
        )

    def test_component_package_has_exact_governed_transform_chain(self) -> None:
        generated = generate_3mf_fixture.build_component_3mf(
            generate_3mf_fixture.SOURCE.read_bytes()
        )
        with zipfile.ZipFile(BytesIO(generated)) as package:
            model = package.read("3D/3dmodel.model")
        self.assertEqual(model.count(b"<object "), 2)
        self.assertEqual(model.count(b"<component "), 1)
        self.assertIn(
            b'transform="2 0 0 0 1 0 0 0 1 1 2 3"',
            model,
        )
        self.assertIn(
            b'objectid="2" transform="1 0 0 0 1 0 0 0 1 4 5 6"',
            model,
        )

    def test_nested_component_package_has_exact_linear_chain(self) -> None:
        generated = generate_3mf_fixture.build_nested_component_3mf(
            generate_3mf_fixture.SOURCE.read_bytes()
        )
        self.assertEqual(
            generated, generate_3mf_fixture.NESTED_COMPONENT_OUTPUT.read_bytes()
        )
        with zipfile.ZipFile(BytesIO(generated)) as package:
            model = package.read("3D/3dmodel.model")
        self.assertEqual(model.count(b"<object "), 3)
        self.assertEqual(model.count(b"<component "), 2)
        self.assertIn(
            b'objectid="2" transform="1 0 0 0 3 0 0 0 1 1 0 2"',
            model,
        )
        self.assertIn(
            b'objectid="3" transform="1 0 0 0 1 0 0 0 1 4 5 6"',
            model,
        )

    def test_adversarial_corpus_pins_fifteen_distinct_rejection_shapes(self) -> None:
        source = generate_3mf_fixture.SOURCE.read_bytes()
        generated = {
            case: generate_3mf_fixture.build_adversarial_3mf(source, case)
            for case, _ in generate_3mf_fixture.ADVERSARIAL_FIXTURES
        }
        self.assertEqual(len(generated), 15)
        self.assertEqual(len(set(generated.values())), 15)
        expected_ids = {
            "branching_components": "FIX-MESH-014",
            "non_immediate_reference": "FIX-MESH-015",
            "object_metadata": "FIX-MESH-016",
            "relationship_traversal": "FIX-MESH-017",
            "case_ambiguous_part": "FIX-MESH-018",
            "build_union": "FIX-MESH-019",
            "item_metadata": "FIX-MESH-020",
            "vendor_metadata": "FIX-MESH-021",
            "high_compression_ratio": "FIX-MESH-022",
            "unsupported_compression": "FIX-MESH-023",
            "forward_component_reference": "FIX-MESH-024",
            "unused_component_object": "FIX-MESH-025",
            "material_attribute": "FIX-MESH-026",
            "required_extension": "FIX-MESH-027",
            "encrypted_entry": "FIX-MESH-028",
        }
        for case, output_path in generate_3mf_fixture.ADVERSARIAL_FIXTURES:
            self.assertEqual(generated[case], output_path.read_bytes())
            expectation_path = (
                generate_3mf_fixture.ROOT
                / "fixtures"
                / "expected"
                / f"adversarial_3mf_{case}_rejection.json"
            )
            expectation = json.loads(expectation_path.read_text())
            self.assertEqual(expectation["fixture_id"], expected_ids[case])
            self.assertEqual(
                expectation["source_sha256"],
                hashlib.sha256(generated[case]).hexdigest(),
            )
            self.assertFalse(expectation["snapshot_expected"])

        with zipfile.ZipFile(BytesIO(generated["branching_components"])) as package:
            self.assertEqual(package.read("3D/3dmodel.model").count(b"<component "), 2)
        with zipfile.ZipFile(BytesIO(generated["non_immediate_reference"])) as package:
            model = package.read("3D/3dmodel.model")
            self.assertIn(b'<object id="3"', model)
            self.assertIn(b'<component objectid="1" transform="1 0 0 0 3', model)
        with zipfile.ZipFile(BytesIO(generated["object_metadata"])) as package:
            self.assertIn(b"<metadatagroup>", package.read("3D/3dmodel.model"))
        with zipfile.ZipFile(BytesIO(generated["relationship_traversal"])) as package:
            self.assertIn(b'Target="/../escape.model"', package.read("_rels/.rels"))
        with zipfile.ZipFile(BytesIO(generated["case_ambiguous_part"])) as package:
            self.assertIn("3d/3dmodel.model", package.namelist())
        with zipfile.ZipFile(BytesIO(generated["build_union"])) as package:
            self.assertEqual(package.read("3D/3dmodel.model").count(b"<item "), 2)
        with zipfile.ZipFile(BytesIO(generated["item_metadata"])) as package:
            self.assertIn(b"<metadatagroup>", package.read("3D/3dmodel.model"))
        with zipfile.ZipFile(BytesIO(generated["vendor_metadata"])) as package:
            self.assertIn(b'name="VendorData"', package.read("3D/3dmodel.model"))
        with zipfile.ZipFile(BytesIO(generated["high_compression_ratio"])) as package:
            padding = package.getinfo("Metadata/repeated-padding.bin")
            self.assertGreater(padding.file_size, padding.compress_size * 100)
        with zipfile.ZipFile(BytesIO(generated["unsupported_compression"])) as package:
            self.assertEqual(
                package.getinfo("[Content_Types].xml").compress_type,
                zipfile.ZIP_BZIP2,
            )
        with zipfile.ZipFile(BytesIO(generated["forward_component_reference"])) as package:
            self.assertIn(
                b'<component objectid="3" transform="2 0 0',
                package.read("3D/3dmodel.model"),
            )
        with zipfile.ZipFile(BytesIO(generated["unused_component_object"])) as package:
            self.assertIn(
                b'<item objectid="1" transform="1 0 0',
                package.read("3D/3dmodel.model"),
            )
        with zipfile.ZipFile(BytesIO(generated["material_attribute"])) as package:
            self.assertIn(b'<triangle pid="2"', package.read("3D/3dmodel.model"))
        with zipfile.ZipFile(BytesIO(generated["required_extension"])) as package:
            self.assertIn(b'requiredextensions="foo"', package.read("3D/3dmodel.model"))
        with zipfile.ZipFile(BytesIO(generated["encrypted_entry"])) as package:
            self.assertTrue(package.getinfo("[Content_Types].xml").flag_bits & 1)

    def test_check_rejects_changed_output(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            changed = Path(directory) / "changed.3mf"
            changed.write_bytes(b"not the governed fixture")
            with self.assertRaisesRegex(RuntimeError, "not reproducible"):
                generate_3mf_fixture.check_fixture(
                    generate_3mf_fixture.SOURCE,
                    changed,
                )

            with self.assertRaisesRegex(RuntimeError, "not reproducible"):
                generate_3mf_fixture.check_component_fixture(
                    generate_3mf_fixture.SOURCE,
                    changed,
                )

            with self.assertRaisesRegex(RuntimeError, "not reproducible"):
                generate_3mf_fixture.check_nested_component_fixture(
                    generate_3mf_fixture.SOURCE,
                    changed,
                )

            with self.assertRaisesRegex(RuntimeError, "not reproducible"):
                generate_3mf_fixture.check_adversarial_fixtures(
                    generate_3mf_fixture.SOURCE,
                    (("branching_components", changed),),
                )

            with self.assertRaisesRegex(RuntimeError, "not reproducible"):
                generate_3mf_fixture.check_metadata_fixture(
                    generate_3mf_fixture.SOURCE,
                    changed,
                )

            with self.assertRaisesRegex(RuntimeError, "not reproducible"):
                generate_3mf_fixture.check_unit_fixtures(
                    generate_3mf_fixture.SOURCE,
                    (("millimeter", "1", changed),),
                )


if __name__ == "__main__":
    unittest.main()
