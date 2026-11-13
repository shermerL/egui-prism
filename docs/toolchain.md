# Toolchain and evidence

## Pinned tools

| Tool | Pin | Owner |
|---|---|---|
| Quest | `0c1da19a74f77e4f662cf9536fb789aca686b3bc` | `flake.nix` and `flake.lock` |
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

## CI scheduling and dependency cache

PR creation, reopening and head updates run the full Check workflow, including
Draft PRs. Changing a PR to Ready does not repeat checks for an unchanged head.
Runs for the same PR share a concurrency group, so a newer run cancels the older
one. Different PRs and every main push have separate groups; main checks are not
superseded by later pushes. A rerun shares its original group and can supersede
another run of that PR, so rerun only the current candidate.

The native jobs use pinned `macos-26` ARM64, Windows 2022 and Ubuntu 24.04 labels,
with the same locked checks and Linux software-GPU tests. Older macOS versions
are outside this CI validation scope; no macOS 15/14 or Intel fallback is kept.
This is a CI baseline, not a release minimum-version or GUI/AT acceptance claim.
No documentation-only exemption is applied. Reducing duplicate requests or moving
to the newer macOS image does not reserve hosted capacity: a job that never
acquires a runner is not a test pass.

A SHA-pinned official actions/cache step stores only Cargo registry indexes,
crate archives, unpacked sources and Git object databases. Keys separate OS and
architecture and include all checked-out Cargo lockfiles and the Rust toolchain
file. A source-only prefix restore can reuse downloads across lockfile changes;
Cargo still resolves and validates the locked dependencies. Every build, lint
and test runs on cache hits and misses. Compiler output, executables, Cargo
configuration and credentials are not cached. Cache contents are not confidential
and must contain only public dependencies. Existing GitHub branch/PR cache scopes
apply; this does not grant PR jobs write repository permissions.

## Quest provenance

The CLI is built from the [pinned upstream source](https://github.com/moq-dev/quest/tree/0c1da19a74f77e4f662cf9536fb789aca686b3bc).
It reports `0.1.0`; that string alone does not identify the source. The shell
selects the upstream default CLI, not its runner or development shell. The
upstream `quest-pinned` input serves its own development and is not this project's
selected CLI. Init installs upstream skill stubs and the clone-local merge driver;
no global hooks or unattended services are enabled.

The project's nixpkgs remains pinned independently at
`d3b13ad75cc492690728f9307e443dec9ef9a20e`. Quest uses its own locked nixpkgs
and Bun inputs, so a Quest update does not implicitly replace the project's
Node.js, Python, lint tools, or Linux runtime libraries. Rust and Cargo pins
remain separate.

Quest now derives epic membership from directory contents. Root, milestone,
and epic READMEs do not repeat their own children under `Required`; that section
keeps actual dependencies outside the epic. Milestone meanings are unchanged.

The newer review command can reuse a clean result for an unchanged head and
review subsequent changes with the previous findings. It starts a new review
when the reviewer or governing instructions change. Keep private review records
and concise public summaries; adopting this CLI does not enable merge daemons,
automatic pin bumps, or global hooks.

On macOS, run `quest gate` from a host context that can query process start times
with `/bin/ps`. A restricted agent command sandbox can allow a PID liveness probe
but deny that query; mixing those observers with host-created admissions can
misclassify a live reservation. Request host execution for the gated command in
that environment. This upgrade uses the unmodified upstream CLI and does not
claim support for mixed-permission gate observers.

Keep Quest revisions exact. Evaluate future updates as separate Quests, review
the upstream behavior and lock changes, and run the affected tooling checks
before proposing a switch. Do not update the pin automatically as routine
maintenance.

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
[Quest setup](https://github.com/moq-dev/quest/blob/0c1da19a74f77e4f662cf9536fb789aca686b3bc/SETUP.md).

## Validation boundaries

The check entry point is `python3 scripts/check.py` inside the pinned shell.
CI builds native macOS, Windows and Linux artifacts without publishing a release. Artifacts
are tied to the workflow commit. Local command results and observed GUI behavior
are reported separately in the PR; compilation and artifact checks do not establish
IME, screen-reader, DPI, browser-engine, or cross-platform interaction acceptance.
