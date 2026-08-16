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
import smoke_linux_desktop_package  # noqa: E402
import verify_native_step  # noqa: E402
import verify_native_runtime_links  # noqa: E402


class BuildOcctTests(unittest.TestCase):
    def test_configure_command_contains_the_reviewed_minimal_profile(self) -> None:
        source = Path("/source")
        build = Path("/build")
        install = Path("/install")
        command = build_occt.configure_command(source, build, install, "Ninja", None)

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
        self.assertIn("-DINSTALL_DIR_BIN=bin", command)
        self.assertIn("-DINSTALL_DIR_LIB=lib", command)
        self.assertIn("-DINSTALL_DIR_INCLUDE=include/opencascade", command)
        self.assertEqual(command[-1], f"-DINSTALL_DIR={install}")

    def test_windows_configure_command_pins_x64_generator_platform(self) -> None:
        command = build_occt.configure_command(
            Path("/source"),
            Path("/build"),
            Path("/install"),
            "Visual Studio 17 2022",
            "x64",
        )

        self.assertEqual(command[7:9], ["-A", "x64"])

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
                "CMAKE_GENERATOR_PLATFORM:INTERNAL=\n"
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
            self.assertEqual(manifest["cmake_generator_platform"], "")
            self.assertEqual(manifest["cxx_compiler"], "/tool/c++")
            self.assertEqual(manifest["additional_toolkits"], ["TKDESTEP", "TKShHealing", "TKMesh"])

    def test_manifest_reads_visual_studio_compilers_from_generated_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            build = Path(temporary)
            (build / "CMakeCache.txt").write_text(
                "CMAKE_GENERATOR:INTERNAL=Visual Studio 17 2022\n"
                "CMAKE_GENERATOR_PLATFORM:INTERNAL=x64\n",
                encoding="utf-8",
            )
            compiler_metadata = build / "CMakeFiles" / "4.3.4"
            compiler_metadata.mkdir(parents=True)
            (compiler_metadata / "CMakeCCompiler.cmake").write_text(
                'set(CMAKE_C_COMPILER "C:/tool/cl.exe")\n', encoding="utf-8"
            )
            (compiler_metadata / "CMakeCXXCompiler.cmake").write_text(
                'set(CMAKE_CXX_COMPILER "C:/tool/cl.exe")\n', encoding="utf-8"
            )
            with mock.patch.object(
                build_occt, "run_capture", return_value="cmake version 4.3.4"
            ):
                manifest_path = build_occt.write_manifest(
                    build, build_occt.EXPECTED_OCCT_COMMIT, "tree", 2
                )

            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            self.assertEqual(manifest["c_compiler"], "C:/tool/cl.exe")
            self.assertEqual(manifest["cxx_compiler"], "C:/tool/cl.exe")
            self.assertEqual(manifest["cmake_generator_platform"], "x64")


class VerifyNativeStepTests(unittest.TestCase):
    def create_install(
        self,
        root: Path,
        version: str = "8.0.0",
        *,
        system: str | None = None,
    ) -> None:
        system = system or verify_native_step.platform.system().lower()
        include = root / "include" / "opencascade"
        libraries = root / "lib"
        include.mkdir(parents=True)
        libraries.mkdir()
        (include / "Standard_Version.hxx").write_text(
            f'#define OCC_VERSION_COMPLETE "{version}"\n', encoding="utf-8"
        )
        for name in verify_native_step.REQUIRED_LIBRARIES:
            if system == "windows":
                binaries = root / "bin"
                binaries.mkdir(exist_ok=True)
                (libraries / f"{name}.lib").write_bytes(f"import-{name}".encode("ascii"))
                (binaries / f"{name}.dll").write_bytes(f"runtime-{name}".encode("ascii"))
            else:
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

    def test_windows_fingerprint_hashes_runtime_dll_not_import_library(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.create_install(root, system="windows")

            fingerprint = verify_native_step.inspect_occt(
                root, "windows", "x86_64"
            )

            self.assertEqual(
                fingerprint["libraries"]["TKernel"],
                verify_native_step.sha256(root / "bin" / "TKernel.dll"),
            )
            self.assertNotEqual(
                fingerprint["libraries"]["TKernel"],
                verify_native_step.sha256(root / "lib" / "TKernel.lib"),
            )

    def test_windows_build_fingerprint_requires_import_libraries(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.create_install(root, system="windows")
            (root / "lib" / "TKernel.lib").unlink()

            with self.assertRaises(ValueError):
                verify_native_step.inspect_occt(root, "windows", "x86_64")


class AssembleNativeRuntimeTests(unittest.TestCase):
    def create_inputs(
        self,
        root: Path,
        *,
        system: str | None = None,
        machine: str | None = None,
    ) -> tuple[Path, Path, Path]:
        host_system, host_machine = assemble_native_runtime.host_identity()
        system = system or host_system
        machine = machine or host_machine
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
                    "cmake_generator_platform": "x64" if system == "windows" else "",
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

    def test_windows_runtime_assembly_keeps_dlls_beside_worker(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            install, worker, build_manifest = self.create_inputs(
                root, system="windows", machine="x86_64"
            )
            worker.chmod(0o755)
            output = root / "runtime"

            with mock.patch.object(
                assemble_native_runtime,
                "host_identity",
                return_value=("windows", "x86_64"),
            ):
                manifest = assemble_native_runtime.assemble_runtime(
                    install, worker, build_manifest, output
                )
                assemble_native_runtime.verify_runtime(output)

            self.assertTrue((output / "bin" / "TKernel.dll").is_file())
            self.assertFalse((output / "lib" / "TKernel.dll").exists())
            self.assertEqual(
                manifest["occt_install_fingerprint"]["libraries"]["TKernel"],
                verify_native_step.sha256(install / "bin" / "TKernel.dll"),
            )

    @unittest.skipIf(sys.platform == "win32", "fixture requires Unix symlinks")
    def test_package_materialization_replaces_safe_aliases_and_regenerates_manifest(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            install, worker, build_manifest = self.create_inputs(root)
            if sys.platform == "darwin":
                unversioned = install / "lib" / "libTKernel.dylib"
                versioned = install / "lib" / "libTKernel.8.0.0.dylib"
            else:
                unversioned = install / "lib" / "libTKernel.so"
                versioned = install / "lib" / "libTKernel.so.8.0.0"
            unversioned.rename(versioned)
            unversioned.symlink_to(versioned.name)
            assembled = root / "assembled-runtime"
            packaged = root / "packaged-runtime"

            original = assemble_native_runtime.assemble_runtime(
                install, worker, build_manifest, assembled
            )
            materialized = assemble_native_runtime.materialize_runtime_for_package(
                assembled, packaged
            )

            assemble_native_runtime.verify_runtime(packaged)
            self.assertTrue(
                any(
                    entry["type"] == "symlink"
                    for entry in original["libraries"]["TKernel"]
                )
            )
            self.assertTrue(
                all(
                    entry["type"] == "file"
                    for entry in materialized["libraries"]["TKernel"]
                )
            )
            self.assertTrue(
                all(
                    not (packaged / entry["path"]).is_symlink()
                    for entry in materialized["libraries"]["TKernel"]
                )
            )
            self.assertTrue(
                any(
                    (assembled / entry["path"]).is_symlink()
                    for entry in original["libraries"]["TKernel"]
                )
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

    def test_windows_link_evidence_parses_declared_dependencies(self) -> None:
        output = """
Dump of file partprobe-geometry-worker.exe

FILE HEADER VALUES
            8664 machine (x64)

File Type: EXECUTABLE IMAGE

  Image has the following dependencies:

    TKDESTEP.dll
    KERNEL32.dll

  Summary
"""

        self.assertEqual(
            verify_native_runtime_links.parse_dumpbin_dependents(output),
            {"TKDESTEP.dll", "KERNEL32.dll"},
        )
        self.assertEqual(
            verify_native_runtime_links.parse_dumpbin_machine(output),
            "x86_64",
        )

    def test_windows_link_evidence_rejects_wrong_or_missing_machine(self) -> None:
        for output in ("14C machine (x86)\n", "no machine evidence\n"):
            with self.subTest(output=output):
                with self.assertRaises(ValueError):
                    verify_native_runtime_links.parse_dumpbin_machine(output)

    def test_windows_link_evidence_rejects_missing_dependency_header(self) -> None:
        with self.assertRaises(ValueError):
            verify_native_runtime_links.parse_dumpbin_dependents("KERNEL32.dll\n")

    def test_windows_link_evidence_rejects_malformed_dependency_lines(self) -> None:
        with self.assertRaises(ValueError):
            verify_native_runtime_links.parse_dumpbin_dependents(
                "Image has the following dependencies:\n"
                "  C:\\unreviewed\\TKMath.dll\n"
            )

    def test_windows_binary_inspection_keeps_headers_and_dependencies_separate(
        self,
    ) -> None:
        header_result = mock.Mock(stdout="8664 machine (x64)\n", stderr="")
        dependency_result = mock.Mock(
            stdout=(
                "Image has the following dependencies:\n"
                "  TKMath.dll\n"
                "  KERNEL32.dll\n"
                "Summary\n"
            ),
            stderr="",
        )
        with mock.patch.object(
            verify_native_runtime_links.subprocess,
            "run",
            side_effect=[header_result, dependency_result],
        ) as run:
            dependencies = verify_native_runtime_links.inspect_windows_binary(
                Path("worker.exe"), Path("dumpbin.exe")
            )

        self.assertEqual(dependencies, {"TKMath.dll", "KERNEL32.dll"})
        self.assertEqual(run.call_count, 2)
        self.assertEqual(run.call_args_list[0].args[0][1:3], ["/HEADERS", "/NOLOGO"])
        self.assertEqual(
            run.call_args_list[1].args[0][1:3], ["/DEPENDENTS", "/NOLOGO"]
        )

    def test_windows_link_evidence_requires_the_complete_internal_occt_closure(self) -> None:
        with self.assertRaises(ValueError):
            verify_native_runtime_links.validate_windows_occt_imports(
                {"TKMath.dll", "KERNEL32.dll"},
                {"tkernel.dll"},
            )

    def test_windows_link_evidence_requires_an_occt_import(self) -> None:
        with self.assertRaises(ValueError):
            verify_native_runtime_links.validate_windows_occt_imports(
                {"KERNEL32.dll"},
                {"tkernel.dll"},
            )


class LinuxDesktopPackageSmokeTests(unittest.TestCase):
    def test_portal_accessibility_contract_matches_hosted_surface(self) -> None:
        self.assertEqual(
            smoke_linux_desktop_package.PORTAL_CHOOSER_NAME,
            "File Chooser Widget",
        )
        self.assertEqual(smoke_linux_desktop_package.PORTAL_ACCEPT_LABEL, "Select")
        self.assertEqual(
            smoke_linux_desktop_package.PORTAL_FILES_LABEL,
            "Files",
        )
        self.assertEqual(
            smoke_linux_desktop_package.PORTAL_WINDOW_TITLE,
            "Select STEP model",
        )
        self.assertEqual(
            smoke_linux_desktop_package.SELECTED_SOURCE_LABEL,
            "Selected model source",
        )

    def test_dialog_dependency_pins_xdg_portal_backend(self) -> None:
        cargo_manifest = (SCRIPTS.parent / "Cargo.toml").read_text(encoding="utf-8")

        self.assertIn(
            'tauri-plugin-dialog = { version = "=2.7.2", '
            'default-features = false, features = ["xdg-portal"] }',
            cargo_manifest,
        )
        self.assertNotIn('tauri-plugin-dialog = "=2.7.2"', cargo_manifest)

    def test_workflows_start_dbus_inside_the_virtual_display(self) -> None:
        repository_root = SCRIPTS.parent
        for workflow_name in (
            "linux-desktop-package.yml",
            "linux-native-desktop-package.yml",
        ):
            with self.subTest(workflow=workflow_name):
                workflow = (
                    repository_root / ".github" / "workflows" / workflow_name
                ).read_text(encoding="utf-8")
                self.assertIn(
                    "xvfb-run --auto-servernum \\\n            dbus-run-session --",
                    workflow,
                )
                self.assertNotIn(
                    "dbus-run-session -- \\\n            xvfb-run --auto-servernum",
                    workflow,
                )
                self.assertIn("xdg-desktop-portal \\", workflow)
                self.assertIn("xdg-desktop-portal-gtk \\", workflow)

    def test_activate_accept_button_confirms_focus_before_return(self) -> None:
        accept_button = mock.sentinel.accept_button
        with (
            mock.patch.object(smoke_linux_desktop_package, "focus_window") as focus_window,
            mock.patch.object(smoke_linux_desktop_package, "focus") as focus,
            mock.patch.object(smoke_linux_desktop_package, "key") as key,
        ):
            smoke_linux_desktop_package.activate_accept_button(
                accept_button,
                "Select",
                mock.sentinel.focused,
            )

        focus_window.assert_called_once_with("Select STEP model")
        focus.assert_called_once_with(
            accept_button,
            "Select selected STEP model",
            focused_state=mock.sentinel.focused,
        )
        key.assert_called_once_with("Return")

    def test_click_accept_button_confirms_focus_before_semantic_activation(self) -> None:
        accept_button = mock.sentinel.accept_button
        with (
            mock.patch.object(smoke_linux_desktop_package, "focus_window") as focus_window,
            mock.patch.object(smoke_linux_desktop_package, "focus") as focus,
            mock.patch.object(
                smoke_linux_desktop_package,
                "activate_click_action",
            ) as activate_click_action,
        ):
            smoke_linux_desktop_package.click_accept_button(
                accept_button,
                "Select",
                mock.sentinel.focused,
            )

        focus_window.assert_called_once_with("Select STEP model")
        focus.assert_called_once_with(
            accept_button,
            "Select selected STEP model",
            focused_state=mock.sentinel.focused,
        )
        activate_click_action.assert_called_once_with(
            accept_button,
            "Select selected STEP model",
        )

    def test_activate_selected_file_confirms_focus_before_return(self) -> None:
        fixture_row = mock.sentinel.fixture_row
        with (
            mock.patch.object(smoke_linux_desktop_package, "focus_window") as focus_window,
            mock.patch.object(smoke_linux_desktop_package, "focus") as focus,
            mock.patch.object(smoke_linux_desktop_package, "key") as key,
        ):
            smoke_linux_desktop_package.activate_selected_file(
                fixture_row,
                mock.sentinel.focused,
            )

        focus_window.assert_called_once_with("Select STEP model")
        focus.assert_called_once_with(
            fixture_row,
            "Selected STEP model",
            focused_state=mock.sentinel.focused,
        )
        key.assert_called_once_with("Return")

    def test_open_location_entry_confirms_file_list_focus_before_slash(self) -> None:
        focus_anchor = mock.sentinel.focus_anchor
        with (
            mock.patch.object(smoke_linux_desktop_package, "focus_window") as focus_window,
            mock.patch.object(smoke_linux_desktop_package, "focus") as focus,
            mock.patch.object(smoke_linux_desktop_package, "key") as key,
        ):
            smoke_linux_desktop_package.open_location_entry(
                focus_anchor,
                "Portal file list",
                mock.sentinel.focused,
            )

        focus_window.assert_called_once_with("Select STEP model")
        focus.assert_called_once_with(
            focus_anchor,
            "Portal file list",
            focused_state=mock.sentinel.focused,
        )
        key.assert_called_once_with("slash")

    def test_wait_for_unique_role_node_requires_one_live_match(self) -> None:
        with (
            mock.patch.object(
                smoke_linux_desktop_package,
                "accessibility_nodes",
                return_value=[mock.sentinel.location_entry],
            ),
            mock.patch.object(
                smoke_linux_desktop_package,
                "role_name",
                return_value="text",
            ),
            mock.patch.object(
                smoke_linux_desktop_package,
                "node_has_states",
                return_value=True,
            ),
            mock.patch.object(
                smoke_linux_desktop_package.time,
                "monotonic",
                return_value=0.0,
            ),
        ):
            found = smoke_linux_desktop_package.wait_for_unique_role_node(
                mock.sentinel.pyatspi,
                {"text", "entry"},
                1.0,
                label="focused portal location entry",
                required_states=(mock.sentinel.focused,),
            )

        self.assertIs(found, mock.sentinel.location_entry)

    def test_wait_for_text_value_requires_exact_accessible_text(self) -> None:
        text = mock.Mock()
        text.characterCount = 15
        text.getText.return_value = "/fixtures/models"
        location_entry = mock.Mock()
        location_entry.queryText.return_value = text

        smoke_linux_desktop_package.wait_for_text_value(
            location_entry,
            "/fixtures/models",
            1.0,
        )

        text.getText.assert_called_once_with(0, 15)

    def test_wait_for_text_interface_value_requires_an_exact_showing_match(self) -> None:
        expected = "rectangular_prism_12x8x5.step"
        text = mock.Mock()
        text.characterCount = len(expected)
        text.getText.return_value = expected
        fixture_name = mock.Mock()
        fixture_name.queryText.return_value = text

        with (
            mock.patch.object(
                smoke_linux_desktop_package,
                "accessibility_nodes",
                return_value=[fixture_name],
            ),
            mock.patch.object(
                smoke_linux_desktop_package,
                "node_has_states",
                return_value=True,
            ) as node_has_states,
        ):
            found = (
                smoke_linux_desktop_package.wait_for_text_interface_value(
                    mock.sentinel.pyatspi,
                    expected,
                    1.0,
                    required_states=(mock.sentinel.showing,),
                )
            )

        self.assertIs(found, fixture_name)
        node_has_states.assert_called_once_with(
            fixture_name,
            (mock.sentinel.showing,),
        )

    def test_portal_directory_text_uses_one_trailing_separator(self) -> None:
        self.assertEqual(
            smoke_linux_desktop_package.canonical_portal_directory_text(
                "/fixtures/models"
            ),
            "/fixtures/models/",
        )
        self.assertEqual(
            smoke_linux_desktop_package.canonical_portal_directory_text(
                "/fixtures/models/"
            ),
            "/fixtures/models/",
        )
        self.assertEqual(
            smoke_linux_desktop_package.canonical_portal_directory_text("/"),
            "/",
        )
        with self.assertRaisesRegex(ValueError, "must be absolute"):
            smoke_linux_desktop_package.canonical_portal_directory_text(
                "fixtures/models"
            )

    def test_selection_acceptance_requires_source_summary_and_closed_picker(self) -> None:
        pyatspi = mock.Mock()
        with mock.patch.object(
            smoke_linux_desktop_package,
            "find_node",
            side_effect=[mock.sentinel.model_selected, None],
        ) as find_node:
            self.assertTrue(
                smoke_linux_desktop_package.selection_acceptance_observed(
                    pyatspi
                )
            )
        find_node.assert_any_call(
            pyatspi,
            "Selected model source",
            exact=True,
            required_states=(pyatspi.STATE_SHOWING,),
        )

        with mock.patch.object(
            smoke_linux_desktop_package,
            "find_node",
            side_effect=[mock.sentinel.model_selected, mock.sentinel.picker_open],
        ):
            self.assertFalse(
                smoke_linux_desktop_package.selection_acceptance_observed(
                    pyatspi
                )
            )

    def test_activate_click_action_requires_exact_semantic_action(self) -> None:
        actions = mock.Mock()
        actions.nActions = 2
        actions.getName.side_effect = ["press", "click"]
        actions.doAction.return_value = True
        node = mock.Mock()
        node.queryAction.return_value = actions

        smoke_linux_desktop_package.activate_click_action(node, "Open")

        actions.doAction.assert_called_once_with(1)

    def test_activate_click_action_rejects_unknown_actions(self) -> None:
        actions = mock.Mock()
        actions.nActions = 1
        actions.getName.return_value = "press"
        node = mock.Mock()
        node.queryAction.return_value = actions

        with self.assertRaisesRegex(RuntimeError, "no exact click action"):
            smoke_linux_desktop_package.activate_click_action(node, "Open")

    def test_wait_for_unique_node_rejects_ambiguous_live_controls(self) -> None:
        with (
            mock.patch.object(
                smoke_linux_desktop_package,
                "matching_nodes",
                return_value=[mock.sentinel.first, mock.sentinel.second],
            ),
            mock.patch.object(
                smoke_linux_desktop_package.time,
                "monotonic",
                return_value=0.0,
            ),
            self.assertRaisesRegex(RuntimeError, "ambiguous accessible target"),
        ):
            smoke_linux_desktop_package.wait_for_unique_node(
                mock.sentinel.pyatspi,
                "Open",
                1.0,
                {"push button"},
                exact=True,
            )

    def test_wait_for_unique_node_waits_for_one_live_control(self) -> None:
        with (
            mock.patch.object(
                smoke_linux_desktop_package,
                "matching_nodes",
                side_effect=[[], [mock.sentinel.open_button]],
            ),
            mock.patch.object(
                smoke_linux_desktop_package.time,
                "monotonic",
                side_effect=[0.0, 0.1],
            ),
            mock.patch.object(smoke_linux_desktop_package.time, "sleep"),
        ):
            found = smoke_linux_desktop_package.wait_for_unique_node(
                mock.sentinel.pyatspi,
                "Open",
                1.0,
                {"push button"},
                exact=True,
            )

        self.assertIs(found, mock.sentinel.open_button)

    def test_focus_waits_for_confirmed_accessible_focus(self) -> None:
        component = mock.Mock()
        component.grabFocus.return_value = True
        node = mock.Mock()
        node.queryComponent.return_value = component
        with mock.patch.object(
            smoke_linux_desktop_package,
            "node_has_states",
            side_effect=[False, True],
        ) as node_has_states:
            smoke_linux_desktop_package.focus(
                node,
                "Open",
                focused_state=mock.sentinel.focused,
            )

        component.grabFocus.assert_called_once_with()
        self.assertEqual(node_has_states.call_count, 2)
        node_has_states.assert_called_with(
            node,
            (mock.sentinel.focused,),
        )

    def test_focus_fails_when_accessible_focus_never_arrives(self) -> None:
        component = mock.Mock()
        component.grabFocus.return_value = True
        node = mock.Mock()
        node.queryComponent.return_value = component

        with (
            mock.patch.object(
                smoke_linux_desktop_package,
                "node_has_states",
                return_value=False,
            ),
            mock.patch.object(
                smoke_linux_desktop_package.time,
                "monotonic",
                side_effect=[0.0, 0.0, 2.0],
            ),
            self.assertRaisesRegex(RuntimeError, "did not acquire focus"),
        ):
            smoke_linux_desktop_package.focus(
                node,
                "Open",
                focused_state=mock.sentinel.focused,
            )

    def test_wait_for_node_absent_confirms_stale_control_is_removed(self) -> None:
        with (
            mock.patch.object(
                smoke_linux_desktop_package,
                "find_node",
                side_effect=[mock.sentinel.stale, None],
            ) as find_node,
            mock.patch.object(
                smoke_linux_desktop_package.time,
                "monotonic",
                side_effect=[0.0, 0.1],
            ),
        ):
            smoke_linux_desktop_package.wait_for_node_absent(
                mock.sentinel.pyatspi,
                "Picker open",
                1.0,
                {"push button"},
                exact=True,
            )

        self.assertEqual(find_node.call_count, 2)
        find_node.assert_called_with(
            mock.sentinel.pyatspi,
            "Picker open",
            {"push button"},
            exact=True,
        )

    def test_required_accessible_state_skips_hidden_duplicate(self) -> None:
        hidden = mock.Mock(name="hidden_open")
        hidden.name = "Open"
        hidden.description = ""
        hidden.getRoleName.return_value = "push button"
        hidden.getState.return_value.contains.return_value = False
        visible = mock.Mock(name="visible_open")
        visible.name = "Open"
        visible.description = ""
        visible.getRoleName.return_value = "push button"
        visible.getState.return_value.contains.return_value = True

        with mock.patch.object(
            smoke_linux_desktop_package,
            "accessibility_nodes",
            return_value=[hidden, visible],
        ):
            found = smoke_linux_desktop_package.find_node(
                mock.sentinel.pyatspi,
                "Open",
                {"push button"},
                exact=True,
                required_states=(mock.sentinel.showing, mock.sentinel.enabled),
            )

        self.assertIs(found, visible)
        self.assertEqual(hidden.getState.return_value.contains.call_count, 1)
        self.assertEqual(visible.getState.return_value.contains.call_count, 2)

    def test_required_accessible_state_fails_closed_when_unavailable(self) -> None:
        node = mock.Mock()
        node.getState.side_effect = RuntimeError("state unavailable")

        self.assertFalse(
            smoke_linux_desktop_package.node_has_states(
                node,
                (mock.sentinel.showing,),
            )
        )

    def test_select_node_uses_parent_selection_and_confirms_state(self) -> None:
        selection = mock.Mock()
        selection.selectChild.return_value = True
        selection.isChildSelected.side_effect = [False, True]
        parent = mock.Mock()
        parent.querySelection.return_value = selection
        node = mock.Mock()
        node.parent = parent
        node.getIndexInParent.return_value = 3

        smoke_linux_desktop_package.select_node(node, "fixture.step")

        selection.selectChild.assert_called_once_with(3)
        self.assertEqual(selection.isChildSelected.call_count, 2)

    def test_select_node_walks_to_selection_ancestor(self) -> None:
        selection = mock.Mock()
        selection.selectChild.return_value = True
        selection.isChildSelected.return_value = True
        table = mock.Mock()
        table.querySelection.return_value = selection
        intermediate = mock.Mock()
        intermediate.parent = table
        intermediate.getIndexInParent.return_value = 4
        intermediate.querySelection.side_effect = RuntimeError("unsupported")
        node = mock.Mock()
        node.parent = intermediate
        node.getIndexInParent.return_value = 1

        smoke_linux_desktop_package.select_node(node, "fixture.step")

        selection.selectChild.assert_called_once_with(4)

    def test_select_node_rejects_unselectable_item(self) -> None:
        parent = mock.Mock()
        parent.parent = None
        parent.querySelection.side_effect = RuntimeError("unsupported")
        node = mock.Mock()
        node.parent = parent
        node.getIndexInParent.return_value = 0

        with self.assertRaisesRegex(RuntimeError, "could not select accessible item"):
            smoke_linux_desktop_package.select_node(node, "fixture.step")

    def test_exact_accessible_match_does_not_accept_substring(self) -> None:
        picker_open = mock.Mock(name="picker_open")
        picker_open.name = "Picker open"
        picker_open.description = ""
        picker_open.getRoleName.return_value = "push button"
        open_button = mock.Mock(name="open_button")
        open_button.name = "Open"
        open_button.description = ""
        open_button.getRoleName.return_value = "push button"

        with mock.patch.object(
            smoke_linux_desktop_package,
            "accessibility_nodes",
            return_value=[picker_open, open_button],
        ):
            found = smoke_linux_desktop_package.find_node(
                mock.sentinel.pyatspi,
                "Open",
                {"push button"},
                exact=True,
            )

        self.assertIs(found, open_button)

    @mock.patch.object(smoke_linux_desktop_package.subprocess, "run")
    def test_focus_window_uses_exact_visible_title_without_coordinates(
        self, run: mock.Mock
    ) -> None:
        smoke_linux_desktop_package.focus_window("Open File")

        run.assert_called_once_with(
            [
                "xdotool",
                "search",
                "--sync",
                "--onlyvisible",
                "--name",
                "^Open\\ File$",
                "windowfocus",
                "--sync",
            ],
            check=True,
            timeout=10,
        )

    @mock.patch.object(smoke_linux_desktop_package.subprocess, "run")
    def test_focus_window_escapes_regex_metacharacters(self, run: mock.Mock) -> None:
        smoke_linux_desktop_package.focus_window("PartProbe [test]")

        self.assertEqual(run.call_args.args[0][5], r"^PartProbe\ \[test\]$")


if __name__ == "__main__":
    unittest.main()
