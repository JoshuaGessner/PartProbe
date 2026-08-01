from __future__ import annotations

import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest import mock


SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))

import build_occt  # noqa: E402
import verify_native_step  # noqa: E402


class BuildOcctTests(unittest.TestCase):
    def test_configure_command_contains_the_reviewed_minimal_profile(self) -> None:
        source = Path("/source")
        build = Path("/build")
        install = Path("/install")
        command = build_occt.configure_command(
            source, build, install, "Ninja"
        )

        self.assertEqual(
            command[:7],
            ["cmake", "-S", str(source), "-B", str(build), "-G", "Ninja"],
        )
        self.assertIn("-DBUILD_LIBRARY_TYPE=Shared", command)
        self.assertIn("-DBUILD_CPP_STANDARD=C++17", command)
        self.assertIn("-DBUILD_ADDITIONAL_TOOLKITS=TKDESTEP;TKShHealing;TKMesh", command)
        self.assertIn("-DBUILD_MODULE_DataExchange=OFF", command)
        self.assertIn("-DUSE_TBB=OFF", command)
        self.assertIn("-DUSE_FREETYPE=OFF", command)
        self.assertEqual(command[-1], f"-DINSTALL_DIR={install}")

    def test_output_paths_reject_source_children_and_nested_outputs(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "source"
            source.mkdir()
            with self.assertRaises(ValueError):
                build_occt.validate_output_paths(
                    source, source / "build", root / "install", False
                )
            with self.assertRaises(ValueError):
                build_occt.validate_output_paths(
                    source, root / "output", root / "output" / "install", False
                )

    def test_source_validation_requires_commit_tag_and_clean_tree(self) -> None:
        responses = iter(
            [build_occt.EXPECTED_OCCT_COMMIT, "", "V8_0_0", "tree-digest"]
        )
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary)
            (source / ".git").mkdir()
            with mock.patch.object(
                build_occt, "run_capture", side_effect=lambda *_args, **_kwargs: next(responses)
            ):
                self.assertEqual(
                    build_occt.validate_source(source),
                    (build_occt.EXPECTED_OCCT_COMMIT, "tree-digest"),
                )

    def test_manifest_records_selected_generator_and_compilers(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            build = Path(temporary)
            (build / "CMakeCache.txt").write_text(
                "CMAKE_GENERATOR:INTERNAL=Ninja\n"
                "CMAKE_C_COMPILER:FILEPATH=/tool/cc\n"
                "CMAKE_CXX_COMPILER:FILEPATH=/tool/c++\n",
                encoding="utf-8",
            )
            with mock.patch.object(
                build_occt, "run_capture", return_value="cmake version 4.3.4"
            ):
                manifest_path = build_occt.write_manifest(
                    build, build_occt.EXPECTED_OCCT_COMMIT, "tree", 4
                )
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))

            self.assertEqual(manifest["cmake_generator"], "Ninja")
            self.assertEqual(manifest["cxx_compiler"], "/tool/c++")
            self.assertEqual(manifest["additional_toolkits"], ["TKDESTEP", "TKShHealing", "TKMesh"])


class VerifyNativeStepTests(unittest.TestCase):
    def create_install(self, root: Path, version: str = "8.0.0") -> None:
        include = root / "include" / "opencascade"
        libraries = root / "lib"
        include.mkdir(parents=True)
        libraries.mkdir()
        (include / "Standard_Version.hxx").write_text(
            f'#define OCC_VERSION_COMPLETE "{version}"\n', encoding="utf-8"
        )
        for name in verify_native_step.REQUIRED_LIBRARIES:
            (libraries / f"lib{name}.so").write_bytes(name.encode("ascii"))

    def test_install_fingerprint_is_content_addressed(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.create_install(root)
            fingerprint = verify_native_step.inspect_occt(root)

            self.assertEqual(fingerprint["occt_version"], "8.0.0")
            self.assertEqual(
                set(fingerprint["libraries"]),
                set(verify_native_step.REQUIRED_LIBRARIES),
            )
            self.assertTrue(
                all(len(value) == 64 for value in fingerprint["libraries"].values())
            )

    def test_install_fingerprint_rejects_another_occt_version(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.create_install(root, "7.9.3")
            with self.assertRaises(ValueError):
                verify_native_step.inspect_occt(root)


if __name__ == "__main__":
    unittest.main()
