# Slider

`slider::Slider` borrows a caller-owned `f64` and theme for one frame. It implements
the pinned shadcn single-thumb horizontal LTR slider using egui input, drawing,
focus and accessibility services. The component does not own settings, start work,
or apply business policy.

```rust
use egui_prism::{egui, slider::Slider, theme::Theme};

fn amount_control(ui: &mut egui::Ui, theme: &Theme, amount: &mut f64) {
    if ui.add(Slider::new(theme, amount, "Amount").range(0.0..=100.0).step_by(5.0)).changed() {
        // The caller applies the changed amount.
    }
}
```

## Values and interaction

The default inclusive range is 0 to 1 and the default step is 1, matching the
reference primitive. Use `.step_by(0.01)` for fractional editing of that default
range. Values and bounds must be finite, bounds must increase with a finite span,
and a configured step must be positive and finite. Invalid configuration is a
programmer error. Steps are anchored at the range start. Edits snap to the step
grid first and clamp to the bounds second. Consequently End can land below the
maximum when it is closer to that grid point (0 to 1 with step 0.3 ends at 0.9).
An additional increasing step reaches the clamped maximum.

- Right and Up add one step; Left and Down subtract one.
- Shift plus any arrow, PageUp and PageDown move ten steps.
- Control, Alt and Command do not change the step multiplier.
- Home and End submit the minimum and maximum through the same snapping rule.
- Pressing the thumb focuses it without changing its value. Moving the captured
  pointer maps its position over the entire rail, even beyond its bounds.
- Pressing the track maps immediately. Releasing ends that pointer interaction.

Merely displaying an out-of-range caller value does not rewrite it: the thumb
displays the nearest endpoint until the next edit. Disabled calls reject pointer,
keyboard and accessibility writes, including while a drag is in progress.
`Response::changed()` reports value changes, without repeating them on idle frames.

## Geometry and styling

The rail is 6 points high with a 3-point radius; the circular thumb is 16 points
across with a white fill, a 1-point primary border, and the reference small shadow.
Hovering the thumb or focusing it by keyboard shows the 4-point neutral ring.
Pointer focus alone does not show the keyboard ring. Disabled controls currently apply 50% opacity to individual drawing primitives.
CSS group compositing parity is an open acceptance gap where the rail, thumb and
shadow overlap. Sizes are logical points, scaled by the host at 100% or 150% zoom.

For rail width W and normalized value t, the pointer value uses x/W and the
painted thumb center uses `8 + (W - 16) * t`. The range fill ends at `W * t`.
These mappings intentionally differ, as in Radix; clicking an inset endpoint
position while the thumb is elsewhere need not select the exact endpoint.

The host's `ui.spacing().slider_width` is capped by available width, with a
16-point intrinsic minimum. Less space overflows that minimum instead of reversing
the thumb travel. The comparison fixture uses the same minimum. At exactly 16
points the thumb stays centered; captured pointer movement and keyboard input
still change the value. This minimum is a Rust layout constraint, not a claim
that unconstrained Radix prevents reversed geometry below its thumb width.

The required label wraps above the control and names it through accessibility
metadata. The returned response combines track and thumb interaction and retains
the thumb's focus ID. The label is outside the control's hit target. The initial
API has no inline numeric editor, multiple thumbs, vertical or RTL mode. Display
the value separately when useful. The widget leaves the host style unchanged.
Use stable caller UI scopes for reorderable instances.

## Migration from the draft candidate

The constructor, range and step builders are unchanged. The unpublished draft
used egui's native slider, automatic steps, inset pointer mapping and host-dependent
handle sizes. Specify a fractional step explicitly when needed; account for the
16-point intrinsic width, the new whole-rail mapping and reference keyboard rules.
Do not derive pointer positions from the thumb's inset travel.

## Comparison and evidence

The Gallery Sliders case includes two ranges, steps, change counters, disabling,
reordering, long labels and 448/200/20/16-point rails. Its theme and scale controls
exercise light/dark and 100/150% states. The real React reference is available at
`/?sliders` when serving `reference/`, with matching ranges and width choices.
It uses shadcn `8a3536678a528cc012b9e62484290e79d0be410f` and the locked Radix Slider
1.3.6 primitive. See the [parity contract](shadcn-parity.md).

Automated checks, browser observations and native observations are separate PR
evidence. Accessibility metadata is not screen-reader acceptance. Other operating
systems require their own runtime validation.
