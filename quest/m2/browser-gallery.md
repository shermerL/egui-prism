# [M] Build and validate a browser gallery

## Goal

A deployable Project Pages artifact runs the actual Rust Button/Input components
with theme/state examples, reproducible build identity, and usage documentation.

## Plan

This existing task is deferred from M1 while native macOS, Windows and Linux take
priority. Browser support may be omitted if its complexity is excessive. If
selected later, start with accepted components and themes. Build below
/egui-prism/ with explicit, verified backend and browser requirements,
readable loading/failure states, source and version links, licenses, and compiling
Rust examples. Planned components must not look available.

Verify direct navigation, refresh, cache updates, relative assets, wasm MIME,
console errors, keyboard interaction, and resource failures in a real browser.
Store build artifacts from read-only PR CI. Prepare a separate deployment workflow
limited to the approved main/release path and environment, without exposing write
credentials to quest branches. No custom domain, service, account, or telemetry.

Present the exact artifact/commit and observed browser evidence for the separate
first-deployment quest. Completion here is a reviewed, reproducibly built artifact
and deployment workflow, not a live site. Keeping publication separate allows
implementation and verification to finish before the publication decision.

The visible Gallery and every presented component follow /docs/shadcn-parity.md.
Do not publish approximate components as accepted shadcn implementations.

## Required

- [Build and validate the native gallery](/quest/m1/native-gallery.md)

## Waiting

- Maintainer: select browser delivery after reviewing native results and a concrete browser cost/capability assessment; recommend keeping it deferred until then. Automation can measure compatibility and maintenance cost, but deciding whether the library should support browsers is a product choice.
