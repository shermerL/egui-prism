# [M] Shared native and WebAssembly acceptance gallery

## Goal

One Rust gallery exercises the same library on native and browser targets, with
repeatable theme/state/size cases and a useful interaction test harness.

## Plan

M0 owns the acceptance harness. Provide a case selector, light/dark theme controls,
viewport/scale cases, and a visible distinction between planned and available
components. Harness-only egui controls must not appear as implemented prism APIs.

Exercise public responses using meaningful pointer/keyboard input, with stable
IDs, no disabled actions, no repeated events over idle frames, and independent
instances. Support a reproducible screenshot procedure; record actual screenshots
separately from automated headless output. Check normal and restricted layouts,
long labels, scrolling, and caller-owned values across theme changes.

Build the same Rust code for wasm. Add readable loading/error UI and verify local
hosting below /egui-prism/, refreshing, resources, and console errors in one real
browser. Explicitly select WebGL rather than silently selecting another backend.
Add a source/build version label and license links. Do not deploy Pages here.
Document untested browsers, IME, DPI combinations, and assistive technology.
