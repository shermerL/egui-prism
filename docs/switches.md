# Switch

`switch::Switch` edits a caller-owned boolean. It borrows the value and theme
for one call and does not start application work. Use a persistent `.id(...)`
for reorderable instances and reuse that ID in both layouts of an ordered pass.

A nonempty label is required. The rounded track and rectangular label area
(including the gap) register separate hit geometry with one owner, one Tab stop
and one accessible Switch node. The label uses a 14 point Geist face, a 20 point
default line height, and an 8 point gap. The track stays centered beside wrapped
text. Rich text and caller font families remain supported. Install the reference
fonts with `theme::install_fonts` for comparisons.

| State | Pinned reference contract |
| --- | --- |
| Default size | 32 × 18.4 point track, 16 point thumb, 14 point translation |
| Small size | 24 × 14 point track, 12 point thumb, 10 point translation |
| Off | Input fill; dark input uses 80% of its token alpha |
| On | Primary fill; dark thumb uses primary foreground |
| Hover and press | No additional fill or outline |
| Keyboard focus | Ring-colored 1 point border and 3 point outer ring at 50% alpha |
| Pointer focus | Focus is retained without the keyboard ring |
| Disabled | No action or focus; the complete control is composited at 50% opacity, and its label stays independent |
| Invalid | `.invalid(true)` exposes invalid accessibility semantics without adding a visual style |

Pointer release activates an enabled Switch when the press belongs to it and the
release is on its hit area. This includes a long mouse hold. Clipping, rounded
corners, cover and capture come from the ordered egui dispatcher. Enter
activates on keydown, including repeat. Space activates once on keyup and a loss
of focus cancels an armed Space. Arrow keys and Escape retain focus. Tab and
Shift+Tab navigate. Accessibility Click uses the same value change path.

Track color, control opacity, focus and thumb position use the reference
150 ms cubic-bezier(0.4, 0, 0.2, 1) transition. Track colors interpolate in
premultiplied Oklab. Transitions reverse from their current value. The first
render starts at the caller's value. Interaction and animation state are
temporary and scoped to the widget ID.

## Migration to ordered input and explicit painting

Replace `ui.add(Switch::new(...))` and `ui.add_enabled(false, switch)` with the
following stages. See [ordered input](input-compatibility.md) for host setup and
[host compositing](compositing.md) for renderer setup.

1. Enable `Context::options_mut(|o| o.ordered_input = true)` at context creation
   and install a compositor. After `begin_input`, build the active layout with
   `Switch::register`. Retain the returned `SwitchRegistration` for this pass.
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
4. Once the cursor is exhausted, lay out the final frame with `Switch::show`.
   Submit its `SwitchOutput` with `.paint(&painter)` or include its drawing in a
   parent `Group`. The output's response describes the final presentation. The
   changed responses come from event handling.

For example, the following code handles one switch in an already registered
layout. A full pass is demonstrated in the `Switch` Rust API documentation.

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
Switch groups the shadow, ring, track, thumb and focus border before applying
disabled opacity, then draws the label in the enclosing group. The host's
painter opacity applies once to that enclosing group. A visible output submitted
without an installed compositor returns an explicit error.

The default labeled row is 20 points high and expands for wrapped text. The track
does not scale with `theme.metrics.control_height` or `item_spacing`. Select
`SwitchSize::Default` or `SwitchSize::Small` instead of assuming the old 36 × 20
track and 32 point row.

## Reference and validation

The source is `new-york-v4/ui/switch.tsx` at shadcn commit
`8a3536678a528cc012b9e62484290e79d0be410f`. The fixture locks
`@radix-ui/react-switch` to 1.2.6. `scripts/check_reference.py` checks the pinned
component source and font hashes. The React/Radix comparison fixture includes
controlled values, counters, sizes, invalid and disabled states, reordered
instances, long labels and compositing samples.

The public behavior tests cover ordered activation, occlusion, focus modality,
instance identity, geometry, accessibility metadata, group construction and
transitions. Browser, native window behavior, assistive technology, independent
review and CI each have separate evidence under the
[strict parity contract](shadcn-parity.md).
