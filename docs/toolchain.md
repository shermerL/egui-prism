# Toolchain and evidence

## Pinned tools

| Tool | Pin | Owner |
|---|---|---|
| Quest | `19a2438cd80cf968dddfbf664570a2c8b9a254bf` | `flake.nix` and `flake.lock` |
| Rust | 1.96.0, rustfmt, Clippy | `rust-toolchain.toml` |
| egui / egui-winit / egui-wgpu | 0.37.0 | Cargo manifests and `Cargo.lock` |
| winit / wgpu | 0.30.13 / 30.0.0 | Gallery manifest and `Cargo.lock` |
| Node.js | 24.x; reference packages locked separately | `flake.nix`, `reference/package-lock.json` |

Rustup selects the project toolchain without changing the global default.
Cargo.lock fixes native dependency resolution. The core depends only on egui
without default features. The separate adapter uses the official egui-wgpu
renderer; the Gallery adds default fonts and the official winit/egui-winit
window, clipboard, IME, link, Wayland and X11 integration.

The Gallery owns the rendering loop and selects Metal on macOS, DX12 on Windows
and Vulkan on Linux. Browser product packaging is outside this phase's required
checks; the local pinned CSS reference remains part of design validation.
The replaced Trunk entry point and its asset checker are removed. Browser
delivery must establish a supported host and build pipeline in its own Quest.

The Nix shell exposes Apple Silicon macOS, x86_64 Linux and ARM64 Linux.
Portable Cargo checks run separately in the three-OS CI matrix, including Windows
without Nix. A CI binary is not a signed installer or proof of GUI/IME acceptance.
Linux runtime libraries and an appropriate Vulkan driver are required.

## Quest provenance

The CLI is built from the [pinned source mirror](https://github.com/shermerL/quest/tree/19a2438cd80cf968dddfbf664570a2c8b9a254bf).
It reports `0.1.0`; that string alone does not identify the source. The shell
selects the upstream default CLI, not its runner or development shell. The
upstream `quest-pinned` input serves its own development and is not this project's
selected CLI. Init installs upstream skill stubs and the clone-local merge driver;
no global hooks or unattended services are enabled.

Retain Quest at `19a2438cd80cf968dddfbf664570a2c8b9a254bf`. Upgrade evaluation
is deferred until the maintainer explicitly requests it again. Do not track newer
revisions automatically, resume upgrade experiments, or change the Quest pin or
lock entry as routine maintenance. This deferral does not block the completed M0
foundations or component work using the current tool.

If the maintainer requests a new evaluation, scope it as a separate Quest before
changing explicit pins or regenerating locks. Review upstream behavior and rerun
the relevant checks before proposing a version switch. Keep the tool and runtime
dependency graphs separate.

## Source and assets

The original bootstrap consulted the eframe template. The current native host
uses public winit, egui-winit, egui-wgpu and wgpu APIs directly and does not retain
an eframe runtime or renderer fork.

The reference preset embeds Geist Regular, Medium, SemiBold and Geist Mono Regular
from Geist 1.9.2. The gallery retains epaint_default_fonts 0.37.0 as fallbacks: Hack,
Ubuntu Light, and egui's icon subset. Their notices and upstream MIT notices are
retained in `THIRD_PARTY_NOTICES.md`, copied alongside native CI artifacts. The
project adds no separate CJK font or image bundle.

Sources: [egui 0.37.0](https://github.com/emilk/egui/tree/0.37.0),
[Quest setup](https://github.com/shermerL/quest/blob/19a2438cd80cf968dddfbf664570a2c8b9a254bf/SETUP.md).

## Validation boundaries

The check entry point is `python3 scripts/check.py` inside the pinned shell.
CI builds native macOS, Windows and Linux artifacts without publishing a release. Artifacts
are tied to the workflow commit. Local command results and observed GUI behavior
are reported separately in the PR; compilation and artifact checks do not establish
IME, screen-reader, DPI, browser-engine, or cross-platform interaction acceptance.
