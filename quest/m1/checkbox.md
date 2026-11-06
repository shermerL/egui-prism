# [S] Labeled checkbox

## Goal

A checkbox exposes caller-owned checked state and responds consistently through both its box and label.

## Plan

M1 owns core components. Cover checked, unchecked, focused, and disabled states. Click and keyboard toggles change the value once; disabled controls do not change it. Check label hit bounds and separate instances. Defer tri-state behavior unless a concrete requirement justifies it.

Include the public API, compiling minimal Rust example, light/dark gallery cases,
behavior regressions, actual visual evidence, and explicit platform limitations
in the same PR. Propose material API choices before implementing them.
