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
import assemble_native_runtime  # noqa: E402
import verify_native_step  # noqa: E402
import verify_native_runtime_links  # noqa: E402


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
            [
                build_occt.EXPECTED_OCCT_COMMIT,
                "",
                "V8_0_0",
                build_occt.EXPECTED_OCCT_TREE,
            ]
        )
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary)
            (source / ".git").mkdir()
            with mock.patch.object(
                build_occt, "run_capture", side_effect=lambda *_args, **_kwargs: next(responses)
            ):
                self.assertEqual(
                    build_occt.validate_source(source),
                    (
                        build_occt.EXPECTED_OCCT_COMMIT,
                        build_occt.EXPECTED_OCCT_TREE,
                    ),
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


class AssembleNativeRuntimeTests(unittest.TestCase):
    def create_inputs(self, root: Path) -> tuple[Path, Path, Path]:
        system, machine = assemble_native_runtime.host_identity()
        install = root / "install"
        include = install / "include" / "opencascade"
        libraries = install / "lib"
        include.mkdir(parents=True)
        libraries.mkdir()
        (include / "Standard_Version.hxx").write_text(
            '#define OCC_VERSION_COMPLETE "8.0.0"\n', encoding="utf-8"
        )
        if system == "windows":
            runtime_libraries = install / "bin"
            runtime_libraries.mkdir()
            for name in verify_native_step.REQUIRED_LIBRARIES:
                content = name.encode("ascii")
                (libraries / f"{name}.lib").write_bytes(content)
                (runtime_libraries / f"{name}.dll").write_bytes(content)
            (runtime_libraries / "TKDE.dll").write_bytes(b"transitive-runtime")
        else:
            suffix = ".dylib" if system == "darwin" else ".so"
            for name in verify_native_step.REQUIRED_LIBRARIES:
                (libraries / f"lib{name}{suffix}").write_bytes(name.encode("ascii"))
            (libraries / f"libTKDE{suffix}").write_bytes(b"transitive-runtime")

        worker = root / (
            "partprobe-geometry-worker.exe"
            if system == "windows"
            else "partprobe-geometry-worker"
        )
        worker.write_bytes(b"verified-native-worker")
        if system != "windows":
            worker.chmod(0o755)
        build_manifest = root / "partprobe-occt-build-manifest.json"
        build_manifest.write_text(
            json.dumps(
                {
                    "schema_version": 1,
                    "occt_version": verify_native_step.EXPECTED_OCCT_VERSION,
                    "occt_commit": verify_native_step.EXPECTED_OCCT_COMMIT,
                    "occt_tree": verify_native_step.EXPECTED_OCCT_TREE,
                    "platform": system,
                    "machine": machine,
                    "build_type": "Release",
                    "library_type": "Shared",
                }
            ),
            encoding="utf-8",
        )
        return install, worker, build_manifest

    def test_runtime_is_assembled_and_verified_without_absolute_source_paths(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            install, worker, build_manifest = self.create_inputs(root)
            output = root / "runtime"

            manifest = assemble_native_runtime.assemble_runtime(
                install, worker, build_manifest, output
            )
            verified = assemble_native_runtime.verify_runtime(output)

            self.assertEqual(verified, manifest)
            self.assertEqual(
                manifest["configuration"]["PARTPROBE_GEOMETRY_WORKER"],
                f"bin/{worker.name}",
            )
            self.assertEqual(manifest["configuration"]["PARTPROBE_OCCT_ROOT"], ".")
            self.assertIn("TKDE", manifest["libraries"])
            serialized = json.dumps(manifest)
            self.assertNotIn(str(root), serialized)
            self.assertTrue((output / "bin" / worker.name).is_file())

    def test_runtime_verification_detects_worker_tampering(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            install, worker, build_manifest = self.create_inputs(root)
            output = root / "runtime"
            assemble_native_runtime.assemble_runtime(
                install, worker, build_manifest, output
            )

            (output / "bin" / worker.name).write_bytes(b"replaced-worker")

            with self.assertRaises(ValueError):
                assemble_native_runtime.verify_runtime(output)

    def test_runtime_assembly_never_overwrites_existing_output(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            install, worker, build_manifest = self.create_inputs(root)
            output = root / "runtime"
            output.mkdir()

            with self.assertRaises(ValueError):
                assemble_native_runtime.assemble_runtime(
                    install, worker, build_manifest, output
                )

    def test_runtime_manifest_rejects_traversal(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            install, worker, build_manifest = self.create_inputs(root)
            output = root / "runtime"
            assemble_native_runtime.assemble_runtime(
                install, worker, build_manifest, output
            )
            manifest_path = output / assemble_native_runtime.MANIFEST_NAME
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            manifest["configuration"]["PARTPROBE_GEOMETRY_WORKER"] = (
                "/unreviewed/worker"
            )
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            with self.assertRaises(ValueError):
                assemble_native_runtime.verify_runtime(output)

            manifest["worker"]["path"] = "../partprobe-geometry-worker"
            manifest["configuration"]["PARTPROBE_GEOMETRY_WORKER"] = manifest[
                "worker"
            ]["path"]
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

            with self.assertRaises(ValueError):
                assemble_native_runtime.verify_runtime(output)

    def test_windows_runtime_library_comes_from_bin(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            binaries = root / "bin"
            binaries.mkdir()
            expected = binaries / "TKernel.dll"
            expected.write_bytes(b"runtime")

            self.assertEqual(
                assemble_native_runtime.runtime_library_sources(
                    root, "TKernel", "windows"
                ),
                [expected],
            )


class VerifyNativeRuntimeLinksTests(unittest.TestCase):
    def test_linux_link_evidence_accepts_runtime_occt_and_system_dependencies(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            library = root / "libTKMath.so.8"
            library.write_bytes(b"runtime")
            output = (
                f"libTKMath.so.8 => {library} (0x00000001)\n"
                "libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6 (0x00000002)\n"
                "/lib64/ld-linux-x86-64.so.2 (0x00000003)\n"
            )

            dependencies = verify_native_runtime_links.parse_ldd_output(output, root)

            self.assertEqual(dependencies, {"libTKMath.so.8", "libc.so.6"})

    def test_linux_link_evidence_rejects_an_unresolved_dependency(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            with self.assertRaises(ValueError):
                verify_native_runtime_links.parse_ldd_output(
                    "libTKMath.so.8 => not found\n", Path(temporary)
                )

    def test_linux_link_evidence_rejects_occt_outside_the_runtime(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            runtime_library_directory = root / "runtime"
            runtime_library_directory.mkdir()
            external_library_directory = root / "external"
            external_library_directory.mkdir()
            external = external_library_directory / "libTKMath.so.8"
            external.write_bytes(b"external")

            with self.assertRaises(ValueError):
                verify_native_runtime_links.parse_ldd_output(
                    f"libTKMath.so.8 => {external} (0x00000001)\n",
                    runtime_library_directory,
                )


if __name__ == "__main__":
    unittest.main()
