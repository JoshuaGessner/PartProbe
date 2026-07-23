#!/usr/bin/env python3
"""Print SHA-256 digests for committed fixture files."""

import hashlib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
for path in sorted((ROOT / "fixtures" / "models").iterdir()):
    if path.is_file():
        print(f"{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.relative_to(ROOT)}")
