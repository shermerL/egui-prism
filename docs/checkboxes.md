# Checkbox

`checkbox::Checkbox` edits a caller-owned boolean through native egui checkbox
interaction. The component borrows a theme and value for one frame; it does not
save settings, start asynchronous work, or interpret the application's state.

```rust
use egui_prism::{checkbox::Checkbox, egui, theme::Theme};

fn preview_setting(ui: &mut egui::Ui, theme: &Theme, enabled: &mut bool) {
    if ui.add(Checkbox::new(theme, enabled, "Enable previews")).changed() {
        // React to the new value in the application.
    }
}
```

A nonempty visible label is required. The box and label share one response and
hit target. `Response::changed()` reports the activation that changes the value;
ordinary repaint frames do not repeat it. Native pointer, keyboard focus,
Enter/Space activation, and checkbox accessibility metadata are retained.

Use `ui.add_enabled(false, checkbox)` for disabled controls. Give reorderable
instances stable caller UI scopes so their focus and interaction IDs follow the
same values. The application retains the values when controls disappear.

The theme supplies the appearance, metrics, and typography during the call;
following widgets retain the caller's style. Host font families and global
font/style overrides are preserved, and explicit rich-text formatting remains
available. Long labels wrap within the caller's available width. Disabled visuals
use egui's opacity. No indeterminate state is exposed by this boolean API.

The Gallery Checkboxes case includes two independently counted values, disable
and reorder controls, and long labels. Use the shared theme, 320pt-content, and
scale controls to inspect layout and focus. Automated behavior tests and actual
platform observations are recorded separately in the PR; metadata tests do not
establish screen-reader or untested-platform acceptance.
