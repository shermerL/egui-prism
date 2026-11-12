# External settings consumer

This unpublished recipe is one application, not a reusable host framework. It
uses only public Prism, compositor, egui and native-host APIs. It does not import
Gallery modules. The canonical runnable sources are [application state and
workflow](src/lib.rs) and [native frame/resource ownership](src/main.rs); the
[manifest](Cargo.toml) and [standalone lockfile](Cargo.lock) are part of the recipe.
There are no duplicated runnable listings to keep synchronized.

## Fixed candidate and requirements

- Core, compositor and controlled egui source: Prism
  `5f2c2280d5a4e221b6dd9c9a49d6fafe573c8bfb`. Both Prism crates use that revision.
- Official egui/egui-winit/egui-wgpu baseline: 0.37.0. The root patch in this
  recipe selects the controlled egui. A dependency's patch table would not do so.
- Third-party control: shadcn-egui Button from
  `2e9b53058657fa26f70e27a8d0fa8f63ab1761fc`. Its stable allocated response joins
  this application's event dispatch. This is not general legacy-plugin support
  or acceptance of that library's visual contract.
- Toolchain: Rust 1.96.0, declared minimum 1.96. The actual commands and CI results
  are evidence; the declaration itself is not an MSRV observation.
- Single root window, winit 0.30.13 and wgpu 30.0.0. Native backend selection is
  Metal on macOS, DX12 on Windows and Vulkan on Linux. A single-sample gamma
  `Rgba8Unorm` or `Bgra8Unorm` surface is required; an unavailable configuration
  fails rather than silently substituting an unsupported target.
- The app initializes ordered input and its fixed third-party font/theme setup
  once per Context. Stock egui Modal is an explicit alternative in this recipe.
  It neither uses nor accepts Prism Dialog's outstanding Issue 29 semantics.

The [library-owned support schema](../../docs/releases.md#version-support-contract),
[input protocol](../../docs/input-compatibility.md) and
[compositor contract](../../crates/egui-prism-wgpu/README.md) remain authoritative.
The recipe selects them; it does not redefine the library's support scope.

## Run outside the repository

Copy this directory alone to any directory outside the Prism checkout. No parent
lockfile, patch table, private host code or generated source is needed. Keep
`Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml` and `src` together. The dependencies
are fetched by exact revision, so first use needs network access. Then run:

```sh
cargo test --locked
cargo run --locked
```

Use the native development packages for winit/egui-winit on the chosen platform.
On Debian/Ubuntu Linux these include pkg-config, X11, Xcursor, Xi, Xrandr,
xkbcommon and Wayland development packages, plus a Vulkan driver. A real display
server is needed to run the window. A build-only/headless environment is not
native desktop acceptance. No `--no-sandbox`, renderer fallback or media server
is needed.

From the repository, `python3 scripts/check_settings_example.py` exports only the
standalone package files to an isolated workspace, validates its resolved source
graph, then runs formatting, Clippy, controlled workflow tests and the native
build. The normal repository check invokes it on all three native CI platforms.
Use the repository's documented Quest resource gate when running expensive checks.
This checker does not start a real window or attest OS accessibility/IME.

## Application-owned editing

The application retains the saved name and modal draft, not a copied UI framework.
Controls keep stable IDs. The same update function handles synthetic test input
and platform input. It registers current geometry, handles one authoritative
routed event at a time, refreshes changed structure and paints only the final
state. Business actions do not run again during layout or paint.

Try editing the name, trigger an unrelated action/redraw, and continue editing.
Open the modal, edit and cancel, then reopen, edit and confirm. Closing removes
its input layer and restores focus to the opener. Ordinary egui, Prism and the
selected third-party Button share this path. CPU regressions assert the retained
values, actions, modal isolation and focus; they do not synthesize a passing
screen-reader or OS IME result.

## Synthetic image ownership

The native host owns a bounded GPU image for its runtime and registers its view
with its egui renderer. The current frame's producer writes that image before the
compositor samples it. The same registration is drawn inside an opacity Group
and as an ordinary overlay, rather than copied through an unrelated image path.
The frame borrows the renderer for preparation and encoding; callback command
buffers precede the main command buffer at submission. Source ownership survives
submission. Frame-local compositor registrations and texture deltas are released
at their documented points; teardown invalidates the compositor and unregisters
the app-owned source. Suspension retains application settings but recreates the
native/GPU runtime, and stale generation callbacks are ignored.

No decoding, network session or hidden queue is involved. Device/surface errors
are surfaced; the recipe does not promise seamless device-loss recovery.
A source producer and group drawing are different from platform acceptance.

## Evidence boundary

Controlled input, graph/build checks, software GPU pixels and real native use
are separate evidence. Exact-head results belong to the PR and CI; record the
recipe revision as well as its pinned dependency candidate. The source revision
alone does not establish an executed check. Native GUI, IME, clipboard and
screen-reader observations remain unperformed until explicitly recorded for a
named OS/backend/display protocol. The existing native-gallery Quest owns those
platform observations. The recipe is not a release, application migration or
permission to publish.

## Notices

Dependencies retain their upstream authorship. The fixed shadcn-egui source is
MIT-licensed; its bundled Inter font is copyright 2020 The Inter Project Authors
and licensed under SIL OFL 1.1. Preserve its
[font license](https://github.com/oetiker/shadcn-egui/blob/2e9b53058657fa26f70e27a8d0fa8f63ab1761fc/crates/shadcn-egui-font-inter/LICENSE-FONT.txt)
and the Prism candidate's
[third-party notices](https://github.com/shermerL/egui-prism/blob/5f2c2280d5a4e221b6dd9c9a49d6fafe573c8bfb/THIRD_PARTY_NOTICES.md)
when distributing a derived binary. This recipe creates no release or installer.
