# [L] Host-owned group compositing

## Goal

The core can draw a complete component at one opacity over arbitrary host
surfaces and caller-owned images, matching the pinned CSS compositing behavior
on native OpenGL and WebGL2 without taking ownership of a renderer.

## Plan

M1 owns the shared capability required by strict component parity. Keep the core
library dependent only on egui. Add an explicit host-installed compositing
capability and a separate glow adapter package, integrated by the Gallery.
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

Define the smallest installation and frame submission API needed by the existing
components. Select the supported rendering capability at host setup and report
initialization or unsupported-backend failures explicitly. Document migration
for consumers and the behavior of missing capabilities; an apparently working
but visually incorrect fallback is not supported. Do not expose internal pools,
shaders or frame state as public configuration.

The host owns context creation, GPU resources, frame ordering, setup failures and
shutdown. Component groups own only immutable frame-local drawing data; callers
retain their image handles through rendering. Scope the required unsafe GL calls
to the adapter with explicit safety invariants; preserve the core's unsafe-code
prohibition and do not weaken workspace policy globally. Bound and recycle
render targets by current rendering needs, and clean up while the owning context
is valid. Context loss, restart, DPI/size changes and failures must invalidate
stale resources and callbacks. Restore the target framebuffer, viewport, scissor
and other modified GL state on every path. Match the host's alpha, color-space,
texture filtering and clipping conventions.

Include final-frame comparisons against the real pinned reference on white,
dark, colored and patterned backgrounds, with opaque/translucent images,
shadows, borders and focus rings. Cover opacity endpoints and transition frames,
parent opacity, clipping, 100%/150% scale and DPI changes. Validate cleanup,
resource reuse and unsupported/missing capability behavior. Add repeatable
regressions, native and wasm builds, consumer documentation, independent review,
GitHub review and CI. Report actual native OpenGL and WebGL2 observations
separately from shape tests and compilation.

The Gallery fixture must exercise this capability without taking over unfinished
component implementations. Component Quests integrate the reviewed capability
after it lands and retain their own exact-head acceptance. This Quest does not
publish a release or deploy Pages.
