# Buttons

`button::Button` and `button::IconButton` borrow a `Theme` for one frame.
`show(ui)` returns a `ButtonOutput` containing the interaction `response` and an
optional unfaded `drawing`. The caller submits that drawing with `paint`, using
an installed [compositing host](compositing.md). Applications own actions, values,
theme selection, images, and texture lifetime. The core library depends only on
egui; the native adapter is provided by `egui-prism-wgpu`.

```rust
use egui_prism::{button::{Button, ButtonRole, ButtonSize}, theme::Theme};

fn actions(ui: &mut egui_prism::egui::Ui, theme: &Theme, can_save: bool)
    -> Result<bool, egui_prism::compositing::Error>
{
    // Retain actual parent opacity before egui applies its disabled fade.
    let painter = ui.painter().clone();
    let output = ui.add_enabled_ui(can_save, |ui| {
        Button::new(theme, "Save")
            .role(ButtonRole::Primary)
            .size(ButtonSize::Medium)
            .show(ui)
    }).inner;
    Ok(output.paint(&painter)?.clicked())
}
```

The submission painter must use the same context, layer and coordinates. Capture
it before the first disabled ancestor and pass it through to nested callers;
cloning an already disabled painter cannot recover its original parent opacity.
A button's disabled opacity is applied once to the complete drawing, including
text, images, border, shadow and focus ring. Child clips remain attached to the
primitives and also intersect the submission painter's clip. Hidden UI returns
no drawing. A visible drawing without an installed host returns an error.

When overlapping sibling buttons share a parent opacity, consume their `drawing`
values as `compositing::Item::Group` children of one parent `Group`, then submit
that parent once. Do not apply the same parent opacity both to the parent Group
and its Painter. Keep outputs local to the current frame.

Use `Button::icon(image)` for a leading image and label. Use
`IconButton::new(theme, image, "Accessible action name")` for an image-only action.
The name is required, exposed through egui widget information, and shown as a
hover tooltip; an empty or whitespace-only name panics as a programmer error.
This metadata does not establish screen-reader acceptance on a particular host.

Images use the host's egui image sources and loaders. Untinted white images follow
the state foreground; explicitly colored images retain their tint. Keep texture
handles alive in the caller. A missing image loader or expired texture is not
replaced by a bundled font or hidden resource manager. Ready images retain UVs,
tint, background, corner radius and rotation (rotation and rounded corners are
mutually exclusive). While an asynchronous image is pending, its icon box stays
empty; `Image::show_loading_spinner` does not affect this component. Loader
failures show an error glyph. The loader remains responsible for repainting when
loading completes.

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
and the focus-ring color and exposes invalid metadata. Use a disabled UI scope for buttons that emit no clicks, retaining the parent
Painter as shown above. Drawing does not mutate the UI's styles or opacity.

Enter activates on keydown; Space activates once on keyup after a focused keydown.
Losing focus or disabling the button cancels the pending Space activation.
Tab traverses controls. Ordered hosts register the focus filter before dispatch,
so arrow keys and Escape keep focus even immediately after acquisition. Pointer release
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
`Button` and `IconButton` no longer implement `egui::Widget`. Replace
`ui.add(button)` with `button.show(ui).paint(ui.painter())?`; replace
`ui.add_enabled(enabled, button)` with the disabled-scope example above. Install
the compositing host before submission. Constructors, role/size builders and
`Response` action handling remain unchanged.

The Gallery Buttons case displays roles, sizes, invalid/disabled states, labels,
icons and independent activation counts in both themes. See the
[gallery procedure](gallery.md) for manual input and screenshot checks. Automated
regressions cover geometry, activation timing, cancellation, focus painting,
opacity, identity, accessible names and explicit formatting. Native/browser
observations and untested platforms are reported separately in the PR; metadata
and screenshots do not establish assistive-technology acceptance.

## Compositing and ordered-input evidence

Disabled drawing now uses the shared host compositor. The GPU regression renders
actual Button output over light and dark colored backgrounds, comparing it with
ordinary unfaded primitives followed by one framebuffer blend. It covers primary,
outline and destructive roles, translucent icons overlapping fills, labels,
clipping, parent opacity and 1x/1.5x scale. Run the native-adapter test documented
in [compositing](compositing.md); CPU shape tests alone do not establish pixels.
Native observation and remaining platform acceptance are reported in the PR.

Ordered hosts follow the [input phase contract](input-compatibility.md). Register
Button and IconButton in the current layout before dispatch. Forward each routed
event to the corresponding registered output. Cancel pending Space activation
on the previous button when focus or enablement changes. Submit the final drawing
with a Painter captured before entering the disabled scope. Hosts using `show`
with ordinary egui receive the usual aggregate response.
