#!/usr/bin/env python3
"""Run the same locked checks locally and in CI."""

import argparse
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def run(*args, env=None):
    print("+ " + " ".join(args), flush=True)
    subprocess.run(args, cwd=ROOT, check=True, env=env)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--native-only", action="store_true")
    options = parser.parse_args()

    run("quest", "check")
    run("nixfmt", "--check", "flake.nix")
    run("actionlint")
    run("cargo", "fmt", "--all", "--", "--check")
    run("cargo", "check", "-p", "egui-prism", "--locked", "--no-default-features")
    run("cargo", "clippy", "--workspace", "--all-targets", "--locked", "--", "-D", "warnings")
    run("cargo", "test", "--workspace", "--locked")
    run("cargo", "doc", "--workspace", "--no-deps", "--locked")
    run("cargo", "build", "--workspace", "--locked")
    if not options.native_only:
        run("cargo", "clippy", "--workspace", "--target", "wasm32-unknown-unknown", "--locked", "--", "-D", "warnings")
        # Trunk 0.21 parses NO_COLOR as a boolean rather than the conventional
        # nonempty string. Preserve its meaning only for this child process.
        trunk_env = os.environ.copy()
        if "NO_COLOR" in trunk_env:
            trunk_env["NO_COLOR"] = "true" if trunk_env["NO_COLOR"] else "false"
        run("trunk", "build", "--release", env=trunk_env)
        run("python3", "scripts/check_web.py")


if __name__ == "__main__":
    os.environ["RUSTDOCFLAGS"] = os.environ.get("RUSTDOCFLAGS", "") + " -D warnings"
    main()
