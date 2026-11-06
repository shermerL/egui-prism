# Slider

`slider::Slider` borrows a caller-owned `f64` and theme for one frame. It uses egui's
native rail geometry, pointer drag, arrow keys, focus, and accessibility metadata.
The component does not own settings, start work, or apply business policy.

```rust
use egui_prism::{egui, slider::Slider, theme::Theme};

fn amount_control(ui: &mut egui::Ui, theme: &Theme, amount: &mut f64) {
    if ui.add(Slider::new(theme, amount, "Amount").range(0.0..=100.0).step_by(5.0)).changed() {
        // The caller applies the changed amount.
    }
}
```

The default inclusive range is 0 to 1. Values and bounds must be finite, bounds
must increase, and a configured step must be positive and finite. Invalid
configuration is a programmer error. Steps use egui's grid anchored at the range start;
range endpoints constrain edits even when they are not grid multiples.

Edits are clamped to the range. Merely displaying an out-of-range caller value
does not rewrite it: the thumb displays the nearest endpoint until the next edit.
Disabled calls reject writes from pointer, keyboard, and accessibility requests.
Read `Response::changed()` to observe edits; idle frames do not repeat changes.

The required label wraps above the rail and names it through accessibility
metadata. The returned response belongs to the rail, not the label. This initial
API is horizontal and single-value, with no inline numeric text editor. Display
the value separately in the host when useful. Host slider width is capped by
available width; theme styling is restored after the call. Use stable caller UI
scopes for reorderable instances.

The Gallery Sliders case has two ranges, step sizes, change counters, disabling,
reordering, and long labels. Automated tests and actual native/browser observations
are reported separately in the PR; metadata is not screen-reader acceptance.
