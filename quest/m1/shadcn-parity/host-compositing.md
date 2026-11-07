# [L] Host-owned group compositing

## Goal

The core can draw a complete component at one opacity over arbitrary host
surfaces and caller-owned images, matching the pinned CSS compositing behavior
on native macOS, Windows and Linux without the core owning a renderer.

## Plan

M1 owns the shared capability required by strict component parity. Keep the core
library dependent only on egui. Add an explicit host-installed compositing
capability and a separate wgpu adapter package, integrated by the Gallery.
A disabled primary action exposes the current gap: fading its fill and text
separately makes white text gray, whereas CSS fades the assembled component once.
The earlier Button and Checkbox implementations in [PR #5](https://github.com/shermerL/egui-prism/pull/5) and
[PR #7](https://github.com/shermerL/egui-prism/pull/7) use egui's
primitive opacity. Their correction Quests own component adoption; this Quest
owns the common capability, adapter, host integration and acceptance fixture.

Use a host-owned offscreen target to assemble a frame-local group of shapes,
including content, border, ring and shadow, then composite it once. Preserve
arbitrary backgrounds and caller-managed texture IDs. Do not infer a constant
background, remove only some overlapping geometry, silently fall back to
primitive opacity, or maintain a fork of the renderer stack. The separate adapter
keeps backend dependencies and their maintenance outside the core.

The native Gallery directly integrates official winit, egui-winit and egui-wgpu.
It owns the window, surface and frame lifecycle, while egui-winit bridges platform
input, clipboard and IME. This supersedes the glow/eframe path planned in
[PR #14](https://github.com/shermerL/egui-prism/pull/14): explicit host ownership
provides the frame preparation point required by current renderer texture access
and ordered GPU commands without a renderer fork. Use the pinned official
integration source as the reference, and document the relevant entry points.
Rust browser delivery is deferred and is not a prerequisite for this Quest.

Define the smallest installation and frame submission API needed by the existing
components. Select the supported rendering capability at host setup and report
initialization or unsupported-backend failures explicitly. Document migration
for consumers and the behavior of missing capabilities; an apparently working
but visually incorrect fallback is not supported. Do not expose internal pools,
shaders or frame state as public configuration.

The host owns GPU resources, frame ordering, setup failures and shutdown.
Component groups own immutable frame-local drawing data; callers retain their
image handles through rendering. After uploading the current frame's textures,
the host explicitly prepares the compositor using current renderer bindings and
callback geometry. Order producer writes, group composition and main rendering
on the host's command submission; do not read stale atlas bindings, reenter a
locked renderer or submit a consumer before its producer.

Preserve the core's unsafe-code prohibition. Bound and recycle render targets by
current needs. Surface/device loss, restart, DPI/size changes and failures must
invalidate stale resources and callbacks. Match the host's alpha, color-space,
texture filtering, viewport, transforms and clipping conventions. Keep internal
GPU pools and resource generations private; document cleanup and setup errors.

Include final-frame comparisons against the real pinned reference on white,
dark, colored and patterned backgrounds, with opaque/translucent images,
shadows, borders and focus rings. Cover opacity endpoints and transition frames,
parent opacity, clipping, 100%/150% scale and DPI changes. Validate cleanup,
resource reuse and unsupported/missing capability behavior. Add repeatable
regressions, builds for macOS/Windows/Linux, consumer documentation, independent
review, GitHub code/security review and CI. Report actual GPU/backend observations
separately from shape tests and compilation, identifying each tested platform.
The native Gallery Quest retains complete three-platform end-to-end acceptance;
unobserved platforms must remain visible gaps and cannot inherit macOS evidence.

The Gallery fixture must exercise this capability without taking over unfinished
component implementations. Component Quests integrate the reviewed capability
after it lands and retain their own exact-head acceptance. This Quest does not
publish a release or deploy Pages.
