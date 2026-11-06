# Switch

`switch::Switch` edits a caller-owned boolean. It borrows the value and theme for
one frame and returns an egui response. It does not own a setting, save state,
start a network request, or treat a toggle as successful completion of an
application operation.

```rust
use egui_prism::{egui, switch::Switch, theme::Theme};

fn preview_setting(ui: &mut egui::Ui, theme: &Theme, enabled: &mut bool) {
    if ui.add(Switch::new(theme, enabled, "Show previews")).changed() {
        // The application decides whether and how to apply the new value.
    }
}
```

A nonempty label is required. The track and label share one hit target. Pointer
activation and native egui Enter/Space activation toggle once, marking the
response changed; idle frames do not repeat changes. Use `ui.add_enabled(false,
switch)` to disable interaction. Focus and pointer press have a visible track
outline; the thumb position and track color distinguish on and off states.

Theme metrics determine track, thumb, gap, and minimum height. Labels wrap inside
the remaining width and may increase the control height. The same allocated
rectangle defines painting and hit testing. The theme's appearance and typography
apply during this call; host font families and font/style overrides are retained,
explicit rich-text formatting is supported, and subsequent controls retain the
caller's style. Disabled rendering uses egui opacity.

Use stable caller UI scopes for reorderable instances. Values remain owned by the
application when controls disappear. The switch exposes the Switch role, label,
and boolean state through egui accessibility metadata. That metadata is not proof
of screen-reader support on a particular host.

The Gallery Switches case includes independently counted values, disabling,
reordering, and long labels. Automated input, geometry, and metadata regressions
and actual native/browser observations are listed separately in the PR.
