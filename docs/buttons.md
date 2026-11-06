# Buttons

`button::Button` and `button::IconButton` borrow a `Theme` for one frame and return
an ordinary `egui::Response`. Applications own actions, values, theme selection,
images, and texture lifetime. The library adds no window, renderer, or icon font.

```rust
use egui_prism::{button::{Button, ButtonRole, ButtonSize}, theme::Theme};

fn actions(ui: &mut egui_prism::egui::Ui, theme: &Theme, can_save: bool) -> bool {
    ui.add_enabled(
        can_save,
        Button::new(theme, "Save")
            .role(ButtonRole::Primary)
            .size(ButtonSize::Medium),
    ).clicked()
}
```

Use `Button::icon(image)` for a leading image and label. Use
`IconButton::new(theme, image, "Accessible action name")` for an image-only action.
The name is required, exposed through egui widget information, and shown as a
hover tooltip; an empty or whitespace-only name panics as a programmer error.
This metadata does not establish screen-reader acceptance on a particular host.

Images use the host's egui image sources and loaders. Untinted white images follow
the state foreground; explicitly colored images retain their tint. Keep texture
handles alive in the caller. A missing image loader or expired texture is not
replaced by a bundled font or hidden resource manager.

The built-in themes use the [pinned shadcn reference](shadcn-parity.md).
Install the bundled Geist fonts with `theme::install_fonts` and apply the theme
as described in [themes](themes.md). The host can still explicitly override the
button text style or supply `RichText`; these customizations change the reference
appearance without changing the control's fixed height.

| Role | Resting appearance |
|---|---|
| Primary | Primary background and foreground |
| Secondary | Secondary background and foreground |
| Outline | Input border, background, and a small shadow; translucent fill in dark mode |
| Ghost | Transparent background |
| Destructive | Destructive background and white text; 60% background alpha in dark mode |
| Link | Primary-colored text, underlined on hover |

| Size | Height / icon-button width | Text padding per side | Padding with icon | Icon | Gap |
|---|---|---|---|---|---|
| ExtraSmall | 24 | 8 | 6 | 12 | 4 |
| Small | 32 | 12 | 10 | 16 | 6 |
| Medium (default) | 36 | 16 | 12 | 16 | 8 |
| Large | 40 | 24 | 16 | 16 | 8 |

Measurements are logical points at 100% zoom. The Outline border adds one point
per side to a text button's width. Icon buttons remain fixed squares. ExtraSmall
uses 12-point text with the standard theme; the other sizes use 14-point text.
Caller-supplied images fit the reference icon box instead of growing the button.
Long text keeps its intrinsic width; the host owns wrapping the group or providing
a horizontal scroll area. The Gallery demonstrates both.

Hover colors and focus rings transition over 150 ms with the reference easing.
A pointer press preserves the hover appearance. Keyboard focus adds a three-point
outer ring; pointer focus does not. `.invalid(true)` changes the Outline border
and the focus-ring color and exposes invalid metadata. Use `ui.add_enabled(false,
...)` or a disabled parent UI for buttons that emit no clicks.
Styles and opacity are restored for subsequent host widgets.

Enter activates on keydown; Space activates once on keyup after a focused keydown.
Losing focus or disabling the button cancels the pending Space activation.
Tab traverses controls. Once the focus filter is installed, arrow keys and Escape
keep button focus; the acquisition-frame gap is recorded below. Pointer release
inside the button activates through the ordinary `Response::clicked()` contract.
Give reorderable instances stable caller-owned UI scopes. Consume the response
in the current frame; do not retain it as application state. The component stores
only focus modality, pending Space activation and color-transition state under
that instance's ID. Applications retain action and resource ownership.

## Migration from the earlier button implementation

Add `ExtraSmall` and `Link` cases to exhaustive enum matches. Heights and icon
boxes now use the table above instead of scaling `Theme::metrics.control_height`
and `control_padding`. Labels no longer wrap or grow the control vertically.
Replace layouts that depended on automatic wrapping with an explicit scrolling
container or shorter action text. Space-based actions now occur on release.
The constructors, role/size builders and `Response` handling remain unchanged.

The Gallery Buttons case displays roles, sizes, invalid/disabled states, labels,
icons and independent activation counts in both themes. See the
[gallery procedure](gallery.md) for manual input and screenshot checks. Automated
regressions cover geometry, activation timing, cancellation, focus painting,
opacity, identity, accessible names and explicit formatting. Native/browser
observations and untested platforms are reported separately in the PR; metadata
and screenshots do not establish assistive-technology acceptance.

## Remaining parity work

Disabled painting currently applies 50% opacity to each primitive. This differs
from the reference's group opacity where text, images, fill and shadow overlap.
The real native and browser comparisons exposed the difference; per-shape alpha
tests do not establish final compositing parity. Integrating the shared host
compositor and validating arbitrary parent backgrounds and translucent images
remain required before this component is accepted as strictly matching shadcn.

The pinned egui focus API cannot install a focus filter on the pass that first
acquires focus. If Escape or an arrow arrives in the immediately following frame,
before another pass installs the filter, default egui navigation can move focus.
Restoring focus from inside the widget also overrides deliberate caller focus
release, so that workaround is not used. Closing this acquisition-frame gap at
the input/framework boundary remains required; ordinary Tab traversal and caller
focus ownership must remain intact.
