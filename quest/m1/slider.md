# [M] Single-value slider

## Goal

A single-value slider keeps visual geometry, hit targets, range, step, and value mapping consistent.

## Plan

M1 owns core components. Use egui slider behavior where possible. Verify drag and track clicks at both ends and intermediate values, keyboard changes, constrained ranges/steps, focus, and disabled state. Compare painted thumb/track positions with the actual value mapping at different sizes and scale factors. Callers own mute or other business semantics.

Include the public API, compiling minimal Rust example, light/dark gallery cases,
behavior regressions, actual visual evidence, and explicit platform limitations
in the same PR. Propose material API choices before implementing them.
