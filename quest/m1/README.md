# M1: Core components and native gallery

## Goal

Ship usable core widgets and a native Gallery for macOS, Windows and Linux, with
documented ownership and strict shadcn appearance and interaction evidence.

## Plan

Use the official winit, egui-winit and egui-wgpu integration for the native host.
Keep the core egui-only, with a separately installed wgpu group compositor and
controlled, fixed-version egui input compatibility patches where required.
The host owns windows, frame ordering and resource lifetime; egui-winit bridges
platform input and clipboard services.

Native desktop is the primary product. Rust browser compatibility and Pages do
not block M1; their existing preparation and deployment tasks move to the deferred
M2 track. The real pinned shadcn browser reference remains the comparison source.
Record macOS, Windows and Linux build, rendering, input and IME evidence separately.

## Required

- [Strict shadcn parity for existing components](/quest/m1/shadcn-parity/README.md)

- [Slider](/quest/m1/slider.md)
- [Select](/quest/m1/select.md)
- [Dialog](/quest/m1/dialog.md)
- [Build and validate the native gallery](/quest/m1/native-gallery.md)
