# Text input

`text_input::TextInput` borrows the caller's `String` and `Theme` for one invocation.
The application owns validation, submission, async work and value lifetime. A
stable UI scope or `.id(egui::Id::unique(...))` retains native selection, undo and
composition state when instances move. Never give two live fields the same ID.

## Drawing and migration

TextInput now uses explicit drawing, replacing `ui.add(input)` and
`ui.add_enabled(enabled, input)`. Install the [compositing host](compositing.md),
capture the same-layer painter before disabling a scope, and submit the output:

```rust
use egui_prism::{compositing, egui, text_input::TextInput, theme::Theme};

fn name_field(
    ui: &mut egui::Ui,
    theme: &Theme,
    name: &mut String,
) -> Result<egui::Response, compositing::Error> {
    let painter = ui.painter().clone();
    TextInput::new(theme, name, "Name")
        .hint("Your name")
        .show(ui)
        .paint(&painter)
}
```

For a disabled field, keep `painter` before entering `ui.scope` and calling
`ui.disable()`. Call `show` inside that scope, then `paint` its output with the
saved painter. The editor group already owns its 50% disabled opacity; the
painter applies intentional parent opacity once. `drawing` may instead be moved
into a parent `Group` for overlapping content. Label/error responses are separate
visible text sources and retain the caller's normal UI opacity and enablement.

`paint` consumes the frame-local output and returns the editor response. A missing
or failing compositor is an explicit error, with no fallback painting. A different
Context or layer is a programmer error. The host owns resource and texture lifetime
through submission; the component creates no renderer or frame lifecycle.

In [ordered mode](input-compatibility.md), call `show` during registration and for
the current target's event using its same ID/local layout. Discard those drawings;
submit only the output from final painting after draining `next_input_event`.
The stock editor registers its own focus policy. Hosts must retire disappeared
editor, label and error IDs before the next event. `changed`, `has_focus` and
`lost_focus` always describe the editor rather than the wrapper.

## Reference appearance and editing

The frame follows pinned `new-york-v4/ui/input.tsx`: 36-point height, a 1-point
border, 12-point horizontal padding inside that border, the theme's default
8-point radius, transparent light fill and 30% of the input token in dark mode.
Border and padding impose a 26-point minimum even in a narrower container. Text
uses the host Body family, 14/20-point type/line height at viewport width 768 or
more and 16/24 below it. The breakpoint uses the viewport, not a narrow column.
Long single-line values scroll; placeholder text clips without ellipsis.

Text inputs show the focus ring for both pointer and keyboard focus. The border
changes immediately; color and ring/shadow transitions use the reference's
150 ms timing. Caller `error` adds the destructive border, validation description
and invalid accessibility metadata. Placeholder text is also exposed to AccessKit.
Selected text uses the primary foreground/background pair independently of rings.
The label must contain non-whitespace text; an empty label is a programmer error.

One synchronous frameless `TextEdit::show` supplies editing, selection, caret and
IME visuals. Only that call's same-layer paint-list range is moved into the
returned group, retaining each shape's clip and transform and replacing the
original slots with Noop. No range or paint-list reference survives the call.
Labels, validation messages and neighboring widgets remain outside that capture.
The pinned editor must retain this synchronous shape-only contract on upgrades.

## Evidence and limits

The Gallery Inputs page and `reference/?inputs` provide independent values,
invalid/disabled states, reordering and Unicode inspection. Automated checks cover
reference geometry, selection/paste/undo, first-disabled-frame safety, metadata,
synthetic composition and real ordered dispatch. They do not establish OS IME,
screen-reader behavior or another platform's rendering. Actual native/reference
observations and untested platforms are recorded separately in the PR.
