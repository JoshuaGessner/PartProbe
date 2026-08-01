#!/usr/bin/env python3
"""Validate a pinned OCCT root and build/test the provisional native STEP seam."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import sys


EXPECTED_OCCT_VERSION = "8.0.0"
REQUIRED_LIBRARIES = (
    "TKDESTEP",
    "TKXSBase",
    "TKShHealing",
    "TKMesh",
    "TKTopAlgo",
    "TKPrim",
    "TKBRep",
    "TKGeomAlgo",
    "TKGeomBase",
    "TKG3d",
    "TKG2d",
    "TKMath",
    "TKernel",
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Validate an explicit OCCT 8.0.0 installation, then build and test "
            "PartProbe's developer-only native STEP worker."
        )
    )
    parser.add_argument(
        "--occt-root",
        type=Path,
        help="OCCT install root; defaults to PARTPROBE_OCCT_ROOT",
    )
    parser.add_argument(
        "--check-only",
        action="store_true",
        help="Validate and fingerprint the OCCT root without invoking Cargo",
    )
    return parser.parse_args()


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def resolve_library(library_dir: Path, name: str) -> Path:
    candidates = sorted(
        path
        for pattern in (f"lib{name}.*", f"{name}.*")
        for path in library_dir.glob(pattern)
        if path.is_file()
    )
    if not candidates:
        raise ValueError(f"missing required OCCT library {name} in {library_dir}")
    unversioned_names = {
        f"lib{name}.dylib",
        f"lib{name}.so",
        f"{name}.dll",
        f"{name}.lib",
    }
    unversioned = [path for path in candidates if path.name in unversioned_names]
    return (unversioned or candidates)[0].resolve()


def inspect_occt(root: Path) -> dict[str, object]:
    root = root.expanduser().resolve(strict=True)
    include_dir = root / "include" / "opencascade"
    library_dir = root / "lib"
    version_header = include_dir / "Standard_Version.hxx"
    if not include_dir.is_dir() or not library_dir.is_dir() or not version_header.is_file():
        raise ValueError(
            "OCCT root must contain include/opencascade/Standard_Version.hxx and lib"
        )
    version_match = re.search(
        r'^#define OCC_VERSION_COMPLETE "([^"]+)"$',
        version_header.read_text(encoding="utf-8"),
        re.MULTILINE,
    )
    if version_match is None or version_match.group(1) != EXPECTED_OCCT_VERSION:
        actual = version_match.group(1) if version_match else "unreadable"
        raise ValueError(
            f"expected OCCT {EXPECTED_OCCT_VERSION}, found {actual} in {version_header}"
        )
    libraries = {name: resolve_library(library_dir, name) for name in REQUIRED_LIBRARIES}
    return {
        "schema_version": 1,
        "occt_version": EXPECTED_OCCT_VERSION,
        "platform": platform.system().lower(),
        "machine": platform.machine().lower(),
        "version_header_sha256": sha256(version_header),
        "libraries": {name: sha256(path) for name, path in libraries.items()},
    }


def run(command: list[str], repository: Path, environment: dict[str, str]) -> None:
    print("+", " ".join(command), flush=True)
    subprocess.run(command, cwd=repository, env=environment, check=True)


def main() -> int:
    args = parse_args()
    configured_root = args.occt_root or (
        Path(value) if (value := os.environ.get("PARTPROBE_OCCT_ROOT")) else None
    )
    if configured_root is None:
        raise ValueError("pass --occt-root or set PARTPROBE_OCCT_ROOT")

    fingerprint = inspect_occt(configured_root)
    print(json.dumps(fingerprint, indent=2, sort_keys=True))
    if args.check_only:
        return 0

    repository = Path(__file__).resolve().parent.parent
    root = configured_root.expanduser().resolve(strict=True)
    environment = os.environ.copy()
    environment["PARTPROBE_OCCT_ROOT"] = str(root)
    loader_key = (
        "PATH"
        if os.name == "nt"
        else ("DYLD_LIBRARY_PATH" if sys.platform == "darwin" else "LD_LIBRARY_PATH")
    )
    native_library_dir = root / (
        "bin" if os.name == "nt" and (root / "bin").is_dir() else "lib"
    )
    environment[loader_key] = os.pathsep.join(
        value
        for value in (str(native_library_dir), environment.get(loader_key, ""))
        if value
    )

    run(
        [
            "cargo",
            "clippy",
            "-p",
            "partprobe-geometry-occt-adapter",
            "--all-targets",
            "--features",
            "fixture-tools",
            "--locked",
            "--",
            "-D",
            "warnings",
        ],
        repository,
        environment,
    )
    run(
        [
            "cargo",
            "clippy",
            "-p",
            "partprobe-geometry-worker",
            "--all-targets",
            "--features",
            "native-occt",
            "--locked",
            "--",
            "-D",
            "warnings",
        ],
        repository,
        environment,
    )
    run(
        [
            "cargo",
            "test",
            "-p",
            "partprobe-geometry-occt-adapter",
            "--features",
            "fixture-tools",
            "--locked",
        ],
        repository,
        environment,
    )
    run(
        [
            "cargo",
            "test",
            "-p",
            "partprobe-geometry-worker",
            "--features",
            "native-occt",
            "--test",
            "process_boundary",
            "--locked",
        ],
        repository,
        environment,
    )
    run(
        [
            "cargo",
            "build",
            "-p",
            "partprobe-geometry-worker",
            "--features",
            "native-occt",
            "--locked",
        ],
        repository,
        environment,
    )
    executable = repository / "target" / "debug" / (
        "partprobe-geometry-worker.exe" if os.name == "nt" else "partprobe-geometry-worker"
    )
    if not executable.is_file():
        raise ValueError(f"Cargo did not produce the expected worker at {executable}")
    print(
        json.dumps(
            {
                "worker": str(executable),
                "worker_sha256": sha256(executable),
                "status": "developer_native_step_seam_verified",
            },
            indent=2,
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"native STEP verification failed: {error}", file=sys.stderr)
        raise SystemExit(1) from error
