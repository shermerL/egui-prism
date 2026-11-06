#!/usr/bin/env python3
"""Check one egui source and official integration type identity from a consumer root."""
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "tests/input-consumer/Cargo.toml"


def main():
    version = subprocess.check_output(["rustc", "-vV"], text=True)
    host = next(line.removeprefix("host: ") for line in version.splitlines() if line.startswith("host: "))
    metadata = json.loads(subprocess.check_output([
        "cargo", "metadata", "--locked", "--all-features", "--format-version", "1",
        "--manifest-path", str(MANIFEST), "--filter-platform", host,
    ], cwd=ROOT, text=True))
    for name in ("egui", "epaint", "egui-winit", "egui-wgpu", "egui_system_font_provider"):
        packages = [p for p in metadata["packages"] if p["name"] == name]
        if len(packages) != 1 or packages[0]["version"] != "0.37.0":
            raise ValueError(f"expected exactly one {name} 0.37.0, got {packages}")
        package = packages[0]
        if name == "egui":
            if package["source"] is not None or Path(package["manifest_path"]).resolve() != ROOT / "vendor/egui/Cargo.toml":
                raise ValueError("consumer does not use the vendored egui")
        elif not (package["source"] or "").startswith("registry+"):
            raise ValueError(f"{name} must remain an official registry package")
    subprocess.run([
        "cargo", "test", "--locked", "--all-features", "--manifest-path", str(MANIFEST),
        "--target-dir", str(ROOT / "target/input-consumer"),
    ], cwd=ROOT, check=True)
    print("Verified one egui/epaint source and compiled official integration/provider type boundaries")


if __name__ == "__main__":
    main()
