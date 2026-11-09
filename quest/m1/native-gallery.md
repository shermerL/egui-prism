# [M] Build and validate the native gallery

## Goal

A runnable Gallery demonstrates accepted library components on macOS, Windows
and Linux with reproducible build identity, usage examples and explicit evidence
for each supported platform.

## Plan

M1 prioritizes desktop native use. Build on the official winit, egui-winit and
egui-wgpu host delivered by the compositing Quest; do not implement a second host
or renderer lifecycle. This Quest owns the completed Gallery presentation,
examples, consumer documentation and cross-platform acceptance after the shared
host and components land.

Follow /docs/shadcn-parity.md throughout the visible Gallery. Present real library
components, themes, states and multiple-instance examples with compiling Rust
usage. Label unfinished capabilities accurately. Include source/build identity,
licenses and useful setup or unsupported-device errors. Do not add application
business policy, telemetry, installers, signing or release publication.

Verify startup, resize, scale changes, light/dark states, keyboard navigation,
pointer input, focus loss/reentry, clipboard and actual text composition. Record
macOS, Windows and Linux separately, including Linux display protocol where
tested. CI/builds, controlled GPU tests and real GUI/IME observations are distinct
evidence. Missing platform observation remains an explicit acceptance gap.

Use the real pinned shadcn browser fixture for comparisons; Rust browser support
and Pages deployment are not prerequisites. Preserve repeatable native build and
test commands and read-only CI artifacts for all three desktop platforms.

## Required

- [Strict shadcn parity for existing components](/quest/m1/shadcn-parity/README.md)
- [Dialog](/quest/m1/dialog.md)
