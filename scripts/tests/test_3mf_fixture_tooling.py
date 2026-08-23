"""Regression tests for the deterministic governed 3MF fixture generator."""

from pathlib import Path
import sys
import tempfile
import unittest
import zipfile
from io import BytesIO


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
