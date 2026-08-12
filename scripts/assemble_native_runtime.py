#!/usr/bin/env python3
"""Assemble and verify a self-contained developer-only native runtime."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path, PurePosixPath
import platform
import re
import shutil
import stat
import sys
import tempfile

from verify_native_step import (
    EXPECTED_OCCT_COMMIT,
    EXPECTED_OCCT_TREE,
    EXPECTED_OCCT_VERSION,
    REQUIRED_LIBRARIES,
    inspect_occt,
    sha256,
)


RUNTIME_KIND = "partprobe_developer_native_runtime"
SUPPORT_STATUS = "internal_developer_checkpoint"
MANIFEST_NAME = "partprobe-native-runtime.json"
BUILD_MANIFEST_NAME = "partprobe-occt-build-manifest.json"


def host_identity() -> tuple[str, str]:
    return platform.system().lower(), platform.machine().lower()


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Assemble or verify an explicit, developer-only PartProbe native runtime. "
            "This tool performs no downloads and never overwrites an existing output."
        )
    )
    subparsers = parser.add_subparsers(dest="command", required=True)
    assemble = subparsers.add_parser("assemble")
    assemble.add_argument("--occt-root", required=True, type=Path)
    assemble.add_argument("--worker", required=True, type=Path)
    assemble.add_argument("--build-manifest", required=True, type=Path)
    assemble.add_argument("--output", required=True, type=Path)
    materialize = subparsers.add_parser("materialize-for-package")
    materialize.add_argument("--runtime-root", required=True, type=Path)
    materialize.add_argument("--output", required=True, type=Path)
    verify = subparsers.add_parser("verify")
    verify.add_argument("--runtime-root", required=True, type=Path)
    return parser.parse_args()


def normalized_new_output(path: Path) -> Path:
    return path.expanduser().resolve(strict=False)


def path_contains(parent: Path, child: Path) -> bool:
    return child == parent or parent in child.parents


def validate_distinct_new_output(source: Path, output: Path) -> tuple[Path, Path]:
    source = source.expanduser().resolve(strict=True)
    output = normalized_new_output(output)
    if not source.is_dir():
        raise ValueError("source runtime must be a directory")
    if output.exists():
        raise ValueError(f"output already exists; refusing to overwrite it: {output}")
    if output in {Path(output.anchor), Path.home().resolve()}:
        raise ValueError("output must not be filesystem root or the user home directory")
    if path_contains(output, source) or path_contains(source, output):
        raise ValueError("output must be separate from the source runtime")
    return source, output


def validate_inputs(
    occt_root: Path,
    worker: Path,
    build_manifest: Path,
    output: Path,
) -> tuple[Path, Path, Path, Path]:
    occt_root = occt_root.expanduser().resolve(strict=True)
    worker = worker.expanduser().resolve(strict=True)
    build_manifest = build_manifest.expanduser().resolve(strict=True)
    output = normalized_new_output(output)
    if not occt_root.is_dir():
        raise ValueError("OCCT root must be a directory")
    if not worker.is_file():
        raise ValueError("worker must be a regular file")
    if os.name != "nt" and not worker.stat().st_mode & stat.S_IXUSR:
        raise ValueError("worker must be executable")
    if not build_manifest.is_file():
        raise ValueError("build manifest must be a regular file")
    if output.exists():
        raise ValueError(f"output already exists; refusing to overwrite it: {output}")
    if output in {Path(output.anchor), Path.home().resolve()}:
        raise ValueError("output must not be filesystem root or the user home directory")
    for label, source in (
        ("OCCT root", occt_root),
        ("worker", worker),
        ("build manifest", build_manifest),
    ):
        if path_contains(output, source) or path_contains(source, output):
            raise ValueError(f"output must be separate from {label}")
    return occt_root, worker, build_manifest, output


def load_json_object(path: Path, label: str) -> dict[str, object]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ValueError(f"{label} must be valid UTF-8 JSON") from error
    if not isinstance(value, dict):
        raise ValueError(f"{label} must contain a JSON object")
    return value


def validate_build_manifest(
    path: Path,
    expected_platform: str,
    expected_machine: str,
) -> dict[str, object]:
    manifest = load_json_object(path, "OCCT build manifest")
    expected = {
        "schema_version": 1,
        "occt_version": EXPECTED_OCCT_VERSION,
        "occt_commit": EXPECTED_OCCT_COMMIT,
        "occt_tree": EXPECTED_OCCT_TREE,
        "platform": expected_platform,
        "machine": expected_machine,
        "build_type": "Release",
        "library_type": "Shared",
    }
    for key, value in expected.items():
        if manifest.get(key) != value:
            raise ValueError(
                f"OCCT build manifest {key} must be {value!r}, found {manifest.get(key)!r}"
            )
    if expected_platform == "windows" and manifest.get("cmake_generator_platform") != "x64":
        raise ValueError("Windows OCCT build manifest must pin CMake generator platform x64")
    return manifest


def runtime_library_sources(root: Path, name: str, system: str) -> list[Path]:
    if system == "windows":
        candidate = root / "bin" / f"{name}.dll"
        if not candidate.is_file():
            raise ValueError(f"missing required OCCT runtime library {name}.dll")
        return [candidate]
    library_dir = root / "lib"
    if system == "darwin":
        patterns = (f"lib{name}.dylib", f"lib{name}.*.dylib")
    elif system == "linux":
        patterns = (f"lib{name}.so", f"lib{name}.so.*")
    else:
        raise ValueError(f"unsupported native runtime platform: {system}")
    sources = sorted({path for pattern in patterns for path in library_dir.glob(pattern)})
    if not sources:
        raise ValueError(f"missing required OCCT runtime library family {name}")
    for source in sources:
        if not (source.is_file() or source.is_symlink()):
            raise ValueError(f"runtime library is not a file: {source}")
        if source.is_symlink():
            target = Path(os.readlink(source))
            if target.is_absolute() or target.parent != Path("."):
                raise ValueError(f"runtime library symlink must use a local basename: {source}")
            resolved = source.resolve(strict=True)
            if resolved.parent != library_dir.resolve(strict=True):
                raise ValueError(f"runtime library symlink escapes its source directory: {source}")
    return sources


def runtime_library_families(root: Path, system: str) -> dict[str, list[Path]]:
    for name in REQUIRED_LIBRARIES:
        runtime_library_sources(root, name, system)
    if system == "windows":
        candidates = sorted((root / "bin").glob("TK*.dll"))
        name_pattern = re.compile(r"^(TK.+)\.dll$", re.IGNORECASE)
    elif system == "darwin":
        candidates = sorted((root / "lib").glob("libTK*.dylib"))
        name_pattern = re.compile(r"^lib(TK[^.]+)(?:\.|$)")
    elif system == "linux":
        candidates = sorted((root / "lib").glob("libTK*.so*"))
        name_pattern = re.compile(r"^lib(TK[^.]+)(?:\.|$)")
    else:
        raise ValueError(f"unsupported native runtime platform: {system}")
    families: dict[str, list[Path]] = {}
    for candidate in candidates:
        match = name_pattern.match(candidate.name)
        if match is None or not (candidate.is_file() or candidate.is_symlink()):
            raise ValueError(f"unexpected OCCT runtime library artifact: {candidate}")
        if candidate.is_symlink():
            target = Path(os.readlink(candidate))
            if target.is_absolute() or target.parent != Path("."):
                raise ValueError(
                    f"runtime library symlink must use a local basename: {candidate}"
                )
            if candidate.resolve(strict=True).parent != candidate.parent.resolve(strict=True):
                raise ValueError(
                    f"runtime library symlink escapes its source directory: {candidate}"
                )
        families.setdefault(match.group(1), []).append(candidate)
    missing = sorted(set(REQUIRED_LIBRARIES) - set(families))
    if missing:
        raise ValueError(f"runtime library closure lacks required families: {missing}")
    return families


def runtime_library_directory_name(system: str) -> str:
    return "bin" if system == "windows" else "lib"


def relative_artifact_path(path: str) -> Path:
    pure = PurePosixPath(path)
    if pure.is_absolute() or not pure.parts or any(part in {"", ".", ".."} for part in pure.parts):
        raise ValueError(f"manifest artifact path is unsafe: {path!r}")
    if pure.as_posix() != path:
        raise ValueError(f"manifest artifact path is not canonical: {path!r}")
    return Path(*pure.parts)


def copy_artifact(source: Path, destination: Path, root: Path) -> dict[str, object]:
    destination.parent.mkdir(parents=True, exist_ok=True)
    relative = destination.relative_to(root).as_posix()
    if source.is_symlink():
        target = os.readlink(source)
        destination.symlink_to(target)
        return {"path": relative, "type": "symlink", "target": target}
    shutil.copy2(source, destination, follow_symlinks=False)
    return {
        "path": relative,
        "type": "file",
        "size_bytes": destination.stat().st_size,
        "sha256": sha256(destination),
    }


def assemble_runtime(
    occt_root: Path,
    worker: Path,
    build_manifest: Path,
    output: Path,
) -> dict[str, object]:
    occt_root, worker, build_manifest, output = validate_inputs(
        occt_root, worker, build_manifest, output
    )
    system, machine = host_identity()
    validate_build_manifest(build_manifest, system, machine)
    install_fingerprint = inspect_occt(occt_root, system, machine)
    output.parent.mkdir(parents=True, exist_ok=True)
    staging = Path(tempfile.mkdtemp(prefix=".partprobe-native-runtime-", dir=output.parent))
    try:
        worker_name = "partprobe-geometry-worker.exe" if system == "windows" else "partprobe-geometry-worker"
        worker_entry = copy_artifact(worker, staging / "bin" / worker_name, staging)
        header = occt_root / "include" / "opencascade" / "Standard_Version.hxx"
        header_entry = copy_artifact(
            header,
            staging / "include" / "opencascade" / header.name,
            staging,
        )
        provenance_entry = copy_artifact(
            build_manifest,
            staging / "provenance" / BUILD_MANIFEST_NAME,
            staging,
        )
        libraries: dict[str, list[dict[str, object]]] = {}
        for name, sources in runtime_library_families(occt_root, system).items():
            libraries[name] = [
                copy_artifact(
                    source,
                    staging / runtime_library_directory_name(system) / source.name,
                    staging,
                )
                for source in sources
            ]
        manifest: dict[str, object] = {
            "schema_version": 1,
            "kind": RUNTIME_KIND,
            "support_status": SUPPORT_STATUS,
            "platform": system,
            "machine": machine,
            "source_policy": {
                "occt_version": EXPECTED_OCCT_VERSION,
                "occt_commit": EXPECTED_OCCT_COMMIT,
                "occt_tree": EXPECTED_OCCT_TREE,
            },
            "occt_install_fingerprint": install_fingerprint,
            "worker": worker_entry,
            "version_header": header_entry,
            "build_provenance": provenance_entry,
            "libraries": libraries,
            "configuration": {
                "PARTPROBE_GEOMETRY_WORKER": worker_entry["path"],
                "PARTPROBE_OCCT_ROOT": ".",
                "PARTPROBE_GEOMETRY_WORKSPACE": "external_directory_required",
            },
        }
        (staging / MANIFEST_NAME).write_text(
            json.dumps(manifest, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
        verify_runtime(staging)
        staging.replace(output)
        return manifest
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise


def verify_artifact(
    root: Path,
    entry: object,
    expected_paths: set[str],
) -> None:
    if not isinstance(entry, dict):
        raise ValueError("manifest artifact entry must be an object")
    path_value = entry.get("path")
    if not isinstance(path_value, str):
        raise ValueError("manifest artifact entry must contain a string path")
    relative = relative_artifact_path(path_value)
    if path_value in expected_paths:
        raise ValueError(f"manifest repeats artifact path: {path_value}")
    expected_paths.add(path_value)
    artifact = root / relative
    artifact_type = entry.get("type")
    if artifact_type == "file":
        if not artifact.is_file() or artifact.is_symlink():
            raise ValueError(f"manifested regular file is missing: {path_value}")
        if entry.get("size_bytes") != artifact.stat().st_size or entry.get("sha256") != sha256(artifact):
            raise ValueError(f"manifested file fingerprint differs: {path_value}")
    elif artifact_type == "symlink":
        if not artifact.is_symlink() or entry.get("target") != os.readlink(artifact):
            raise ValueError(f"manifested symlink differs: {path_value}")
        target = Path(os.readlink(artifact))
        if target.is_absolute() or target.parent != Path("."):
            raise ValueError(f"manifested symlink target is unsafe: {path_value}")
        resolved = artifact.resolve(strict=True)
        if resolved.parent != artifact.parent.resolve(strict=True):
            raise ValueError(f"manifested symlink escapes the runtime: {path_value}")
    else:
        raise ValueError(f"unknown artifact type for {path_value}")


def verify_runtime(runtime_root: Path) -> dict[str, object]:
    root = runtime_root.expanduser().resolve(strict=True)
    if not root.is_dir():
        raise ValueError("runtime root must be a directory")
    manifest_path = root / MANIFEST_NAME
    manifest = load_json_object(manifest_path, "native runtime manifest")
    system, machine = host_identity()
    expected = {
        "schema_version": 1,
        "kind": RUNTIME_KIND,
        "support_status": SUPPORT_STATUS,
        "platform": system,
        "machine": machine,
    }
    for key, value in expected.items():
        if manifest.get(key) != value:
            raise ValueError(f"runtime manifest {key} must be {value!r}")
    policy = manifest.get("source_policy")
    if policy != {
        "occt_version": EXPECTED_OCCT_VERSION,
        "occt_commit": EXPECTED_OCCT_COMMIT,
        "occt_tree": EXPECTED_OCCT_TREE,
    }:
        raise ValueError("runtime source policy does not match the pinned OCCT source")

    worker_entry = manifest.get("worker")
    if not isinstance(worker_entry, dict) or not isinstance(worker_entry.get("path"), str):
        raise ValueError("runtime worker entry must contain a string path")
    if manifest.get("configuration") != {
        "PARTPROBE_GEOMETRY_WORKER": worker_entry["path"],
        "PARTPROBE_OCCT_ROOT": ".",
        "PARTPROBE_GEOMETRY_WORKSPACE": "external_directory_required",
    }:
        raise ValueError("runtime launch configuration must use the reviewed relative paths")

    expected_paths = {MANIFEST_NAME}
    for key in ("worker", "version_header", "build_provenance"):
        verify_artifact(root, manifest.get(key), expected_paths)
    libraries = manifest.get("libraries")
    if not isinstance(libraries, dict) or not set(REQUIRED_LIBRARIES).issubset(libraries):
        raise ValueError("runtime manifest must contain every required OCCT library family")
    for name in sorted(libraries):
        entries = libraries[name]
        if not isinstance(entries, list) or not entries:
            raise ValueError(f"runtime library family {name} must not be empty")
        for entry in entries:
            verify_artifact(root, entry, expected_paths)

    actual_paths = {
        path.relative_to(root).as_posix()
        for path in root.rglob("*")
        if path.is_file() or path.is_symlink()
    }
    if actual_paths != expected_paths:
        extra = sorted(actual_paths - expected_paths)
        missing = sorted(expected_paths - actual_paths)
        raise ValueError(f"runtime contents differ from manifest; extra={extra}, missing={missing}")
    worker_path = root / relative_artifact_path(str(worker_entry["path"]))
    if os.name != "nt" and not worker_path.stat().st_mode & stat.S_IXUSR:
        raise ValueError("runtime worker is not executable")
    provenance_entry = manifest["build_provenance"]
    assert isinstance(provenance_entry, dict)
    provenance_path = root / relative_artifact_path(str(provenance_entry["path"]))
    validate_build_manifest(provenance_path, system, machine)
    if manifest.get("occt_install_fingerprint") != inspect_occt(
        root,
        system,
        machine,
        require_link_libraries=False,
    ):
        raise ValueError("runtime OCCT fingerprint differs from the assembled install")
    return manifest


def artifact_entries(manifest: dict[str, object]) -> list[dict[str, object]]:
    entries: list[dict[str, object]] = []
    for key in ("worker", "version_header", "build_provenance"):
        entry = manifest.get(key)
        if not isinstance(entry, dict):
            raise ValueError(f"runtime manifest {key} must be an artifact object")
        entries.append(entry)
    libraries = manifest.get("libraries")
    if not isinstance(libraries, dict):
        raise ValueError("runtime manifest libraries must be an object")
    for family_entries in libraries.values():
        if not isinstance(family_entries, list):
            raise ValueError("runtime manifest library family must be a list")
        for entry in family_entries:
            if not isinstance(entry, dict):
                raise ValueError("runtime manifest library artifact must be an object")
            entries.append(entry)
    return entries


def materialize_runtime_for_package(
    runtime_root: Path,
    output: Path,
) -> dict[str, object]:
    source, output = validate_distinct_new_output(runtime_root, output)
    manifest = verify_runtime(source)
    output.parent.mkdir(parents=True, exist_ok=True)
    staging = Path(tempfile.mkdtemp(prefix=".partprobe-packaged-runtime-", dir=output.parent))
    try:
        shutil.copytree(source, staging, dirs_exist_ok=True, symlinks=False)
        for entry in artifact_entries(manifest):
            if entry.get("type") != "symlink":
                continue
            path_value = entry.get("path")
            if not isinstance(path_value, str):
                raise ValueError("runtime artifact must contain a string path")
            artifact = staging / relative_artifact_path(path_value)
            entry.clear()
            entry.update(
                {
                    "path": path_value,
                    "type": "file",
                    "size_bytes": artifact.stat().st_size,
                    "sha256": sha256(artifact),
                }
            )
        (staging / MANIFEST_NAME).write_text(
            json.dumps(manifest, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
        verify_runtime(staging)
        staging.replace(output)
        return manifest
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise


def main() -> int:
    args = parse_args()
    if args.command == "assemble":
        manifest = assemble_runtime(
            args.occt_root,
            args.worker,
            args.build_manifest,
            args.output,
        )
        print(json.dumps(manifest, indent=2, sort_keys=True))
        print(f"assembled verified developer runtime at {args.output.expanduser().resolve()}")
    elif args.command == "materialize-for-package":
        manifest = materialize_runtime_for_package(args.runtime_root, args.output)
        print(
            json.dumps(
                {
                    "status": "packaged_native_runtime_materialized",
                    "runtime_root": str(args.output.expanduser().resolve()),
                    "artifact_count": len(artifact_entries(manifest)),
                },
                indent=2,
                sort_keys=True,
            )
        )
    else:
        manifest = verify_runtime(args.runtime_root)
        root = args.runtime_root.expanduser().resolve(strict=True)
        worker = manifest["worker"]
        assert isinstance(worker, dict)
        print(
            json.dumps(
                {
                    "status": "developer_native_runtime_verified",
                    "runtime_root": str(root),
                    "worker": str(root / relative_artifact_path(str(worker["path"]))),
                    "occt_root": str(root),
                    "workspace": "external_directory_required",
                },
                indent=2,
                sort_keys=True,
            )
        )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError) as error:
        print(f"native runtime operation failed: {error}", file=sys.stderr)
        raise SystemExit(1) from error
