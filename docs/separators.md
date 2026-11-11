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
