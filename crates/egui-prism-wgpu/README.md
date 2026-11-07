# Host-owned group compositing

A group's opacity is applied once after assembling its shapes. An opaque white
glyph over an opaque dark fill therefore remains white before the assembled
control is faded over its background. Per-shape alpha changes that result.
See the [CSS opacity definition](https://www.w3.org/TR/css-color-4/#transparency).

The core `egui-prism` library depends only on egui. The separate
`egui-prism-wgpu` adapter integrates with the unmodified egui-wgpu 0.37 renderer.
The native Gallery uses winit and egui-winit for macOS, Windows and Linux input,
IME and clipboard, and explicitly owns its rendering loop. Browser deployment
is not an acceptance requirement for this phase. The pinned shadcn webpage
remains the visual and interaction reference.

The integration follows egui 0.37's pinned official sources:
[`Renderer::update_texture`, `update_buffers` and `render`](https://github.com/emilk/egui/blob/7c0b40402271e8d40cda69f87cf3c2617c2e4382/crates/egui-wgpu/src/renderer.rs),
the [native wgpu painter](https://github.com/emilk/egui/blob/7c0b40402271e8d40cda69f87cf3c2617c2e4382/crates/egui-wgpu/src/winit.rs),
and [`egui_winit::State` and viewport command handling](https://github.com/emilk/egui/blob/7c0b40402271e8d40cda69f87cf3c2617c2e4382/crates/egui-winit/src/lib.rs).

## Installation and frame ownership

Create the device and egui renderer, then install one compositor per egui context:

```no_run
# use egui_prism::egui;
# use egui_prism_wgpu::egui_wgpu::wgpu;
# fn install(context: egui::Context, device: wgpu::Device) -> Result<(), egui_prism::compositing::Error> {
let mut compositor = egui_prism_wgpu::Host::new(&context, &device)?;
# Ok(())
# }
```

The context registration lives with the host. Duplicate installation fails. The
host, renderer, queue and source textures must belong to the same device. Use
single-sample gamma color attachments (`Rgba8Unorm` or `Bgra8Unorm`), premultiplied
egui colors and normal hardware texture filtering. The adapter preserves the
registered image's sampler, including its filter and wrap modes. Its binding
layout is pinned to egui-wgpu 0.37; upgrading that dependency requires compatibility
validation. Manual predictable-filtering and sRGB attachment configurations are
not part of this integration contract.

For each frame:

1. Run the UI and tessellate the final egui shapes.
2. Upload every current texture delta, including all deltas for an image and font
   atlas growth, to the same egui renderer.
3. Call `prepare_frame` with the final paint jobs and physical screen descriptor.
4. Call `Frame::encode` with the host's main encoder. This prepares ordinary
   egui-wgpu callbacks, then encodes the group's offscreen passes.
5. Start the host's final render pass and call `Frame::render`.
6. Submit returned callback command buffers before the main command buffer.
7. Drop the frame and free egui's requested source textures after submission.

```no_run
# use egui_prism::egui;
# use egui_prism_wgpu::{Host, egui_wgpu::{self, wgpu}};
# fn render(compositor: &mut Host, mut renderer: egui_wgpu::Renderer,
#     queue: &wgpu::Queue, mut encoder: wgpu::CommandEncoder,
#     paint_jobs: Vec<egui::ClippedPrimitive>, screen: egui_wgpu::ScreenDescriptor,
#     descriptor: wgpu::RenderPassDescriptor<'_>) -> Result<(), egui_prism::compositing::Error> {
let mut frame = compositor.prepare_frame(&mut renderer, paint_jobs, screen)?;
let commands = frame.encode(&queue, &mut encoder)?;
{
    let mut pass = encoder.begin_render_pass(&descriptor).forget_lifetime();
    frame.render(&mut pass)?;
}
queue.submit(commands.into_iter().chain([encoder.finish()]));
drop(frame);
# Ok(())
# }
```

The frame exclusively borrows the compositor and renderer. Temporary image
registrations are removed on drop, including a failed preparation; caller image
registrations are never removed by the adapter. The host owns source handles
through submission. wgpu command buffers retain GPU resources needed for their
execution; the adapter does not call destroy on borrowed textures.

Caller GPU image producers may encode work in the shared encoder during
prepare/finish_prepare or return command buffers from these callbacks. Both
execute before the offscreen passes under the submission sequence above. A
producer that writes a consumed image later in the final paint pass is outside
this ordering contract. No group callback submits a hidden queue or reacquires a
renderer lock. The host must treat preparation/encoding errors as a failed frame,
not submit a partial frame as successful output.

## Drawing groups

Use unfaded shapes in egui points, individual clip rectangles, and finite bounds
that include desired antialiasing, borders, rings and shadows:

```no_run
use egui_prism::{compositing::{self, Group, Item}, egui};
# fn draw(ui: &egui::Ui, visual_bounds: egui::Rect, clip: egui::Rect,
#     opacity: f32, fill: egui::Shape, image: egui::Shape, text: egui::Shape)
#     -> Result<(), compositing::Error> {
let group = Group::new(visual_bounds, opacity, vec![
    Item::Shape(egui::epaint::ClippedShape::new(clip, fill)),
    Item::Shape(egui::epaint::ClippedShape::new(clip, image)),
    Item::Shape(egui::epaint::ClippedShape::new(clip, text)),
])?;
compositing::paint(ui.painter(), group)?;
# Ok(())
# }
```

Inherited painter opacity is applied to the assembled root once. It cannot undo
colors already faded by the caller. Use `Item::Group` for a parent containing
potentially overlapping children. Contents are back-to-front. Custom callbacks
inside groups are rejected because their drawing and resource contracts cannot
be inferred. Missing host or texture bindings are explicit errors; there is no
fixed-background or primitive-opacity fallback.

Preparation uses the final callback rectangle and clip after egui's layer
transforms. Positive uniform scale and translation apply to group contents and
clips together. Targets cover the visible pixel-aligned extent and respect the
device's texture size limit. Each live group has a target; the pool reuses exact
sizes across frames and shrinks to the current frame's needs. Nested groups are
encoded before the parent that samples them. This is a lifetime policy, not a
performance claim.

## Shutdown, loss and migration

Drop the host at normal shutdown. Call `Host::invalidate` on device loss; it
unregisters the capability and revokes old submissions. Recreate the whole host
for a replacement device. Expired submissions are skipped, and live submissions
from another host are rejected. Surface reconfiguration is separate from device
loss. The Gallery reports device failure and requires an explicit host restart;
it does not promise seamless restoration of caller GPU state.

The Gallery owns one root window. A custom multi-viewport host must route each
frame to the correct device/renderer/viewport and preserve the same ordering.
The core does not own native windows, event loops or device-loss callbacks.
The adapter and core both retain the workspace's unsafe-code prohibition.

Stock eframe does not expose the required after-upload preparation point. This
adapter is an explicit render-loop integration, not an install-only eframe plugin.
Consumers keeping eframe need a separately supported integration; this project
ships no renderer fork or dependency patch for compositing. Existing components
adopt this capability in their own parity Quests after it is reviewed.

## Verification

```sh
nix develop --command quest gate python3 scripts/check.py
nix develop --command quest gate cargo run --locked -p egui-prism-gallery -- --compositing-check
```

The conformance fixture compares final framebuffer pixels across backgrounds,
image alpha, opacity transitions, parent opacity, clipping, layer transforms,
rings/shadows and 100/150-percent UI zoom. It also checks that a callback queued
before installation replacement cannot draw through the new host. The reference
site's `/?compositing` route uses real CSS opacity and pinned Button roles.

Native compile/test jobs, GPU framebuffer regressions, manual GUI/IME/clipboard,
OS DPI transitions and actual platform observations are separate evidence.
Metal observations do not establish Windows or Linux runtime acceptance. Linux
X11 and Wayland observations are also separate. No browser product compatibility
or release/deployment follows from building the reference page.

CI explicitly runs the GPU contract test on Linux Mesa software Vulkan; an
unavailable adapter fails the check. It covers current-frame image producers,
temporary registration cleanup and installation revocation. This headless
software backend is regression coverage, not desktop or physical GPU acceptance.
The documentation examples compile as consumer doctests of the adapter crate.
