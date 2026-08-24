from __future__ import annotations

from pathlib import Path
import hashlib
import json
import math
import struct
import sys
import tempfile
import unittest


SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))

import generate_ascii_stl_adversarial_fixtures  # noqa: E402
import generate_binary_stl_fixture  # noqa: E402


class BinaryStlFixtureTests(unittest.TestCase):
    def test_generated_fixture_has_exact_binary_stl_framing(self) -> None:
        source = generate_binary_stl_fixture.SOURCE.read_bytes()
        generated = generate_binary_stl_fixture.build_binary_stl(source)

        triangle_count = struct.unpack_from("<I", generated, 80)[0]
        self.assertEqual(triangle_count, 12)
        self.assertEqual(len(generated), 84 + triangle_count * 50)
        self.assertFalse(generated[:80].startswith(b"solid"))

    def test_committed_fixture_is_reproducible(self) -> None:
        generate_binary_stl_fixture.check_fixture()
        generate_binary_stl_fixture.check_adversarial_fixtures()

    def test_adversarial_binary_corpus_pins_four_distinct_failures(self) -> None:
        source = generate_binary_stl_fixture.SOURCE.read_bytes()
        expected_ids = {
            "truncated_record": "FIX-MESH-032",
            "attribute_data": "FIX-MESH-033",
            "non_finite_normal": "FIX-MESH-037",
            "triangle_count_limit": "FIX-MESH-038",
        }
        generated = {}
        for case, output_path in generate_binary_stl_fixture.ADVERSARIAL_FIXTURES:
            contents = generate_binary_stl_fixture.build_adversarial_binary_stl(
                source, case
            )
            generated[case] = contents
            self.assertEqual(contents, output_path.read_bytes())
            expectation_path = (
                generate_binary_stl_fixture.ROOT
                / "fixtures"
                / "expected"
                / f"adversarial_binary_stl_{case}_rejection.json"
            )
            expectation = json.loads(expectation_path.read_text())
            self.assertEqual(expectation["fixture_id"], expected_ids[case])
            self.assertEqual(
                expectation["source_sha256"], hashlib.sha256(contents).hexdigest()
            )
            self.assertFalse(expectation["snapshot_expected"])

        self.assertEqual(len(generated), 4)
        self.assertEqual(len(set(generated.values())), 4)
        self.assertEqual(
            len(generated["truncated_record"]), len(generated["attribute_data"]) - 1
        )
        self.assertEqual(struct.unpack_from("<H", generated["attribute_data"], 132)[0], 1)
        self.assertTrue(
            math.isnan(
                struct.unpack_from("<f", generated["non_finite_normal"], 84)[0]
            )
        )
        self.assertEqual(
            struct.unpack_from("<I", generated["triangle_count_limit"], 80)[0],
            1_001,
        )

    def test_check_rejects_changed_output(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            changed = Path(temporary) / "changed.stl"
            changed.write_bytes(b"not the governed fixture")
            with self.assertRaisesRegex(RuntimeError, "not reproducible"):
                generate_binary_stl_fixture.check_fixture(
                    generate_binary_stl_fixture.SOURCE,
                    changed,
                )
            with self.assertRaisesRegex(RuntimeError, "not reproducible"):
                generate_binary_stl_fixture.check_adversarial_fixtures(
                    generate_binary_stl_fixture.SOURCE,
                    (("truncated_record", changed),),
                )


class AsciiStlAdversarialFixtureTests(unittest.TestCase):
    def test_committed_ascii_rejection_corpus_is_reproducible(self) -> None:
        generate_ascii_stl_adversarial_fixtures.check_fixtures()

    def test_ascii_rejection_corpus_pins_four_distinct_failures(self) -> None:
        expected = {
            "invalid_utf8": ("FIX-MESH-040", "STL_INVALID_TEXT"),
            "malformed_facet": ("FIX-MESH-041", "STL_INVALID_STRUCTURE"),
            "empty_solid": ("FIX-MESH-042", "STL_EMPTY_MESH"),
            "degenerate_triangle": ("FIX-MESH-043", "STL_DEGENERATE_TRIANGLE"),
        }
        generated = {}
        for case, output_path in (
            generate_ascii_stl_adversarial_fixtures.ADVERSARIAL_FIXTURES
        ):
            contents = (
                generate_ascii_stl_adversarial_fixtures.build_adversarial_ascii_stl(
                    case
                )
            )
            generated[case] = contents
            self.assertEqual(contents, output_path.read_bytes())
            expectation_path = (
                generate_ascii_stl_adversarial_fixtures.ROOT
                / "fixtures"
                / "expected"
                / f"adversarial_ascii_stl_{case}_rejection.json"
            )
            expectation = json.loads(expectation_path.read_text())
            fixture_id, diagnostic = expected[case]
            self.assertEqual(expectation["fixture_id"], fixture_id)
            self.assertEqual(expectation["expected_diagnostic_code"], diagnostic)
            self.assertEqual(
                expectation["source_sha256"], hashlib.sha256(contents).hexdigest()
            )
            self.assertFalse(expectation["snapshot_expected"])

        self.assertEqual(len(generated), 4)
        self.assertEqual(len(set(generated.values())), 4)
        with self.assertRaises(UnicodeDecodeError):
            generated["invalid_utf8"].decode("utf-8")

    def test_ascii_rejection_check_rejects_changed_output(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            changed = Path(temporary) / "changed.stl"
            changed.write_bytes(b"not the governed fixture")
            with self.assertRaisesRegex(RuntimeError, "not reproducible"):
                generate_ascii_stl_adversarial_fixtures.check_fixtures(
                    (("invalid_utf8", changed),)
                )


if __name__ == "__main__":
    unittest.main()
