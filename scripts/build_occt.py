#!/usr/bin/env python3
"""Build the pinned developer-only OCCT profile from an existing source checkout."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import sys

from verify_native_step import (
    EXPECTED_OCCT_COMMIT,
    EXPECTED_OCCT_TREE,
    EXPECTED_OCCT_VERSION,
)


TOP_LEVEL_MODULES = (
    "ApplicationFramework",
    "DataExchange",
    "Draw",
    "FoundationClasses",
    "ModelingAlgorithms",
    "ModelingData",
    "Visualization",
)
ADDITIONAL_TOOLKITS = "TKDESTEP;TKShHealing;TKMesh"
FIXED_CMAKE_OPTIONS = (
    "-DBUILD_LIBRARY_TYPE=Shared",
    "-DBUILD_CPP_STANDARD=C++17",
    "-DCMAKE_BUILD_TYPE=Release",
    f"-DBUILD_ADDITIONAL_TOOLKITS={ADDITIONAL_TOOLKITS}",
    "-DBUILD_USE_PCH=OFF",
    "-DUSE_TBB=OFF",
    "-DUSE_FREETYPE=OFF",
    "-DUSE_XLIB=OFF",
    "-DUSE_FFMPEG=OFF",
    "-DUSE_FREEIMAGE=OFF",
    "-DUSE_OPENVR=OFF",
    "-DBUILD_DOC_Overview=OFF",
    "-DBUILD_DOC_RefMan=OFF",
    "-DBUILD_RESOURCES=OFF",
    "-DBUILD_YACCLEX=OFF",
    "-DINSTALL_TEST_CASES=OFF",
    "-DBUILD_RELEASE_DISABLE_EXCEPTIONS=ON",
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Configure, build, install, fingerprint, and verify the pinned OCCT "
            "8.0.0 developer profile without downloading source or dependencies."
        )
    )
    parser.add_argument("--source", required=True, type=Path)
    parser.add_argument("--build", required=True, type=Path)
    parser.add_argument("--install", required=True, type=Path)
    parser.add_argument("--jobs", type=int, default=max(1, os.cpu_count() or 1))
    parser.add_argument(
        "--generator",
        help="Optional explicit CMake generator; the selected generator is recorded",
    )
    parser.add_argument(
        "--architecture",
        choices=("x64",),
        help="Explicit Visual Studio generator platform; currently Windows x64 only",
    )
    parser.add_argument(
        "--allow-existing",
        action="store_true",
        help="Allow nonempty build/install directories after all path checks pass",
    )
    parser.add_argument(
        "--configure-only",
        action="store_true",
        help="Stop after configure and manifest generation",
    )
    parser.add_argument(
        "--skip-native-verification",
        action="store_true",
        help="Build/install OCCT without running PartProbe's native verifier",
    )
    return parser.parse_args()


def run_capture(command: list[str], cwd: Path | None = None) -> str:
    return subprocess.run(
        command,
        cwd=cwd,
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    ).stdout.strip()


def run(command: list[str], cwd: Path | None = None) -> None:
    print("+", " ".join(command), flush=True)
    subprocess.run(command, cwd=cwd, check=True)


def normalized_output(path: Path) -> Path:
    return path.expanduser().resolve(strict=False)


def validate_output_paths(
    source: Path,
    build: Path,
    install: Path,
    allow_existing: bool,
) -> tuple[Path, Path, Path]:
    source = source.expanduser().resolve(strict=True)
    build = normalized_output(build)
    install = normalized_output(install)
    forbidden = {Path(source.anchor), Path.home().resolve(), source}
    for label, path in (("build", build), ("install", install)):
        if path in forbidden or source in path.parents:
            raise ValueError(f"{label} directory must not be root, home, or inside source")
        if path.exists() and not path.is_dir():
            raise ValueError(f"{label} path exists and is not a directory: {path}")
        if path.is_dir() and any(path.iterdir()) and not allow_existing:
            raise ValueError(
                f"{label} directory is nonempty; pass --allow-existing to reuse it: {path}"
            )
    if build == install or build in install.parents or install in build.parents:
        raise ValueError("build and install directories must be separate and non-nested")
    return source, build, install


def validate_source(source: Path) -> tuple[str, str]:
    if not (source / ".git").exists():
        raise ValueError("OCCT source must be a Git checkout with provenance metadata")
    commit = run_capture(["git", "-C", str(source), "rev-parse", "HEAD"])
    if commit != EXPECTED_OCCT_COMMIT:
        raise ValueError(f"expected OCCT commit {EXPECTED_OCCT_COMMIT}, found {commit}")
    status = run_capture(["git", "-C", str(source), "status", "--porcelain"])
    if status:
        raise ValueError("OCCT source checkout must be clean")
    tag = run_capture(["git", "-C", str(source), "describe", "--tags", "--exact-match"])
    if tag != "V8_0_0":
        raise ValueError(f"expected exact tag V8_0_0, found {tag}")
    tree = run_capture(["git", "-C", str(source), "rev-parse", "HEAD^{tree}"])
    if tree != EXPECTED_OCCT_TREE:
        raise ValueError(f"expected OCCT tree {EXPECTED_OCCT_TREE}, found {tree}")
    return commit, tree


def configure_command(
    source: Path,
    build: Path,
    install: Path,
    generator: str | None,
    architecture: str | None,
) -> list[str]:
    command = ["cmake", "-S", str(source), "-B", str(build)]
    if generator:
        command.extend(["-G", generator])
    if architecture:
        command.extend(["-A", architecture])
    command.extend(FIXED_CMAKE_OPTIONS)
    command.extend(f"-DBUILD_MODULE_{module}=OFF" for module in TOP_LEVEL_MODULES)
    command.append(f"-DINSTALL_DIR={install}")
    return command


def cmake_cache_value(cache: Path, key: str) -> str:
    prefix = f"{key}:"
    for line in cache.read_text(encoding="utf-8").splitlines():
        if line.startswith(prefix):
            return line.split("=", 1)[1]
    raise ValueError(f"configured CMake cache does not contain {key}")


def cmake_compiler_value(build: Path, cache: Path, language: str) -> str:
    key = f"CMAKE_{language}_COMPILER"
    try:
        return cmake_cache_value(cache, key)
    except ValueError:
        candidates = sorted(
            (build / "CMakeFiles").glob(f"*/CMake{language}Compiler.cmake")
        )
        if len(candidates) != 1:
            raise ValueError(
                f"configured CMake build does not expose one {key} definition"
            ) from None
        content = candidates[0].read_text(encoding="utf-8")
        match = re.search(
            rf'^set\({re.escape(key)} "([^"]+)"\)$',
            content,
            flags=re.MULTILINE,
        )
        if match is None or not match.group(1).strip():
            raise ValueError(
                f"configured CMake compiler metadata does not contain {key}"
            ) from None
        return match.group(1)


def write_manifest(
    build: Path,
    commit: str,
    tree: str,
    jobs: int,
) -> Path:
    cache = build / "CMakeCache.txt"
    manifest = {
        "schema_version": 1,
        "occt_version": EXPECTED_OCCT_VERSION,
        "occt_commit": commit,
        "occt_tree": tree,
        "platform": platform.system().lower(),
        "machine": platform.machine().lower(),
        "cmake_version": run_capture(["cmake", "--version"]).splitlines()[0],
        "cmake_generator": cmake_cache_value(cache, "CMAKE_GENERATOR"),
        "cmake_generator_platform": cmake_cache_value(cache, "CMAKE_GENERATOR_PLATFORM"),
        "c_compiler": cmake_compiler_value(build, cache, "C"),
        "cxx_compiler": cmake_compiler_value(build, cache, "CXX"),
        "build_jobs": jobs,
        "build_type": "Release",
        "library_type": "Shared",
        "additional_toolkits": ADDITIONAL_TOOLKITS.split(";"),
        "fixed_cmake_options": list(FIXED_CMAKE_OPTIONS),
        "disabled_top_level_modules": list(TOP_LEVEL_MODULES),
    }
    path = build / "partprobe-occt-build-manifest.json"
    path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return path


def main() -> int:
    args = parse_args()
    if args.jobs <= 0:
        raise ValueError("--jobs must be positive")
    system = platform.system().lower()
    if system == "windows" and args.architecture != "x64":
        raise ValueError("Windows construction requires explicit --architecture x64")
    if system != "windows" and args.architecture is not None:
        raise ValueError("--architecture is supported only for Windows construction")
    source, build, install = validate_output_paths(
        args.source,
        args.build,
        args.install,
        args.allow_existing,
    )
    commit, tree = validate_source(source)
    build.mkdir(parents=True, exist_ok=True)
    install.mkdir(parents=True, exist_ok=True)
    run(configure_command(source, build, install, args.generator, args.architecture))
    manifest = write_manifest(build, commit, tree, args.jobs)
    print(f"wrote {manifest}")
    if args.configure_only:
        return 0

    run(["cmake", "--build", str(build), "--config", "Release", "--parallel", str(args.jobs)])
    run(["cmake", "--install", str(build), "--config", "Release"])
    if not args.skip_native_verification:
        repository = Path(__file__).resolve().parent.parent
        run(
            [
                sys.executable,
                str(repository / "scripts" / "verify_native_step.py"),
                "--occt-root",
                str(install),
            ],
            repository,
        )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"OCCT construction failed: {error}", file=sys.stderr)
        raise SystemExit(1) from error
