# [M] Modal dialog

## Goal

A dialog isolates modal input, exposes close intent, restores focus, and keeps long content and multiple instances correct.

## Plan

M1 owns core components. Match the pinned shadcn modal behavior and appearance, and verify background pointer/keyboard isolation, close and reopen behavior, focus restoration, Escape policy, viewport bounds, content scrolling, and isolated IDs. A regular egui Window is not sufficient evidence. Closing is neutral unless the caller selects a destructive action; do not require duplicate footer close controls.

Include the public API, compiling minimal Rust example, light/dark gallery cases,
behavior regressions, actual visual evidence, and explicit platform limitations
in the same PR. Propose material API choices before implementing them.

Follow [the strict shadcn contract](/docs/shadcn-parity.md), including actual
reference comparison for every supported state. Earlier themed-egui candidates
do not satisfy this requirement.
