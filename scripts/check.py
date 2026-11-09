#!/usr/bin/env python3
"""Run locked native checks and the pinned shadcn reference checks."""

import argparse
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def run(*args):
    print("+ " + " ".join(args), flush=True)
    subprocess.run(args, cwd=ROOT, check=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--cargo-only", action="store_true")
    mode.add_argument("--policy-only", action="store_true")
    parser.add_argument("--native-only", action="store_true", help="Compatibility alias for the default checks")
    options = parser.parse_args()
    if not options.cargo_only:
        run("quest", "check")
        run("nixfmt", "--check", "flake.nix")
        run("actionlint")
        run(sys.executable, "scripts/check_egui_source.py")
        run(sys.executable, "scripts/check_reference.py")
        run("npm", "ci", "--ignore-scripts", "--no-audit", "--no-fund", "--prefix", "reference")
        run("npm", "run", "build", "--prefix", "reference")
    if not options.policy_only:
        run("cargo", "fmt", "--all", "--", "--check")
        run("cargo", "fmt", "--manifest-path", "vendor/egui/Cargo.toml", "--", "--check")
        run("cargo", "fmt", "--manifest-path", "tests/input-consumer/Cargo.toml", "--", "--check")
        run(sys.executable, "scripts/check_egui_lints.py")
        run("cargo", "test", "--manifest-path", "vendor/egui/Cargo.toml", "--locked", "--lib", "--features", "serde", "--target-dir", "target/vendor-egui")
        run(sys.executable, "scripts/check_input_consumer.py")
        run("cargo", "run", "--locked", "-p", "egui-prism", "--example", "ordered_input")
        run("cargo", "run", "--locked", "-p", "egui-prism", "--example", "select")
        run("cargo", "check", "-p", "egui-prism", "--locked", "--no-default-features")
        run("cargo", "clippy", "--workspace", "--all-targets", "--locked", "--", "-D", "warnings")
        run("cargo", "test", "--workspace", "--locked")
        run("cargo", "doc", "--workspace", "--no-deps", "--locked")
        run("cargo", "build", "--workspace", "--locked")


if __name__ == "__main__":
    os.environ["RUSTDOCFLAGS"] = os.environ.get("RUSTDOCFLAGS", "") + " -D warnings"
    main()
