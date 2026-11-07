# [M] Selectable popup menu

## Goal

A select exposes the current value and a usable popup with disabled items, long-list scrolling, and predictable focus.

## Plan

M1 owns core components. Match the pinned shadcn popup behavior and appearance; egui defaults are not the contract. Verify trigger/selection, disabled items, viewport edges, outside dismissal, Escape, keyboard focus restoration, independent IDs, and no ongoing repaint after dismissal. Verify selected foreground against its background. Do not declare completion from drawing a trigger arrow.

Include the public API, compiling minimal Rust example, light/dark gallery cases,
behavior regressions, actual visual evidence, and explicit platform limitations
in the same PR. Propose material API choices before implementing them.

Follow [the strict shadcn contract](/docs/shadcn-parity.md), including actual
reference comparison for every supported state. Earlier themed-egui candidates
do not satisfy this requirement.

## Required

- [Input compatibility](/quest/m1/shadcn-parity/input-compatibility.md) - ordered input and authoritative focus, hit and capture behavior
