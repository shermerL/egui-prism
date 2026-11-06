# [M] Single-value slider

## Goal

A single-value slider keeps visual geometry, hit targets, range, step, and value mapping consistent.

## Plan

M1 owns core components. Match the pinned shadcn slider behavior and geometry; egui defaults are not the contract. Verify drag and track clicks at both ends and intermediate values, keyboard changes, constrained ranges/steps, focus, and disabled state. Compare painted thumb/track positions with the actual value mapping at different sizes and scale factors. Callers own mute or other business semantics.

Include the public API, compiling minimal Rust example, light/dark gallery cases,
behavior regressions, actual visual evidence, and explicit platform limitations
in the same PR. Propose material API choices before implementing them.

Follow [the strict shadcn contract](/docs/shadcn-parity.md), including actual
reference comparison for every supported state. Earlier themed-egui candidates
do not satisfy this requirement.

## Required

- [Host-owned group compositing](/quest/m1/shadcn-parity/host-compositing.md) - shared opacity and transition rendering
- [Input compatibility](/quest/m1/shadcn-parity/input-compatibility.md) - ordered input and authoritative focus, hit and capture behavior
