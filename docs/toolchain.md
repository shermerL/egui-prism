# Toolchain and evidence

## Pinned tools

| Tool | Pin | Owner |
|---|---|---|
| Quest | `19a2438cd80cf968dddfbf664570a2c8b9a254bf` | `flake.nix` and `flake.lock` |
| Rust | 1.96.0, rustfmt, Clippy, wasm32-unknown-unknown | `rust-toolchain.toml` |
| egui / eframe | 0.37.0 | Workspace `Cargo.toml` and `Cargo.lock` |
| Trunk | 0.21.14 | Locked Nix input and `Trunk.toml` version requirement |
| wasm-bindgen | 0.2.129 | Locked Nix CLI and matching Cargo dependency |
| wasm-bindgen-futures / web-sys | 0.4.79 / 0.3.106 | `gallery/Cargo.toml` and `Cargo.lock` |

Rust is selected by rustup from the project file, not the Nix input's Rust package.
The existing locked Nix input supplies matching Trunk and wasm-bindgen executables,
avoiding an unpinned tool download during packaging. Upgrade the wasm-bindgen
library and CLI together. Cargo.lock fixes transitive resolution for the gallery
and CI. The check command normalizes a nonempty `NO_COLOR` value to `true` for the
Trunk child process because this version expects a boolean. The parent environment
is unchanged, and the requested no-color behavior is preserved.

The core depends only on egui, without its default features. It re-exports egui
so hosts use compatible types; no custom widget API is introduced yet. Only the
gallery depends on eframe, enabling bundled default fonts, glow, native clipboard
and link handling, Wayland and X11.
Native uses OpenGL; the browser explicitly requires WebGL 2, with no WebGPU
fallback. Initialization failures leave a visible message. Themes, systematic
input testing, and accessibility integration belong to later quests.

The shell exposes Apple Silicon macOS, x86_64 Linux, and ARM64 Linux. Native
Windows and Intel macOS development-tool setup are not verified. This is distinct
from eventual component runtime support. Linux runtime libraries are supplied by
the shell; the native CI artifact is not a standalone installer.

## Quest provenance

The CLI is built from the [pinned source mirror](https://github.com/shermerL/quest/tree/19a2438cd80cf968dddfbf664570a2c8b9a254bf).
It reports `0.1.0`; that string alone does not identify the source. The shell
selects the upstream default CLI, not its runner or development shell. The
upstream `quest-pinned` input serves its own development and is not this project's
selected CLI. Init installs upstream skill stubs and the clone-local merge driver;
no global hooks or unattended services are enabled.

Change explicit pins, regenerate locks, review upstream behavior, and rerun checks
when upgrading. Keep the tool and runtime dependency graphs separate.

## Source and assets

The [eframe template](https://github.com/emilk/eframe_template/tree/f108546a2b9f2fc6a82543ceae1625a3893305cb)
was consulted for native/web entry structure and Trunk integration. That revision
still targets eframe 0.36; this host uses the 0.37 `App::ui` API and is compiled
against the actual 0.37 source. It carries no template icons, service worker,
or persistence layer.

The gallery's embedded font assets come from epaint_default_fonts 0.37.0: Hack,
Ubuntu Light, and egui's icon subset. Their notices and upstream MIT notices are
retained in `THIRD_PARTY_NOTICES.md`, copied alongside both CI artifacts. The
project adds no separate CJK font or image bundle.

Sources: [egui 0.37.0](https://github.com/emilk/egui/tree/0.37.0),
[Trunk configuration](https://trunk-rs.github.io/trunk/guide/configuration/index.html),
[Quest setup](https://github.com/moq-dev/quest/blob/19a2438cd80cf968dddfbf664570a2c8b9a254bf/SETUP.md).

## Validation boundaries

The check entry point is `python3 scripts/check.py` inside the pinned shell.
CI builds native Linux and the wasm bundle without publishing either. Artifacts
are tied to the workflow commit. Local command results and observed GUI behavior
are reported separately in the PR; compilation and artifact checks do not establish
IME, screen-reader, DPI, browser-engine, or cross-platform interaction acceptance.
