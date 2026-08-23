from __future__ import annotations

from pathlib import Path
import struct
import sys
import tempfile
import unittest


SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))

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

    def test_check_rejects_changed_output(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            changed = Path(temporary) / "changed.stl"
            changed.write_bytes(b"not the governed fixture")
            with self.assertRaisesRegex(RuntimeError, "not reproducible"):
                generate_binary_stl_fixture.check_fixture(
                    generate_binary_stl_fixture.SOURCE,
                    changed,
                )


if __name__ == "__main__":
    unittest.main()
