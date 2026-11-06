# [M] Button and IconButton

## Goal

Buttons expose clear roles and sizes with aligned text/icons, accessible names, and reliable activation.

## Plan

M1 owns core components. Cover default, hover, pressed, focus, and disabled states in both themes. Verify disabled buttons emit no action, keyboard activation fires once, idle frames do not repeat it, and multiple stable IDs stay independent. Return egui Response and leave application actions to the caller. An icon-only button needs an accessible name; do not bundle an icon font by default.

Include the public API, compiling minimal Rust example, light/dark gallery cases,
behavior regressions, actual visual evidence, and explicit platform limitations
in the same PR. Propose material API choices before implementing them.

## Required

- [Gallery harness](/quest/m0/gallery-harness.md)
