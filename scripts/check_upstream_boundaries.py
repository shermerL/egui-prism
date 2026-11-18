#!/usr/bin/env python3
"""Verify the comparison really uses unpatched official egui, then run it."""
import json
from pathlib import Path
import subprocess
import tomllib

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "tests/upstream-boundaries/Cargo.toml"
CHECKSUM = "bec9144f49b7a28b5f395d3147e325b4fe3e77ba686eac11d1bdf0a7da3f375f"


def main():
    metadata = json.loads(subprocess.check_output([
        "cargo", "metadata", "--locked", "--format-version", "1", "--manifest-path", str(MANIFEST),
    ], text=True, cwd=ROOT))
    egui = [p for p in metadata["packages"] if p["name"] == "egui"]
    if len(egui) != 1 or egui[0]["version"] != "0.37.0" or not (egui[0]["source"] or "").startswith("registry+"):
        raise ValueError("comparison must use one official registry egui 0.37.0, not the controlled patch")
    lock = tomllib.loads(MANIFEST.with_name("Cargo.lock").read_text())
    pinned = [p for p in lock["package"] if p["name"] == "egui"]
    if len(pinned) != 1 or pinned[0].get("checksum") != CHECKSUM:
        raise ValueError("comparison archive checksum changed")
    subprocess.run(["cargo", "fmt", "--manifest-path", str(MANIFEST), "--", "--check"], cwd=ROOT, check=True)
    subprocess.run([
        "cargo", "test", "--locked", "--manifest-path", str(MANIFEST),
        "--target-dir", str(ROOT / "target/upstream-boundaries"),
    ], cwd=ROOT, check=True)
    print("Verified official unpatched egui and batch/split-frame comparison")


if __name__ == "__main__":
    main()
