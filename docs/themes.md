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

`ColorPair` groups content with an intended opaque surface. The pinned palette
includes page, surface, muted, secondary, accent, primary, selection, destructive,
disabled and focus roles; border, input and ring are separate tokens. Dark border
and input colors retain CSS alpha. Destructive action content is white, matching
the Button source rather than the separate CSS destructive-foreground variable.

The compatibility adapter maps ordinary egui controls onto these roles. Prism
components own their reference states and geometry. Prism TextInput uses primary selection
colors and draws its focus outline independently. Ordinary egui editors share
selection foreground with their focus border, so the compatibility adapter uses
the neutral focus pair for both to keep them visible. IME underline colors and the host's IME legacy
setting remain independent. Applying a theme clears the global text-color override.

Reference tokens are preserved even where a particular combination does not meet
4.5:1 contrast. Tests check ordinary page, surface and primary text separately
from muted, disabled and destructive states. Reference fidelity is not an
accessibility certification. Custom themes and actual compositing need their own
checks.

## Spacing and fonts

Dimensions are logical points, with one point corresponding to one CSS pixel at
100% scale. Default button/input height is 36 points; the generic button padding
is 16 by 8 and medium corner radius is 8. Checkbox, Switch, Slider and other
components own their individual reference geometry. Page margins are 16,
container margins 12 and sibling gaps 8 in the Gallery harness.

Typography sizes are 14 for body/button/monospace, 12 for supporting text and 24
for headings. `Theme::apply` changes standard text sizes while preserving font
families, custom styles, overrides, interaction settings and application values.
Font installation is a separate explicit host setup operation described below.

## Evidence and limits

The source fixture and font assets have checked hashes and pinned dependency
versions. Automated tests cover token conversion, alpha preservation, font setup,
selection/focus separation and host settings. The real reference fixture is
available under `reference/`; native and browser observations are reported in
PRs separately. Component parity, IME, screen readers and cross-platform input
are not established by theme tests or successful builds.

## Pinned shadcn foundation

The built-in palettes now use the light/dark CSS tokens pinned in
[the parity contract](shadcn-parity.md). Selection uses primary colors; focus
rings use the separate neutral `ring` token. Dark `border` and `input` tokens
retain their alpha and must be composited by the component on its actual surface.
Do not change a reference token to satisfy a contrast assertion intended for a
different role; component states require their own reference comparison.

Call `egui_prism::theme::install_fonts(&context, fonts)` once during host setup to install
the bundled Geist Regular, Medium, SemiBold, and Geist Mono Regular fonts. Pass
the host's complete `egui::FontDefinitions`; setup transfers them to the context.
Supplied font data, fallback chains and custom named families are preserved.
Default proportional text uses Geist; default button and heading styles use
Medium and SemiBold. The host owns setup and may intentionally supply another
font, but that is not the reference preset.

Geist has limited glyph coverage. The host owns fallback font selection, data,
licensing and family chains for the characters its application needs.
`FontDefinitions::default()` supplies fallback fonts only with egui's
`default_fonts` feature enabled; without it, the definitions are empty. For
feature-independent setup, start with `FontDefinitions::empty()` and explicitly
populate the host fallback data and family chains before calling `install_fonts`.

The theme test supplies Hack Regular from a test-only `epaint_default_fonts`
dependency, with an explicit host family and fallback chains. It starts from
`FontDefinitions::empty()`, so standalone and workspace runs use the same input:

```sh
cargo test --locked -p egui-prism --test theme
cargo test --locked --workspace --test theme
```

The standalone graph has the font package only as a dev-dependency and does not
enable egui's `default_fonts`. Workspace tests also include the Gallery's
`default_fonts` selection. The core runtime graph stays free of default fonts:

```sh
cargo tree --locked -p egui-prism -e normal,build,features
cargo tree --locked -p egui-prism -e features
cargo tree --locked --workspace -e features
```

The glyph assertion covers the chosen fallback symbol, not all-language coverage
or native CJK/IME acceptance.

`Palette` adds `secondary`, `accent`, `input`, and `ring`. Applications constructing
Palette literals must provide these fields; prefer customizing `Theme::light()`
or `Theme::dark()`. Default button/input height is 36 points, padding 16 by 8, and
medium radius 8. Other component dimensions come from their own reference source,
not proportional scaling of this height. Previously implemented components are
still undergoing their individual parity corrections.
