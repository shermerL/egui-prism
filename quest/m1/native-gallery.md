# [M] Integrate and accept the native gallery

## Goal

A coherent runnable Gallery demonstrates accepted controls and their composition on macOS, Windows and Linux with an honest platform evidence matrix.

## Plan

Start at gallery/src/main.rs, native.rs, input.rs, interaction.rs, the per-component pages, docs/gallery.md, the [Dialog contract](/docs/dialogs.md) and the existing CI workflows. Reuse the official winit/egui-winit/egui-wgpu host, ordered input and compositor. This task owns integration and cross-component acceptance, not reimplementation of each component.

Provide compiling usage, clear component availability, exact build/source identity, licenses and actionable setup/device errors. Demonstrate an ordinary settings form and a modal with inputs, selection and fixed surrounding actions. Document host-provided multilingual fonts; check actual glyphs separately from Unicode values. Preserve caller business state when switching pages or closing a presentation surface.

At this planned integration review node, cover focus/pointer capture across components, modal/popover layering, clipboard and real IME, resize/scale changes, focus loss/reentry, window suspend/close/restart, compositor error propagation and accessibility actions. Reuse unchanged component tests and host lifecycle evidence. macOS, Windows and Linux (with X11/Wayland identified) each need separate build, controlled GPU and real desktop evidence. No browser runtime, signing, installer, package publication or new host framework is required. Missing observations remain explicit acceptance gaps; do not silently lower the promised scope.

Reuse the [standalone settings recipe](/examples/settings/README.md)'s existing
controlled caller-state and mixed-widget regressions rather than duplicating them.
For the recipe's real-platform follow-up, record exact recipe/dependency revisions,
OS/backend and X11/Wayland where applicable. Observe edit, unrelated redraw,
continued edit, modal cancel/confirm with focus restoration, native close shortcut,
clipboard/IME composition, resize/DPI, focus loss and suspend/restart. Verify the
current-frame group and overlay image on a real device and record accessibility
actions/readouts separately. This recipe currently has no native AT bridge:
before claiming screen-reader support, the Gallery integration owner must wire
the official egui-winit AccessKit adapter into that host, cover its event/cleanup
path, then obtain real platform observations. Gallery's existing bridge does not
connect another binary automatically. Keep this missing implementation separate
from unperformed observation, without introducing a second semantic tree or
custom action dispatcher. Attach results and failures, not only screenshots.
These observations belong to this platform-acceptance task; they are not hidden
prerequisites for landing the recipe's implementation and basic tests. Keep Gallery platform acceptance separate from external setup evidence;
neither a successful Gallery run nor a source archive proves production adoption.

## Required

- [Native accessibility](/quest/m1/native-accessibility.md) - official system bridge
