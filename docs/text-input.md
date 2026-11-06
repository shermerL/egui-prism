# Text input

`text_input::TextInput` is a labeled single-line `egui::TextEdit`. It borrows the
application's `String` and theme for one frame. The application owns validation,
submission, network requests, and the lifetime of the value; the widget owns none
of those operations.

```rust
use egui_prism::{egui, text_input::TextInput, theme::Theme};

fn name_field(ui: &mut egui::Ui, theme: &Theme, name: &mut String) -> egui::Response {
    ui.add(TextInput::new(theme, name, "Name").hint("Your name"))
}
```

The required label must contain non-whitespace text. An empty label is a
programmer error. `hint` is transient guidance while the value is empty. `error`
displays a caller-provided validation description without modifying the value.
Errors have a resting outline and visible description; focus and selection retain
their paired theme colors. Error text and invalid state are also attached to egui's
accessibility node when that host integration is active.

The component applies the theme's appearance, metrics, and typography in a local
vertical layout. Host font families and global font/style overrides are retained;
subsequent widgets keep their original style. Labels and descriptions wrap to the
available width; long single-line values scroll within the editor.

Use `ui.add_enabled(false, input)` to disable editing. The returned response is the
edit field's response, with `changed`, `has_focus`, and `lost_focus` referring to
the editor. Consume it in the current frame. Native egui selection, paste, undo,
and IME event handling are retained rather than recreated by the component.

A field derives its ID from the caller's UI scope. Reorderable instances need
stable scopes; alternatively, `.id(egui::Id::new(...))` accepts an application-unique
ID. Reuse that identity to preserve focus, selection, and undo state. Do not assign
the same explicit ID to two live fields. Values remain in the caller even when a
field is hidden or recreated; egui manages transient editor state by ID.

The Gallery Inputs case provides two independent values, an editing disable
control, caller-error and reordering controls, and Unicode inspection. Unicode
escapes distinguish committed values from missing font glyphs during IME checks.
Use the [gallery procedure](gallery.md) for theme, narrow-layout, zoom, and screenshot
acceptance. The gallery does not bundle a CJK font or modify OS input sources.

Automated tests cover editing events, selection/paste/undo, disabled fields,
instance identity, error metadata, narrow long values, and IME preedit/commit events.
Synthetic IME events do not establish actual operating-system or browser composition.
Actual platform observations and remaining IME, font, browser, and assistive
technology gaps are recorded separately in the PR.
