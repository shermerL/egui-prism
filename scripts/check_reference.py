#!/usr/bin/env python3
"""Verify the pinned shadcn source and unmodified Geist font assets."""

import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parents[1]
manifest = json.loads((root / "reference/manifest.json").read_text())
for name, expected in manifest["sha256"].items():
    actual = hashlib.sha256((root / name).read_bytes()).hexdigest()
    if actual != expected:
        raise SystemExit(f"Reference asset changed: {name}")
print(f"Verified {len(manifest['sha256'])} pinned reference assets")
