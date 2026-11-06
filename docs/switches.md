# Switch

`switch::Switch` edits a caller-owned boolean and returns an egui response. It
borrows the value and theme for one frame. Consume `Response::changed()` to apply
application policy; a toggle does not imply that an asynchronous operation has
succeeded. Stable caller UI scopes keep focus and animations with reorderable
instances. Removing a control does not discard the caller's value.

```rust
use egui_prism::{egui, switch::{Switch, SwitchSize}, theme::Theme};

fn preview_setting(ui: &mut egui::Ui, theme: &Theme, enabled: &mut bool) {
    if ui.add(Switch::new(theme, enabled, "Show previews")
        .size(SwitchSize::Small)).changed() {
        // Apply the new value in the application.
    }
}
```

A nonempty label is required. The label and track form one hit target and one
accessible Switch node. The label uses a 14 point Geist face, 20 point default
line height, and an 8 point gap, matching the reference fixture's wrapping label.
The track stays centered beside a wrapped label. Rich text and caller font
families remain supported; install the reference fonts with `theme::install_fonts`
for comparisons. The widget restores the caller's style after drawing.

| State | Pinned reference contract |
| --- | --- |
| Default size | 32 × 18.4 point track, 16 point thumb, 14 point translation |
| Small size | 24 × 14 point track, 12 point thumb, 10 point translation |
| Off | Input fill; dark input uses 80% of its token alpha |
| On | Primary fill; dark thumb uses primary foreground |
| Hover and press | No additional fill or outline |
| Keyboard focus | Ring-colored 1 point border and 3 point outer ring at 50% alpha |
| Pointer focus | Focus is retained without the keyboard ring |
| Disabled | No action or focus; track/thumb use 50% opacity, label stays independent |
| Invalid | `.invalid(true)` exposes invalid accessibility semantics; the pinned Switch has no invalid visual rules |

Pointer release inside the control activates it, including long holds and a
press/release delivered in one frame. Releasing outside or disabling during the
press cancels it. Multiple completed clicks in one frame apply in sequence;
`Response::changed()` reports whether any activation occurred, not an event count.
Enter activates on keydown
(including repeated keydown); Space activates once on keyup, without repeating
while held. Losing focus or becoming disabled cancels an armed Space press.
Arrow keys retain focus without changing the value. Tab and Shift+Tab navigate.
Use `ui.add_enabled(false, switch)` or a disabled parent UI to disable interaction.
Accessibility activation uses the same value-change path. Accessibility metadata
is not evidence of screen-reader support on a particular platform.

Track color, opacity, focus, and thumb movement follow the reference 150 ms cubic-bezier(0.4, 0, 0.2, 1)
transition. Track colors interpolate in premultiplied Oklab. Transitions reverse from the current position. Initial rendering starts at the
caller-provided value without an entrance animation. No persistent boolean value
is stored by the component; only interaction and visual transition state live in
egui's ID-scoped temporary storage. The current egui painter applies opacity to individual shapes. CSS applies
opacity to the complete switch group; overlapping shadow, track, and thumb
therefore still differ on disabled controls. This candidate is unfinished until
the shared host compositing capability can apply opacity after group rendering.
Do not use a fixed page color to conceal this difference.

## Migration from the previous candidate

`Switch::new` and `Response::changed()` are unchanged. Switch geometry no longer
scales with `theme.metrics.control_height` or `item_spacing`: choose
`SwitchSize::Default` or `SwitchSize::Small`. Remove host assumptions about the
old 36 × 20 track and 32 point row height. The default labeled row is 20 points
high, expanding for wrapped text. Space activation moves from keydown to keyup,
and pointer focus no longer draws a keyboard outline. These are intentional
behavioral changes to follow the pinned source.

## Reference and validation

The source is `new-york-v4/ui/switch.tsx` at shadcn commit
`8a3536678a528cc012b9e62484290e79d0be410f`; the fixture locks
`@radix-ui/react-switch` to 1.2.6. Component source and font hashes are checked by
`scripts/check_reference.py`. The source remains unmodified in `reference/src/ui`.
The Switch comparison fixture adds controlled values, counters, sizes, invalid,
disabled, reordered instances, and long labels alongside the Gallery Switches
case. It is rendered React/Radix, not an imitation HTML switch.

Public behavior tests cover pointer/keyboard timing, first-disabled-frame safety,
focus modality, instance identity, fixed geometry, accessibility metadata,
wrapping, host-style isolation, and animation interruption. Browser, native,
independent-review, and CI evidence are recorded separately in the PR. Native observations precede the final removal of an experimental mesh; the
final painter still needs fresh native visual acceptance after host compositing
is integrated. The strict
[parity contract](shadcn-parity.md) remains the acceptance gate; passing these tests
does not establish visual or platform acceptance.
