# Separator

`separator::Separator` is a noninteractive, one-logical-point border. It borrows
its theme, defaults to horizontal and decorative, and owns no persistent state.
The caller owns the bounded layout, spacing, theme and renderer lifecycle.

## Public API

```rust
use egui_prism::{compositing, egui, separator::{Orientation, Separator}, theme::Theme};

fn content(ui: &mut egui::Ui, theme: &Theme) -> Result<(), compositing::Error> {
    Separator::new(theme).show(ui).paint(ui.painter())?;
    ui.allocate_ui_with_layout(
        egui::vec2(240.0, 48.0), egui::Layout::left_to_right(egui::Align::Center),
        |ui| Separator::new(theme).orientation(Orientation::Vertical)
            .decorative(false).show(ui).paint(ui.painter()),
    ).inner?;
    Ok(())
}
```

`show` allocates the line and returns its response and optional frame-local
compositing `Group`. It does not paint. Install the existing host compositor and
submit with `paint`, or include the drawing in a parent group. The compiling
`separator` example also exercises both orientations without a rendering host.

## Geometry and color

The horizontal line fills the available width before wrapping; the vertical
line fills the available height. Both are exactly one logical point thick,
independent of the theme's control metrics and egui separator styling. There
is no intrinsic margin, padding, grow, shrink, interaction or automatic direction
choice. Normal parent item spacing remains under caller control.

The parent must provide a finite extent on the chosen axis. A vertical separator
inside an unbounded vertical scroll area needs a bounded child row, as above.
Unbounded actual layout fails explicitly instead of creating an infinite surface.
A sizing pass reports zero intrinsic length, and exhausted or zero-length layouts
produce no drawing. Parent clipping affects visible drawing, not the allocated
length. Justified layouts retain the line's actual bounds in the response and
accessibility metadata.

The fill is `theme.colors.border`: opaque neutral in the default light theme and
white at 10% alpha in the default dark theme. A four-corner filled path avoids
epaint's thin-rectangle pixel-snapping shortcut.
Its compositing bounds include the antialiasing fringe within the parent clip,
preserving logical geometry and coverage at fractional scale.
Actual rasterization still depends on the renderer and scale.

Drawing is unfaded. Submission applies the painter's parent opacity once.
Like a static HTML separator, this component has no disabled appearance; capture
a same-context, same-layer painter before an egui disabled scope. Use a shared
parent group when overlapping siblings need group opacity.

## Semantics

Decorative separators create no accessible separator node. `decorative(false)`
exposes AccessKit's `Splitter` role, the platform mapping for a separator, with
explicit horizontal or vertical orientation and actual line bounds. Neither
mode requests click, drag, keyboard focus, or value-changing actions. The role
does not make this a resizable splitter. Accessible metadata is not evidence of
native assistive-technology acceptance; the application owns its platform bridge.

## Reference, fixtures and evidence

The unmodified `registry/new-york-v4/ui/separator.tsx` source is pinned to shadcn
`8a3536678a528cc012b9e62484290e79d0be410f`, with Radix Separator 1.1.7 already
present in the reference lockfile. The manifest records both and the source hash.

The native Separators page and `/?separators` browser fixture cover decorative
and semantic horizontal/vertical cases, 320-point width, 48-point height, clipping
to 160 points, and 60% parent opacity over a tinted surface. Both provide
light/dark and 100%/150% scale controls.

Focused public-API tests cover geometry and colors at 100%/150%/200%, real AccessKit
output, clipping and zero extent, pointer/keyboard noninteraction, explicit
compositor submission, justified-layout registration, and actual tessellated geometry/antialiasing bounds.
Automated checks, browser
observations, native windows, and screen-reader acceptance are separate evidence.
Native macOS/Windows/Linux GUI and assistive-technology observations remain pending.

### Automated framebuffer regression evidence

The `egui-prism-wgpu` test
`tests::current_frame_producers_cleanup_and_revocation` now includes 96
Separator framebuffer cases: light/dark tinted surfaces, both orientations,
1x/1.5x scale, 32/0.25/0-point extents, partial clipping, and 100%/60% parent
opacity. It reads back actual GPU pixels and compares the component compositor
against an independently constructed one-point polygon rendered directly by
egui, with parent opacity applied once to the oracle image. RGB rounding allows
three byte values; opaque alpha and zero-extent background are exact.

Reproduce using the existing Linux CI software-Vulkan setup:

```sh
cargo test --locked -p egui-prism-wgpu tests::current_frame_producers_cleanup_and_revocation -- --ignored --exact --nocapture
```

On 2026-11-12, the candidate based on
`11ce1474157c21216dd47a9a4c640781babcc1a4` passed this matrix on Linux,
Rust 1.96.0, Mesa 25.0.7-2+deb13u1 llvmpipe (LLVM 19.1.7), Vulkan.
The evidence applies to the test added with this document, not the base alone.
Removing the production antialiasing-bounds expansion temporarily made the
new test fail at horizontal/1x/32-point/unclipped/full-opacity; restoring the
original source restored the passing result. A second mutation suppressing only
0.25-point component painting also failed (horizontal/1x/unclipped/full-opacity),
confirming the matrix detects complete subpixel-line loss. Both mutations were
reverted. No production change is included.
The existing CI GPU-contract step executes this test; exact-head CI and review
results belong in PR #32.

This oracle shares egui's rasterizer, not Separator's emitted Group. It tests
compositor coverage, clipping and opacity, **not** browser CSS parity, native
window presentation, decorative/semantic AT exposure, or real keyboard input.
Those observations and cross-platform captures remain blocking acceptance work.
Written by dot (AI).
