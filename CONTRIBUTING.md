# Contributing

The workspace contains the egui-only library in `crates/egui-prism` and the
native/WebAssembly host in `gallery`. The host displays semantic themes, Prism buttons and text inputs, and labeled egui harness
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
utilities. Rustup installs the exact toolchain and wasm target declared in
`rust-toolchain.toml` on first use, without changing the global default Rust
toolchain. See [tool provenance](docs/toolchain.md) for versions and limits.

Nix reads tracked files. When testing newly created, untracked flake inputs,
use `nix develop path:.` until those files are staged.

## Checks

Run the complete local and CI entry point through the resource gate:

```sh
nix develop --command quest gate python3 scripts/check.py
```

It checks Quest structure, Nix and Rust formatting, workflow lint, the standalone
core library, native and wasm Clippy, tests and doctests, documentation, native
builds, and a release wasm bundle. Finally it checks generated assets and license
notices below `/egui-prism/`. Cargo and Trunk use `Cargo.lock` without updating it.
To check only the native path while iterating:

```sh
nix develop --command quest gate python3 scripts/check.py --native-only
```

The library has compiling documentation examples and public component behavior tests.
Add meaningful regressions with each component quest. A green
build is not evidence of real input, IME, accessibility, or another platform's GUI.

## Run the gallery

Native:

```sh
nix develop --command cargo run --locked -p egui-prism-gallery
```

Browser, after the full check builds the bundle:

```sh
nix develop --command python3 -m http.server 8080 --directory dist
```

Open `http://localhost:8080/egui-prism/`. The browser requires WebGL 2. The host
selects glow explicitly, with no WebGPU fallback. The bundle is under
`dist/egui-prism/`, including license notices. Local serving does not deploy Pages.

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

PR CI has read-only permissions, skips drafts, and uploads native/wasm artifacts.
It contains no deployment job. Keep private notes and raw review logs out of
public changes.
