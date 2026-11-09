# Selects

`Select` is a controlled, single-value component. The caller owns an
`Option<T>`; options use `SelectItem<T>` with `T: Clone + Eq + Hash`. A commit
changes the caller's value only when it differs from the chosen option.
Consume `changed()` on the response returned by `handle` before advancing the
input cursor. Opening, navigating, dismissing, registration, and painting are
not value changes.

## Host requirements

Select requires the workspace's opt-in [ordered input integration](input-compatibility.md).
Install its pinned Cargo patch at the **application root**; a dependency's
`[patch.crates-io]` does not propagate to consumers. Keep egui, the platform
adapter, and the renderer on the unified source described there. Enable
`context.options_mut(|options| options.ordered_input = true)` before the pass.

Painting also requires an installed [compositing backend](compositing.md).
For the native wgpu renderer, the application owns an `egui_prism_wgpu::Host`
and its explicit renderer/resource lifecycle. Keep the installation alive
through painting and rendering. A collecting backend, such as the headless
example's backend, verifies submissions but does not render GPU callbacks.

## Minimal usage

This application-root frame function assumes the compositor is already
installed. The returned output still belongs to the host, which applies
platform output and texture deltas and renders it normally.

```rust
use egui_prism::{
    compositing,
    egui,
    select::{Select, SelectItem, SelectSize},
    theme::Theme,
};

pub fn select_frame(
    context: &egui::Context,
    input: egui::RawInput,
    theme: &Theme,
    selected: &mut Option<&'static str>,
    changes: &mut u64,
) -> Result<egui::FullOutput, compositing::Error> {
    context.options_mut(|options| options.ordered_input = true);
    let items = [
        SelectItem::new("apple", "Apple"),
        SelectItem::new("apricot", "Apricot").disabled(true),
        SelectItem::new("banana", "Banana"),
    ];
    let mut paint_result = Ok(());
    let output = context.run_ui(input, |ui| {
        context.begin_input();
        // Capture before entering any disabled UI scope.
        let painter = ui.painter().clone();
        let mut select = Select::new(theme, egui::Id::unique("fruit"), &items)
            .placeholder("Choose an option")
            .size(SelectSize::Default)
            .width(192.0)
            .invalid(false)
            .register(ui, selected);

        while let Some(event) = context.next_input_event() {
            if select.handle(&event, selected).changed() {
                *changes += 1;
            }
            // Other application dispatch, including background scroll, goes here.
        }
        paint_result = select.paint(selected, &painter).map(|_| ());
    });
    if let Err(error) = paint_result {
        output.drop_without_applying_deltas();
        return Err(error);
    }
    Ok(output)
}
```

The executable [`select` example](../crates/egui-prism/examples/select.rs)
uses the same ordering with deterministic keyboard events. It asserts a commit
and nonempty drawing submissions. `scripts/check.py` runs it with:

```sh
cargo run --locked -p egui-prism --example select
```

In an existing application, integrate this registration and dispatch into its
existing root cursor. Do not put a second `begin_input`/event-drain loop inside
a widget or create another Context for the popup.

## Ownership and ordered dispatch

- Give each instance a stable, explicitly supplied ID. Option values must be
  unique within a list and stable across sorting or filtering. Labels and
  positions are not selection identities.
- A registration borrows the theme and option slice, not the selected value or
  `Ui`. Keep borrowed resources alive for the entire event batch and final
  paint. Do not retain registrations or borrowed UIs in application state.
- Register every live Select before draining input. Offer **every**
  `RoutedInput` to **all** registrations, including events targeted elsewhere,
  untargeted releases, `Time`, pointer departure, and window blur.
- Run those handlers before any background scroll handling or ordinary
  targeted actions. Select consumes popup wheel events, including at scroll
  endpoints. Its modal barrier shields underlying controls; do not replay the
  same event against a guessed underlying target.
- Do not dispatch Select triggers again through the ordinary targeted path.
  Use the handler's returned response for change counters and refresh status
  in that event, not just at the end of the batch.
- Before replacing or removing a live registration during the current batch,
  call `unregister()` while the cursor is live. This includes disable/hide,
  page switches, theme/configuration changes, and changed scroll geometry.
  Register replacement geometry before asking for the next event.
- Paint only after draining input, using the live registration. Do not register
  again or run actions in final Paint. If the host repeats structural layout
  for painting, preserve the trigger's allocated cursor geometry without
  allocating another interactive Select.

The Gallery prepares both themes and every option-list variant in immutable,
frame-local `Resources`. A configuration event can then select a different
borrow without invalidating another registration. Its `SelectCase` retains
only values, counters, and settings; `Live` is explicitly passed through the
Gallery's existing register/event/paint chain. The clean painter is captured
before disabled scopes so disabled-group opacity is not applied twice.

## States and interaction

Default triggers are 36 points high; Small triggers are 32 points. Width can be
explicit (the Gallery uses 192 points). Parent UI enablement controls disabled
state. `invalid(true)` supplies invalid styling and accessibility metadata;
it does not validate or reject application values.

Primary press opens the popup. A focused trigger also opens with Enter, Space,
or vertical arrows. Within the popup, arrows and Home/End navigate enabled
options without changing the selected value; Enter or Space commits. Escape
dismisses and restores trigger focus when that scope still permits it. Tab is
contained while open. Disabled options are skipped. Empty and all-disabled
lists remain dismissible and cannot commit.

Typeahead is case-insensitive, uses visible labels, and expires after one
second. Repeated characters cycle matching enabled options. Closed-trigger
typeahead can change the selected value directly; open-popup typeahead moves
the active option until commit. Long lists support wheel scrolling and timed
scroll-indicator hover. Blur, dismissal, and unregister cancel transient
interaction. A value absent from the current list remains caller-owned; its
trigger label is blank until that value is present again or the caller resets
it. `None` displays the placeholder.

## Evidence and limits

The core API tests and headless example exercise ordered input and drawing
construction. The Gallery's ignored native-adapter tests execute production
`Gallery::ui` with an actual wgpu Host, deterministic time, raw pointer/key
input, and AccessKit requests. They inspect drawing submissions without rendering
them into a framebuffer. They cover pointer clicks and split press/release across
idle frames, same-batch navigation and commits,
independent counts, reordering, disabled/hidden retirement, popup shielding,
reopening/focus, option/theme changes, and idle repaint scheduling after
indicator-hover cancellation. They are included in the existing ignored-test
CI command; their presence is not a claim that they have been run here.

These checks do not establish native OS event delivery, assistive-technology
acceptance, GPU pixel parity, or Windows/Linux GUI acceptance. Compare the
Gallery with the pinned `reference/src/select-cases.tsx` at 100% for reference
geometry. Use native 150% and narrow controls as additional layout checks,
without introducing CSS zoom into the browser reference. See the
[Gallery cases](gallery.md) for manual coverage and platform boundaries.

The implementation at `2e9a0fb` passed the complete local check, 48 Select
behavior tests, and all six ignored Gallery adapter tests on macOS. Actual
macOS/Metal observation covered light/dark triggers and popups, both sizes,
disabled/invalid states, tinted backgrounds, independent reordered values,
outside-click shielding, keyboard navigation, empty/all-disabled lists,
long-list scrolling, and native 150% scaling with narrow content. The pinned
shadcn browser fixture supplied the visual and interaction reference at 100%.
Windows/Linux GUI and assistive-technology acceptance remain unverified.
The narrow Gallery's standard egui configuration labels still wrap poorly;
that presentation work belongs to the existing native Gallery Quest.
