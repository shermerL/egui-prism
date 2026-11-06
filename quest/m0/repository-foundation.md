# [M] Reproducible library and gallery workspace

## Goal

A clean checkout builds the egui-only library and the eframe gallery for native
and wasm targets with pinned tools and a documented, read-only CI check path.

## Plan

M0 owns engineering foundations. The bootstrap already pins Quest; use that CLI
and do not replace it with a same-version cached binary. Initialize a Cargo
workspace with crates/egui-prism and gallery. Start with Rust 1.96.0 and exact
egui/eframe 0.37.0; commit Cargo.lock and rust-toolchain.toml after verifying the
combination. Keep eframe, windowing, and renderer dependencies out of the library.

Evaluate explicit glow for native and WebGL for wasm. Select and pin the wasm
packaging tool after checking the upstream eframe 0.37 template. A minimal shell
may use egui widgets to prove the build, labeled as harness controls rather than
finished prism components. Do not copy a consumer application's prototype API.

Add reproducible fmt, clippy, library/test/doc checks and native/wasm builds to a
single documented entry point. CI uses read-only permissions, skips claim-only
draft PRs, and builds artifacts without deploying. Check supported action runtimes
and runner images: the bootstrap CI reports a Node 20 checkout-action warning
and an ubuntu-latest migration notice. Verify the supported Nix
platform evaluations and record build platforms actually exercised. Preserve
notices and inventory any assets; do not bundle large CJK/icon fonts by default.

Completion includes a clean-checkout rebuild, locked dependency resolution, and
published build commands. Compilation does not establish GUI, IME, assistive
technology, or another platform's runtime acceptance. There is no release or
consumer integration in this quest.
