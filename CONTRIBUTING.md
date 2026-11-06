# Contributing

The workspace contains the egui-only library in `crates/egui-prism` and the
native winit/wgpu host in `gallery`. The host displays semantic themes, Prism buttons and text inputs, and labeled egui harness
controls. Other components are subsequent quests.
The [quest tree](quest/README.md) is the task source.

## Development tools

Use Nix with flakes enabled on Apple Silicon macOS or Linux:

```sh
nix develop
quest init
quest context
```

The locked shell supplies Quest, rustup, Trunk, wasm-bindgen, Python, and lint
utilities. Rustup installs the exact toolchain declared in
`rust-toolchain.toml` on first use, without changing the global default Rust
toolchain. See [tool provenance](docs/toolchain.md) for versions and limits.

Nix reads tracked files. When testing newly created, untracked flake inputs,
use `nix develop path:.` until those files are staged.

## Checks

Run the complete local and CI entry point through the resource gate:

```sh
nix develop --command quest gate python3 scripts/check.py
```

The default checks cover repository policy, locked native Rust checks and the
pinned shadcn reference. `--cargo-only` runs portable Rust checks on macOS,
Windows and Linux; `--policy-only` runs repository tooling and the reference.
`--native-only` is retained as an alias for the default native checks.

```sh
python scripts/check.py --cargo-only
```

The library has compiling documentation examples and public component behavior tests.
Add meaningful regressions with each component quest. A green
build is not evidence of real input, IME, accessibility, or another platform's GUI.

## Run the gallery

Native:

```sh
nix develop --command cargo run --locked -p egui-prism-gallery
```

The host selects Metal on macOS, DX12 on Windows and Vulkan on Linux, with
X11 and Wayland input bridges enabled. A matching native toolchain and GPU driver
are required. Browser product builds and Pages are outside this phase's required
checks. Build the pinned CSS reference with `npm ci --prefix reference` followed
by `npm run build --prefix reference`; see `reference/README.md`.

## Changes and review

Read `quest guide`, plan one independently completable outcome per quest, and
claim approved work with `quest start`. Include implementation, runnable examples,
documented ownership, and relevant checks in the same component PR. Keep transient
UI state isolated by stable IDs and preserve third-party notices.

Append commits and merge base updates into pushed branches. Run checks and obtain
an independent review of the exact committed head before marking a PR ready.
A ready PR is a candidate for maintainer review, not an instruction to merge.
The maintainer selects which reviewed changes may land; releases and the first
Pages deployment are separate decisions.

PR CI has read-only permissions, skips drafts, and uploads native artifacts for each OS.
It contains no deployment job. Keep private notes and raw review logs out of
public changes.
