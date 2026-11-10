# [M] Integrate and accept the native gallery

## Goal

A coherent runnable Gallery demonstrates accepted controls and their composition on macOS, Windows and Linux with an honest platform evidence matrix.

## Plan

Start at gallery/src/main.rs, native.rs, input.rs, interaction.rs, the per-component pages, docs/gallery.md and the existing CI workflows. Reuse the official winit/egui-winit/egui-wgpu host, ordered input and compositor. This task owns integration and cross-component acceptance, not reimplementation of each component.

Provide compiling usage, clear component availability, exact build/source identity, licenses and actionable setup/device errors. Demonstrate an ordinary settings form and a modal with inputs, selection and fixed surrounding actions. Document host-provided multilingual fonts; check actual glyphs separately from Unicode values. Preserve caller business state when switching pages or closing a presentation surface.

At this planned integration review node, cover focus/pointer capture across components, modal/popover layering, clipboard and real IME, resize/scale changes, focus loss/reentry, window suspend/close/restart, compositor error propagation and accessibility actions. Reuse unchanged component tests and host lifecycle evidence. macOS, Windows and Linux (with X11/Wayland identified) each need separate build, controlled GPU and real desktop evidence. No browser runtime, signing, installer, package publication or new host framework is required. Missing observations remain explicit acceptance gaps; do not silently lower the promised scope.

## Required

- [Strict parity](/quest/m1/shadcn-parity/README.md) - accepted corrected controls and shared contract
- [Native accessibility](/quest/m1/native-accessibility.md) - official system bridge
