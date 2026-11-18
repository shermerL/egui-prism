#!/usr/bin/env python3
"""Build an isolated copy of the pinned settings consumer, not the parent workspace."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
EXAMPLE = ROOT / "examples/settings"
PRISM = "5f2c2280d5a4e221b6dd9c9a49d6fafe573c8bfb"
THIRD_PARTY = "2e9b53058657fa26f70e27a8d0fa8f63ab1761fc"


def main():
    readme = (EXAMPLE / "README.md").read_text()
    for source in ("src/lib.rs", "src/main.rs", "Cargo.toml", "Cargo.lock"):
        if f"]({source})" not in readme or not (EXAMPLE / source).is_file():
            raise ValueError(f"canonical source link missing or stale: {source}")
    # The exported package has its own workspace, lockfile and root patch.
    # No parent source, Cargo configuration, generated files or private tool is copied.
    # Outside the checkout: even ancestor .cargo/config.toml cannot supply setup.
    with tempfile.TemporaryDirectory(prefix="prism-settings-consumer-") as temp:
        destination = Path(temp)
        for name in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "src"):
            source = EXAMPLE / name
            if source.is_dir():
                shutil.copytree(source, destination / name)
            else:
                shutil.copy2(source, destination / name)
        env = os.environ.copy()
        env["CARGO_TARGET_DIR"] = str(ROOT / "target/settings-example")
        subprocess.run(["cargo", "fmt", "--", "--check"], cwd=destination, env=env, check=True)
        metadata = json.loads(subprocess.check_output(
            ["cargo", "metadata", "--locked", "--format-version", "1"],
            cwd=destination, env=env, text=True))
        if Path(metadata["workspace_root"]).resolve() != destination.resolve():
            raise ValueError("consumer inherited a parent workspace")
        for name in ("egui", "epaint", "egui-winit", "egui-wgpu", "egui-prism", "egui-prism-wgpu", "shadcn-egui"):
            packages = [p for p in metadata["packages"] if p["name"] == name]
            if len(packages) != 1:
                raise ValueError(f"expected one {name}, got {len(packages)}")
            source = packages[0]["source"] or ""
            if name in ("egui", "egui-prism", "egui-prism-wgpu"):
                if not source.startswith("git+https://github.com/shermerL/egui-prism?") or not source.endswith("#" + PRISM):
                    raise ValueError(f"{name} is not the fixed controlled candidate")
            elif name == "shadcn-egui":
                if not source.startswith("git+https://github.com/oetiker/shadcn-egui?") or not source.endswith("#" + THIRD_PARTY):
                    raise ValueError("third-party widget source changed")
            elif not source.startswith("registry+") or packages[0]["version"] != "0.37.0":
                raise ValueError(f"{name} is not official 0.37.0")
        for command in (
            ["cargo", "clippy", "--locked", "--all-targets", "--", "-D", "warnings"],
            ["cargo", "test", "--locked"],
            ["cargo", "build", "--locked"],
        ):
            subprocess.run(command, cwd=destination, env=env, check=True)
    print("Verified isolated settings consumer, fixed type graph, controlled workflow and native build")


if __name__ == "__main__":
    main()
