#!/usr/bin/env python3
"""Fail closed when a verified native runtime has incomplete or escaped OCCT links."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import sys

import assemble_native_runtime


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Verify a PartProbe developer runtime and reject unresolved, incomplete, or "
            "external OCCT library bindings. Linux uses ldd; Windows requires an explicit "
            "dumpbin-compatible dependency tool."
        )
    )
    parser.add_argument("--runtime-root", required=True, type=Path)
    parser.add_argument(
        "--dependency-tool",
        type=Path,
        help="Explicit dumpbin-compatible tool path required on Windows",
    )
    return parser.parse_args()


def parse_ldd_output(output: str, runtime_library_directory: Path) -> set[str]:
    runtime_library_directory = runtime_library_directory.resolve(strict=True)
    dependencies: set[str] = set()
    for raw_line in output.splitlines():
        line = raw_line.strip()
        if not line:
            continue
        if "not found" in line:
            raise ValueError("native runtime contains an unresolved dynamic dependency")
        if "=>" not in line:
            if line == "statically linked":
                raise ValueError("native runtime binary is unexpectedly statically linked")
            continue
        name, resolution = (part.strip() for part in line.split("=>", 1))
        resolved_value = resolution.rsplit("(", 1)[0].strip()
        if not name or not resolved_value or resolved_value == "not found":
            raise ValueError("native runtime has malformed ldd dependency evidence")
        dependencies.add(name)
        if name.startswith("libTK"):
            resolved = Path(resolved_value).resolve(strict=True)
            if resolved.parent != runtime_library_directory:
                raise ValueError(
                    f"OCCT dependency {name} resolves outside the verified runtime"
                )
    return dependencies


def inspect_binary(binary: Path, runtime_library_directory: Path) -> set[str]:
    result = subprocess.run(
        ["ldd", str(binary)],
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env={"LD_LIBRARY_PATH": str(runtime_library_directory), "PATH": "/usr/bin:/bin"},
    )
    return parse_ldd_output(result.stdout, runtime_library_directory)


def parse_dumpbin_dependents(output: str) -> set[str]:
    dependencies: set[str] = set()
    in_dependency_list = False
    saw_dependency_header = False
    for raw_line in output.splitlines():
        line = raw_line.strip()
        if line in {
            "Image has the following dependencies:",
            "Image has the following delay load dependencies:",
        }:
            in_dependency_list = True
            saw_dependency_header = True
            continue
        if not in_dependency_list:
            continue
        if not line:
            continue
        if line == "Summary" or line.endswith(" imports"):
            in_dependency_list = False
            continue
        if (
            line.lower().endswith(".dll")
            and "/" not in line
            and "\\" not in line
            and Path(line).name == line
        ):
            dependencies.add(line)
        else:
            raise ValueError("native runtime has malformed dumpbin dependency evidence")
    if not saw_dependency_header or not dependencies:
        raise ValueError("native runtime exposes no dumpbin dependency evidence")
    return dependencies


def parse_dumpbin_machine(output: str) -> str:
    machines = {
        match.group(1).lower()
        for line in output.splitlines()
        if (match := re.match(r"^\s*([0-9a-fA-F]+)\s+machine\s+\(([^)]+)\)\s*$", line))
    }
    if machines != {"8664"}:
        raise ValueError("native Windows runtime binary is not unambiguously x64")
    return "x86_64"


def inspect_windows_binary(binary: Path, dependency_tool: Path) -> set[str]:
    environment = os.environ.copy()
    environment["PATH"] = str(dependency_tool.parent)
    header_result = subprocess.run(
        [str(dependency_tool), "/HEADERS", "/NOLOGO", str(binary)],
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env=environment,
    )
    dependency_result = subprocess.run(
        [str(dependency_tool), "/DEPENDENTS", "/NOLOGO", str(binary)],
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env=environment,
    )
    parse_dumpbin_machine(header_result.stdout)
    return parse_dumpbin_dependents(dependency_result.stdout)


def validate_windows_occt_imports(
    dependencies: set[str],
    available_names: set[str],
) -> list[str]:
    occt_dependencies = sorted(
        name
        for name in dependencies
        if name.lower().startswith("tk") and name.lower().endswith(".dll")
    )
    if not occt_dependencies:
        raise ValueError("native worker/runtime exposes no OCCT dynamic-link evidence")
    missing = sorted(
        name for name in occt_dependencies if name.lower() not in available_names
    )
    if missing:
        raise ValueError("Windows OCCT import closure escapes the verified runtime")
    return occt_dependencies


def verify_linux_runtime_links(runtime_root: Path) -> dict[str, object]:
    if platform.system().lower() != "linux":
        raise ValueError("Linux native runtime link inspection requires a Linux host")
    root = runtime_root.expanduser().resolve(strict=True)
    manifest = assemble_native_runtime.verify_runtime(root)
    worker_entry = manifest.get("worker")
    if not isinstance(worker_entry, dict) or not isinstance(worker_entry.get("path"), str):
        raise ValueError("verified runtime worker entry is invalid")
    library_directory = (
        root / assemble_native_runtime.runtime_library_directory_name("linux")
    ).resolve(strict=True)
    binaries = [root / assemble_native_runtime.relative_artifact_path(worker_entry["path"])]
    binaries.extend(
        path
        for path in sorted(library_directory.glob("libTK*.so*"))
        if path.is_file() and not path.is_symlink()
    )
    if len(binaries) == 1:
        raise ValueError("verified Linux runtime contains no inspectable OCCT libraries")

    dependencies: set[str] = set()
    for binary in binaries:
        dependencies.update(inspect_binary(binary, library_directory))
    occt_dependencies = sorted(name for name in dependencies if name.startswith("libTK"))
    if not occt_dependencies:
        raise ValueError("native worker/runtime exposes no OCCT dynamic-link evidence")
    return {
        "status": "linux_native_runtime_links_verified",
        "inspected_binary_count": len(binaries),
        "occt_dependency_count": len(occt_dependencies),
        "occt_dependencies": occt_dependencies,
        "system_dependency_count": len(dependencies) - len(occt_dependencies),
    }


def verify_windows_runtime_links(
    runtime_root: Path,
    dependency_tool: Path | None,
) -> dict[str, object]:
    if platform.system().lower() != "windows":
        raise ValueError("Windows native runtime link inspection requires a Windows host")
    if dependency_tool is None:
        raise ValueError("Windows link inspection requires --dependency-tool")
    dependency_tool = dependency_tool.expanduser().resolve(strict=True)
    if not dependency_tool.is_file() or dependency_tool.name.lower() != "dumpbin.exe":
        raise ValueError("Windows dependency tool must be an explicit dumpbin.exe file")

    root = runtime_root.expanduser().resolve(strict=True)
    manifest = assemble_native_runtime.verify_runtime(root)
    worker_entry = manifest.get("worker")
    if not isinstance(worker_entry, dict) or not isinstance(worker_entry.get("path"), str):
        raise ValueError("verified runtime worker entry is invalid")
    library_directory = (
        root / assemble_native_runtime.runtime_library_directory_name("windows")
    ).resolve(strict=True)
    libraries = sorted(library_directory.glob("TK*.dll"))
    if not libraries:
        raise ValueError("verified Windows runtime contains no inspectable OCCT libraries")
    available: dict[str, Path] = {}
    for library in libraries:
        key = library.name.lower()
        if key in available or not library.is_file():
            raise ValueError("verified Windows runtime has duplicate or invalid OCCT libraries")
        available[key] = library.resolve(strict=True)

    binaries = [root / assemble_native_runtime.relative_artifact_path(worker_entry["path"])]
    binaries.extend(libraries)
    dependencies: set[str] = set()
    for binary in binaries:
        dependencies.update(inspect_windows_binary(binary, dependency_tool))
    occt_dependencies = validate_windows_occt_imports(
        dependencies,
        set(available),
    )
    return {
        "status": "windows_native_runtime_links_verified",
        "inspected_binary_count": len(binaries),
        "occt_dependency_count": len(occt_dependencies),
        "occt_dependencies": occt_dependencies,
        "system_dependency_count": len(dependencies) - len(occt_dependencies),
    }


def main() -> int:
    args = parse_args()
    system = platform.system().lower()
    if system == "linux":
        if args.dependency_tool is not None:
            raise ValueError("Linux link inspection does not accept --dependency-tool")
        result = verify_linux_runtime_links(args.runtime_root)
    elif system == "windows":
        result = verify_windows_runtime_links(args.runtime_root, args.dependency_tool)
    else:
        raise ValueError("native runtime link inspection supports Linux and Windows only")
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"native runtime link verification failed: {error}", file=sys.stderr)
        raise SystemExit(1) from error
