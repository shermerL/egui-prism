#!/usr/bin/env python3
"""Require the egui patch to add no diagnostics to a fresh strict upstream Clippy run."""
from collections import Counter
import json
from pathlib import Path
import subprocess
import tempfile

from check_egui_source import ROOT, source_archive, unpack


def diagnostics(manifest, label):
    command = [
        "cargo", "clippy", "--manifest-path", str(manifest), "--locked", "--all-targets",
        "--features", "serde", "--target-dir", str(ROOT / "target/vendor-egui"),
        "--message-format=json", "--", "-D", "warnings",
    ]
    result = subprocess.run(command, cwd=ROOT, text=True, capture_output=True)
    (ROOT / f".tmp/egui-clippy-{label}.log").write_text(result.stdout + result.stderr)
    found = Counter()
    finished = False
    for line in result.stdout.splitlines():
        record = json.loads(line)
        finished |= record.get("reason") == "build-finished"
        if record.get("reason") != "compiler-message":
            continue
        message = record["message"]
        if message["level"] not in ("error", "warning"):
            continue
        spans = []
        for span in message["spans"]:
            if not span["is_primary"]:
                continue
            path = Path(span["file_name"])
            if path.is_absolute():
                path = path.relative_to(manifest.parent)
            spans.append((str(path), tuple(text["text"] for text in span["text"])))
        key = ((message.get("code") or {}).get("code"), message["message"], tuple(spans))
        found[key] += 1
    if not finished or result.returncode not in (0, 101) or (result.returncode and not found):
        raise RuntimeError(f"{label} Clippy failed outside the comparable diagnostic set:\n{result.stderr}")
    print(f"{label} strict Clippy: exit {result.returncode}, {sum(found.values())} diagnostics", flush=True)
    return found


def main():
    archive = source_archive()
    with tempfile.TemporaryDirectory(prefix="egui-lint-base-", dir=ROOT / ".tmp") as directory:
        baseline = Path(directory)
        unpack(archive, baseline)
        # The published manifest is otherwise unchanged; isolate this extracted crate
        # from the enclosing application workspace, just as the vendored crate is.
        with (baseline / "Cargo.toml").open("a") as manifest:
            manifest.write("\n[workspace]\n")
        original = diagnostics(baseline / "Cargo.toml", "upstream")
        patched = diagnostics(ROOT / "vendor/egui/Cargo.toml", "patched")
    introduced = patched - original
    if introduced:
        for diagnostic, count in introduced.items():
            print(f"New diagnostic ({count}): {diagnostic}")
        raise SystemExit("egui patch introduces Clippy diagnostics")
    print("No new vendor Clippy diagnostics; inherited upstream diagnostics remain explicitly reported")


if __name__ == "__main__":
    main()
