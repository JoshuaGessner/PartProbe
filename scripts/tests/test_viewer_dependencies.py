from __future__ import annotations

import copy
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_viewer_windows_bindings import validate_windows_bindings


def metadata() -> dict:
    return {
        "packages": [
            {"id": "hal", "name": "wgpu-hal", "version": "30.0.1"},
            {"id": "allocator", "name": "gpu-allocator", "version": "0.28.0"},
            {"id": "windows62", "name": "windows", "version": "0.62.2"},
            {"id": "windows61", "name": "windows", "version": "0.61.3"},
        ],
        "resolve": {"nodes": [
            {"id": "hal", "deps": [{"name": "windows", "pkg": "windows62"}]},
            {"id": "allocator", "deps": [{"name": "windows", "pkg": "windows62"}]},
        ]},
    }


class ViewerDependencyTests(unittest.TestCase):
    def test_same_binding_accepts_other_windows_versions_elsewhere(self):
        self.assertEqual(validate_windows_bindings(metadata()), "0.62.2")

    def test_original_mismatch_is_rejected(self):
        value = metadata()
        value["resolve"]["nodes"][1]["deps"][0]["pkg"] = "windows61"
        with self.assertRaisesRegex(ValueError, "different Windows bindings"):
            validate_windows_bindings(value)

    def test_same_version_different_package_source_is_not_same_type(self):
        value = metadata()
        value["packages"][3]["version"] = "0.62.2"
        value["resolve"]["nodes"][1]["deps"][0]["pkg"] = "windows61"
        with self.assertRaisesRegex(ValueError, "different Windows bindings"):
            validate_windows_bindings(value)

    def test_missing_binding_is_rejected(self):
        value = metadata()
        value["resolve"]["nodes"][1]["deps"] = []
        with self.assertRaisesRegex(ValueError, "exactly one resolved windows binding"):
            validate_windows_bindings(value)

    def test_ambiguous_allocator_is_rejected(self):
        value = metadata()
        other = copy.deepcopy(value["packages"][1])
        other["id"] = "allocator2"
        value["packages"].append(other)
        with self.assertRaisesRegex(ValueError, "exactly one resolved gpu-allocator"):
            validate_windows_bindings(value)

    def test_duplicate_binding_is_rejected(self):
        value = metadata()
        value["resolve"]["nodes"][0]["deps"].append({"name": "windows", "pkg": "windows61"})
        with self.assertRaisesRegex(ValueError, "exactly one resolved windows binding"):
            validate_windows_bindings(value)


if __name__ == "__main__":
    unittest.main()
