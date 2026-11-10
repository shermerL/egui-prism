# Badge

`badge::Badge` is a static text span. The caller owns its borrowed `&str`
and theme. It has no retained component state, activation, focus behavior,
URL, icons, rich text, or application-specific status values.

## Use

Install fonts with `theme::install_fonts` and install the host compositor.
Badge uses the installed Button font family (Geist Medium with the standard
installation) at a fixed 12-point size.

```rust
use egui_prism::{
    badge::{Badge, BadgeVariant},
    compositing,
    egui,
    theme::Theme,
};

fn status(ui: &mut egui::Ui, theme: &Theme) -> Result<egui::Response, compositing::Error> {
    let painter = ui.painter().clone();
    Badge::new(theme, "Ready")
        .variant(BadgeVariant::Secondary)
        .show(ui)
        .paint(&painter)
}
```

`show` allocates and prepares drawing; it does not paint. Submit the returned
drawing in the current frame, either through `paint` or as a child of a
compositing `Group`. Offscreen badges have no drawing.

## Pinned appearance

| Variant | Fill | Text | Border |
| --- | --- | --- | --- |
| Primary/default | Primary background | Primary foreground | Transparent |
| Secondary | Secondary background | Secondary foreground | Transparent |
| Destructive | Destructive background; 60% alpha in dark mode | White | Transparent |
| Outline | Transparent | Page foreground | Semantic border |
| Ghost | Transparent | Page foreground | Transparent |
| Link | Transparent | Primary background | Transparent |

Link remains a span: no underline, URL, hover treatment, or action.

Geometry is independent of theme control metrics: a 16-point line box,
8-point horizontal and 2-point vertical padding, and a reserved 1-point
border produce a 22-point height. Width is natural text width plus 18 points.
Corners form a full pill. Empty or whitespace-only text has no line box and produces an 18 × 6-point badge.

ASCII whitespace collapses as in CSS `white-space: nowrap`; nonbreaking spaces remain.
Text never wraps or elides. The complete caller string remains the accessible label,
even when its drawing extends beyond the parent's clip.

## Disabled scopes, opacity and clipping

A static span does not receive a disabled-control fade. Capture a same-layer
painter **before** entering an egui disabled scope and submit with that painter.

Drawing colors are unfaded and the badge group has opacity 1. Submission
applies the supplied painter's actual parent opacity once. The dark
Destructive fill's 60% alpha affects only its background, not its white text.

Parent clips are preserved in the drawing. For overlapping siblings requiring
shared opacity, assemble their drawings in one parent group rather than
submitting them independently.

## Fixtures and coverage

The native Badges page uses the existing gallery light/dark and scale controls.
The browser fixture is available at `?badges`, with matching six variants,
a long label in a 320-point horizontal scroll area, and 60% opacity over a
tinted surface.

Public API integration tests cover pinned geometry and colors, host font-family
inheritance, empty and long text, accessible naming, noninteraction, clipping,
and explicit opacity submission. The module contains a compiling API example.

## Evidence and limits

The reference is `registry/new-york-v4/ui/badge.tsx` at shadcn commit
`8a3536678a528cc012b9e62484290e79d0be410f`; its unchanged source and hash are
included in the reference fixture.

On 2026-11-10, the native Gallery on macOS / Apple M3 / Metal and the actual
Chrome reference were observed with light/dark themes at 100% and 150% scale.
The six variants, pill geometry, single-line text, and 60% parent opacity
samples rendered; the long label's end was reachable through horizontal
scrolling (native scrollbar drag and browser horizontal scrolling).
Chrome measured 22 CSS-pixel height, 12-pixel Medium text, 16-pixel line height,
2-by-8-pixel padding and a 1-pixel border. These are logical geometry and visual
observations, not a calibrated framebuffer pixel comparison.

The seven controlled public-API tests exercise layout/color contracts,
whitespace and empty text, accessibility metadata, noninteraction, clipping,
and compositor submission. Accessibility metadata does not establish real
screen-reader acceptance. Windows/Linux GUI, assistive technology and
cross-platform font rasterization remain unverified.
