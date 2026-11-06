# Semantic themes

`egui_prism::theme::Theme` provides editable light and dark palettes and shared
layout tokens. It owns no egui context, font data, application value, or resource.
Apply it to the host's existing style at setup and when the theme choice changes:

```rust
use egui_prism::{egui, theme::Theme};

let context = egui::Context::default();
let mut theme = Theme::light();
theme.typography.body = 16.0;
context.all_styles_mut(|style| theme.apply(style));
```

The gallery chooses an explicit egui light/dark mode as well. Applications may
instead install a different Prism theme into each egui style and follow system
preferences. `Theme::apply` does not choose an application preference or request
repaints; the host owns those decisions.

## Color ownership

Every `ColorPair` keeps content with its intended opaque background. `Palette`
contains page, surface, muted, primary, selection, destructive, disabled, and
focus pairs plus a decorative border. Change a pair together when branding.
Default pairs have at least 4.5:1 calculated sRGB text contrast; custom colors,
alpha blending, images, and actual rendering need their own checks. This is not
an accessibility certification.

The egui adapter maps resting controls to surface, hovered/open controls to
muted, pressed/focused controls to focus, and selection to its own pair. It clears
`override_text_color`, because a global foreground would override selected text.
egui also uses the selected foreground for focused text-edit outlines, so it must
contrast with the input surface as well. Composition underlines receive theme
colors while the host's IME legacy-visuals setting is preserved. Use per-widget
text styling for deliberate exceptions. Page foregrounds and
surface foregrounds match in the built-in palettes; customized palettes should
keep ordinary egui labels readable on both since egui has one ordinary text color.

Primary, destructive, and disabled pairs are available to role-specific Prism
components and are previewed as explicit swatches. They do not change the semantic
role of every standard egui button. In particular, egui's disabled UI multiplies
painter opacity, and this adapter preserves that behavior and the caller's
`disabled_alpha`. The opaque disabled swatch is not a claim that arbitrary egui
disabled controls render that exact pair.

## Spacing and fonts

All dimensions are logical points. The current gallery uses one 32-point minimum
control height: 14-point body text with 6-point vertical padding leaves room for
font metrics without fixing content height. There are no size variants yet.
The host supplies 16-point page margins once; containers supply 12-point inner
margins once; sibling rows use 8-point gaps; controls use 12-by-6-point padding.
Do not apply page margins again inside every card. Corners use a 6-point radius.
These are a measured starting layout for the current harness, not universal
platform hit-target guarantees.

Typography uses 14-point body/button/monospace, 12-point supporting text, and
24-point headings. Applying a theme updates the sizes of existing standard text
styles while preserving their font families, custom text styles, font data,
font overrides, interaction settings, and application values. The host loads
fonts through egui; the library bundles no fonts. Applications can customize
these tokens for their chosen fonts and scale requirements.

## Evidence and limits

Automated checks cover all default foreground/background pairs, selected and
focused style mapping, repeated application, and host font/interaction preservation.
The gallery offers theme switching, semantic swatches, text selection, a checkbox,
and standard egui disabled controls. Actual native observations are reported in
the PR separately from tests. IME, screen readers, other OS/browser combinations,
and a full DPI/input matrix are not established by these checks. The subsequent
gallery-harness quest owns systematic interaction cases and screenshot procedures.
