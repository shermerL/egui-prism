#!/usr/bin/env python3
"""Reconstruct the vendored egui from its checksum-pinned crate and reviewed patch."""
import argparse
import hashlib
import os
from pathlib import Path
import subprocess
import tarfile
import tempfile
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
SHA256 = "bec9144f49b7a28b5f395d3147e325b4fe3e77ba686eac11d1bdf0a7da3f375f"
URL = "https://static.crates.io/crates/egui/egui-0.37.0.crate"
LICENSES = {
    "LICENSE-MIT": "95ca92f5f8ea5231f1580b3a2a799e8260af3114b900e1def5355a7f44bcf60c",
    "LICENSE-APACHE": "8173d5c29b4f956d532781d2b86e4e30f83e6b7878dce18c919451d6ba707c90",
}


def source_archive(explicit=None):
    scratch = ROOT / ".tmp"
    scratch.mkdir(exist_ok=True)
    archive = explicit
    if archive is None:
        cached = sorted((Path.home() / ".cargo/registry/cache").glob("*/egui-0.37.0.crate"))
        archive = cached[0] if cached else scratch / "egui-0.37.0.crate"
        if not archive.exists():
            with urllib.request.urlopen(URL, timeout=60) as response:
                # Bound the downloaded artifact independently of its Content-Length.
                data = response.read(8 * 1024 * 1024 + 1)
            if len(data) > 8 * 1024 * 1024:
                raise ValueError("egui archive exceeds 8 MiB")
            if hashlib.sha256(data).hexdigest() != SHA256:
                raise ValueError("downloaded egui checksum mismatch")
            archive.write_bytes(data)
    if hashlib.sha256(archive.read_bytes()).hexdigest() != SHA256:
        raise ValueError("egui archive checksum mismatch")
    return archive


def unpack(archive, reconstructed):
    with tarfile.open(archive) as source:
        for member in source.getmembers():
            name = Path(member.name)
            if name.is_absolute() or ".." in name.parts or name.parts[0] != "egui-0.37.0":
                raise ValueError(f"invalid archive path: {name}")
            if member.isdir():
                continue
            if not member.isfile():
                raise ValueError(f"unexpected archive member: {name}")
            destination = reconstructed / Path(*name.parts[1:])
            destination.parent.mkdir(parents=True, exist_ok=True)
            with source.extractfile(member) as stream:
                destination.write_bytes(stream.read())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", type=Path)
    args = parser.parse_args()
    archive = source_archive(args.archive)
    scratch = ROOT / ".tmp"
    with tempfile.TemporaryDirectory(prefix="egui-source-", dir=scratch) as directory:
        reconstructed = Path(directory)
        unpack(archive, reconstructed)
        patch_env = dict(os.environ, GIT_CEILING_DIRECTORIES=str(scratch))
        subprocess.run(["git", "apply", "--check", str(ROOT / "vendor/egui.patch")], cwd=reconstructed, env=patch_env, check=True)
        subprocess.run(["git", "apply", str(ROOT / "vendor/egui.patch")], cwd=reconstructed, env=patch_env, check=True)
        expected = {str(p.relative_to(reconstructed)): p.read_bytes() for p in reconstructed.rglob("*") if p.is_file()}
        actual = {str(p.relative_to(ROOT / "vendor/egui")): p.read_bytes() for p in (ROOT / "vendor/egui").rglob("*") if p.is_file()}
        for name, checksum in LICENSES.items():
            if hashlib.sha256(actual.get(name, b"")).hexdigest() != checksum:
                raise ValueError(f"official egui license missing or changed: {name}")
        if expected != actual:
            differences = sorted(name for name in expected.keys() | actual.keys() if expected.get(name) != actual.get(name))
            raise ValueError(f"vendored egui differs from upstream plus patch: {differences}")
        print(f"Verified egui 0.37.0 ({SHA256}), patch and all {len(actual)} vendored files")


if __name__ == "__main__":
    main()
