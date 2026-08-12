#!/usr/bin/env python3
"""Fail closed when a verified Linux native runtime has unresolved or escaped OCCT links."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import platform
import subprocess
import sys

import assemble_native_runtime


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Verify a PartProbe Linux developer runtime, inspect every native binary with "
            "ldd, and reject unresolved or external OCCT library bindings."
        )
    )
    parser.add_argument("--runtime-root", required=True, type=Path)
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


def verify_linux_runtime_links(runtime_root: Path) -> dict[str, object]:
    if platform.system().lower() != "linux":
        raise ValueError("Linux native runtime link inspection requires a Linux host")
    root = runtime_root.expanduser().resolve(strict=True)
    manifest = assemble_native_runtime.verify_runtime(root)
    worker_entry = manifest.get("worker")
    if not isinstance(worker_entry, dict) or not isinstance(worker_entry.get("path"), str):
        raise ValueError("verified runtime worker entry is invalid")
    library_directory = (root / "lib").resolve(strict=True)
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


def main() -> int:
    result = verify_linux_runtime_links(parse_args().runtime_root)
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"Linux native runtime link verification failed: {error}", file=sys.stderr)
        raise SystemExit(1) from error
