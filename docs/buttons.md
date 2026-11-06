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

| Role | Resting appearance |
|---|---|
| Primary | Main action color pair |
| Secondary | Muted surface and foreground |
| Outline | Surface pair with a border |
| Ghost | Transparent background with surface foreground |
| Destructive | Destructive pair, chosen by the application |

Hover adds a visible outline (and a muted background for Ghost); pressed and
keyboard-focused buttons use the focus pair. Small, Medium, and Large scale the
theme's padding and minimum height by 0.875, 1, and 1.25. Text remains at the host's
button font size. These are minimum heights: long labels wrap and grow vertically.
Icon buttons reserve a minimum square hit area; large image content may grow it.
Custom palettes remain responsible for readable foregrounds on their host surfaces.

Use `ui.add_enabled(false, ...)` or a disabled parent UI. Disabled buttons retain
egui's standard opacity treatment and emit no clicks. Role styles are local to
the button and do not change subsequent host widgets. Existing egui IDs, pointer
activation, Tab traversal, Enter/Space activation, and response helpers remain
available. Give each reorderable instance a stable caller-owned UI scope, and
consume `clicked()` only during the current frame; do not persist the response.

The Gallery Buttons case displays roles, sizes, labels, icons, and independent
activation counts in both themes. Use its existing narrow-content and zoom
controls for layout acceptance. See [the gallery procedure](gallery.md) for
manual input and screenshot checks. Automated tests exercise the public widgets;
actual native/browser observations and untested platforms are reported in the PR.
