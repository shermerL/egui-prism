# [M] Build and validate the first Pages gallery

## Goal

A deployable Project Pages artifact runs the actual Rust Button/Input components
with theme/state examples, reproducible build identity, and usage documentation.

## Plan

M1 owns the first site. Start with Button/Input and themes; add other components
as their quests finish. Build below /egui-prism/ with explicit WebGL support,
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

- [Strict shadcn parity for existing components](/quest/m1/shadcn-parity/README.md)
