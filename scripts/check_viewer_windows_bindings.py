#!/usr/bin/env python3
"""Check resolved D3D12 binding identity without compiling or disabling a backend."""

from __future__ import annotations

import json
from pathlib import Path
import subprocess
import sys


def validate_windows_bindings(metadata: dict) -> str:
    """Require the allocator and renderer to exchange the same Windows crate types."""
    packages = {package["id"]: package for package in metadata["packages"]}
    nodes = {node["id"]: node for node in metadata["resolve"]["nodes"]}
    bindings = []
    for name in ("wgpu-hal", "gpu-allocator"):
        matches = [package for package in packages.values() if package["name"] == name]
        if len(matches) != 1:
            raise ValueError(f"expected exactly one resolved {name} package")
        node = nodes[matches[0]["id"]]
        windows = [dependency["pkg"] for dependency in node["deps"] if dependency["name"] == "windows"]
        if len(windows) != 1 or packages[windows[0]]["name"] != "windows":
            raise ValueError(f"expected exactly one resolved windows binding for {name}")
        bindings.append(windows[0])
    if bindings[0] != bindings[1]:
        raise ValueError("wgpu-hal and gpu-allocator resolve different Windows bindings; review Cargo.lock")
    return packages[bindings[0]]["version"]


def main() -> int:
    repository = Path(__file__).resolve().parents[1]
    try:
        result = subprocess.run(
            ["cargo", "metadata", "--format-version", "1", "--locked", "--offline",
             "--filter-platform", "x86_64-pc-windows-msvc"],
            cwd=repository, check=True, capture_output=True, text=True,
        )
        version = validate_windows_bindings(json.loads(result.stdout))
    except (OSError, subprocess.CalledProcessError, ValueError, KeyError, TypeError) as error:
        print(f"Viewer Windows binding validation failed: {error}", file=sys.stderr)
        return 1
    print(f"Viewer D3D12 bindings match: windows {version} (allocator and wgpu-hal)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
