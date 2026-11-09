# Checkbox

`checkbox::Checkbox` edits a caller-owned boolean. It borrows the value and theme
for one call and does not start application work. Use a persistent `.id(...)`
for reorderable instances and reuse that ID in both layouts of an ordered pass.

A nonempty label is required. The rounded box and rectangular label area
(including the gap) register separate hit geometry with one owner, one Tab stop
and one accessible Checkbox node. The label uses a 14 point Geist face, a 20 point
default line height, and an 8 point gap. The box stays centered beside wrapped
text. Rich text and caller font families remain supported. Install the reference
fonts with `theme::install_fonts` for comparisons.

| State | Pinned reference contract |
| --- | --- |
| Geometry | 16 × 16 point box, 4 point corner radius, 1 point inside border, 14 point check icon |
| Unchecked | Transparent light fill; dark input uses 30% of its token alpha |
| Checked | Primary fill and border, primary foreground check |
| Hover and press | No additional fill or outline |
| Keyboard focus | Ring-colored border and 3 point outer ring at 50% alpha |
| Pointer focus | Focus is retained without the keyboard ring |
| Disabled | No action or focus; the completed control is composited at 50% opacity, and its label stays independent |
| Invalid | Destructive border and focused ring at 20% alpha in light mode, 40% in dark mode; checked primary border takes precedence |

Pointer release activates an enabled Checkbox when the press belongs to it and the
release is on its hit area. This includes a long mouse hold. Clipping, rounded
corners, cover and capture come from the ordered egui dispatcher. Enter does not activate. Space activates once on keyup and a loss
of focus cancels an armed Space. Arrow keys and Escape retain focus. Tab and
Shift+Tab navigate. Accessibility Click uses the same value change path.

The focus shadow uses the reference 150 ms cubic-bezier(0.4, 0, 0.2, 1)
transition. Checked colors, indicator visibility and disabled opacity change
immediately because the source specifies `transition-shadow`. The first render
starts at the caller's value. Interaction and animation state are temporary and
scoped to the widget ID. The API remains boolean; no indeterminate state is exposed.

## Migration to ordered input and explicit painting

Replace `ui.add(Checkbox::new(...))` and `ui.add_enabled(false, checkbox)` with the
following stages. See [ordered input](input-compatibility.md) for host setup and
[host compositing](compositing.md) for renderer setup.

1. Enable `Context::options_mut(|o| o.ordered_input = true)` at context creation
   and install a compositor. After `begin_input`, build the active layout with
   `Checkbox::register`. Retain the returned `CheckboxRegistration` for this pass.
2. Drain `next_input_event`. For a matching `event.target`, call
   `registration.handle(&event, &mut value)` once before requesting the next
   event. Consume that response's `changed()` immediately. Two activations in
   one pass yield two changed responses, even if the value returns to its
   original state.
3. Call `registration.cancel_unfocused()` when its focus is lost, including
   when the window loses focus or the owner is disabled or removed. Observe the
   loss before any subsequent focus gain. If structure changes within a pass,
   refresh affected geometry with `input_layout` and remove vanished owners with
   `remove_input_widget` before routing the next event. Removing the owner also
   removes its bound label part.
4. Once the cursor is exhausted, lay out the final frame with `Checkbox::show`.
   Submit its `CheckboxOutput` with `.paint(&painter)` or include its drawing in a
   parent `Group`. The output's response describes the final presentation. The
   changed responses come from event handling.

For example, the following code handles one checkbox in an already registered
layout. A full pass is demonstrated in the `Checkbox` Rust API documentation.

```rust
if event.target == Some(registration.response.id) {
    let response = registration.handle(&event, &mut show_previews);
    if response.changed() {
        // Apply the new value in the application.
    }
}
registration.cancel_unfocused();
```

Use a disabled child Ui for both layouts. Capture the same-layer painter
**before** disabling that child so it supplies the actual parent opacity.
Checkbox groups the shadow, ring, box, border and check before applying
disabled opacity, then draws the label in the enclosing group. The host's
painter opacity applies once to that enclosing group. A visible output submitted
without an installed compositor returns an explicit error.

The default labeled row is 20 points high and expands for wrapped text. The box
stays 16 × 16 points regardless of `theme.metrics.control_height` or host spacing.
The constructor now accepts a shared boolean borrow; only `handle` requires its
mutable borrow. Application values remain owned and retained by the caller.
Explicit rich-text sizes and italics can extend beyond the default line box.
The compositing bounds include that ink while layout and hit geometry keep the
same row. The parent clip still limits all drawing.

## Reference and validation

The source is `new-york-v4/ui/checkbox.tsx` at shadcn commit
`8a3536678a528cc012b9e62484290e79d0be410f`. The fixture locks
`@radix-ui/react-checkbox` to 1.3.3 through the checked-in package lock. `scripts/check_reference.py` checks the pinned
component source and font hashes. The React/Radix comparison fixture includes
controlled values, counters, invalid and disabled states, reordered
instances, long labels and compositing samples.

The public behavior tests cover ordered activation, occlusion, focus modality,
instance identity, geometry, accessibility metadata, group construction and
transitions. Browser, native window behavior, assistive technology, independent
review and CI each have separate evidence under the
[strict parity contract](shadcn-parity.md).

## Evidence and limits

The implementation at `c25b009` passed 27 Checkbox behavior tests, six affected
Select trigger drawing tests, the native Gallery Checkbox routing test, the host
scroll check, library/Gallery clippy, documentation tests, the native build,
pinned reference checks and reference build. Its
[CI run](https://github.com/shermerL/egui-prism/actions/runs/42648094912)
also passed. The rich-label correction at `0a0dd68` adds a regression that first
failed on the previous head; all 28 Checkbox tests, focused clippy and the native
Gallery build then passed. It changes no vendor, host input or renderer code.

Actual macOS Metal Gallery and Chrome reference observations covered light and
dark unchecked/checked states, label activation, Space and Enter, invalid checked
border precedence, disabled safety, reordered instances, narrow long labels and
compositing samples. The corrected native build was also compared at 150% scale
with Chrome at 150% zoom, including both themes and disabled long labels in
320-point content. Chrome retained 16 by 16 CSS-pixel boxes. The rich-font ink
case is covered by shape and group bounds assertions, not a native screenshot.

These observations compare geometry, state and interaction; they do not assert
pixel-identical screenshots across browser and native rasterization or display
color management. Windows/Linux GUI, native assistive technology and browser
product rendering were not exercised by this component acceptance. Accessibility
metadata tests do not replace an assistive-technology check.
