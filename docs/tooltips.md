# Tooltips

Tooltips are plain-text, noninteractive hints attached to ordinary, explicitly
identified triggers. They do not own the trigger, its label, focus policy,
activation, or application value. There is no popup widget, modal barrier,
input capture, overlay service, background task, or component-owned event loop.

## Host contract

Use the workspace's pinned [ordered input integration](input-compatibility.md)
and install its Cargo patch at the application root. Enable `ordered_input`
before the pass. Painting requires an installed [compositor](compositing.md);
the native host remains responsible for frames, GPU resources and submission.

1. Call the host's existing `begin_input` once.
2. Register ordinary triggers with explicit stable IDs and accessible labels.
3. Register hints after their triggers, borrowing text and theme. Capture clean
   painters before disabled scopes. Keep registrations frame-local.
4. Offer **every** routed event to `TooltipRegistration::handle_all`, including
   untargeted Time, pointer releases, window blur and other widgets' events.
   One explicitly supplied slice is the provider scope. Use the complete live
   scope each time; keep its membership consistent except for explicit retirement.
5. Continue ordinary targeted dispatch with the **same** event. Tooltip observes
   input but never consumes keys, wheel events or activation.
6. Handle host scrolling. Call `dismiss` when an ancestor actually scrolls, or
   retire and rebuild affected geometry before the next routed event.
7. After draining the cursor, paint ordinary triggers and call each hint's
   `paint`. Do not register again during final painting.

A trigger should retain Escape in its own focus policy if Escape is to dismiss
without egui's default navigation surrendering focus. Tooltip does not overwrite
that policy or request focus to repair a host navigation decision.

```rust
use egui_prism::{egui, theme::Theme, tooltip::{Tooltip, TooltipRegistration}};

fn register_and_dispatch(ui: &mut egui::Ui, theme: &Theme) {
    let context = ui.ctx().clone();
    context.begin_input(); // only at the application's existing cursor boundary
    let painter = ui.painter().clone();
    let id = egui::Id::unique("save-action");
    let (_, rect) = ui.allocate_space(egui::vec2(120.0, 36.0));
    let trigger = ui.interact(rect, id, egui::Sense::click());
    trigger.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, "Save"));
    context.memory_mut(|memory| memory.register_focus_policy(
        id, trigger.layer_id,
        egui::EventFilter { escape: true, ..Default::default() },
    ));
    let mut hints = [Tooltip::new(theme, &trigger, "Save changes").register(ui)];
    while let Some(event) = context.next_input_event() {
        TooltipRegistration::handle_all(&mut hints, &event);
        if event.target == Some(id) && context.read_response(id).unwrap().clicked() {
            // Perform the application action, once.
        }
    }
    // The host paints the ordinary trigger separately.
    hints[0].paint(&painter).expect("installed compositor");
}
```

The executable [tooltip example](../crates/egui-prism/examples/tooltip.rs)
includes compositor installation and controlled input/time. Its collector checks
submissions; it is not a GPU renderer. Intended commands:

- `cargo test --locked -p egui-prism --test tooltip`
- `cargo run --locked -p egui-prism --example tooltip`
- Run the existing workspace/Gallery/reference checks for affected integrations.

The targeted Tooltip tests and executable example passed during implementation.
Final workspace and platform results are recorded below.

## Public exports

The `tooltip` module exports:

- `Tooltip<'a, 'response>`: builder borrowing theme/text and a response.
  `new(theme, response, text)` uses the response's stable trigger ID.
  `enabled(bool)` additionally controls hint enablement.
  `delay(Duration)` configures pointer delay; zero is the pinned shadcn default.
  `register(&Ui)` snapshots the response and borrows only text/theme thereafter.
- `TooltipRegistration<'a>`: pass-local authority. Its public `response` is the
  original trigger snapshot, not current event activation. `handle_all` observes
  the explicit provider scope. `output` reads state. `dismiss` closes with exit
  animation and cancels a pending open. `unregister` cancels immediately, including
  animation, before same-batch removal/replacement/hide/disable. It does not remove
  the trigger. `paint(&Painter)` submits the hint and returns `TooltipOutput`, or
  a compositing error.
- `TooltipOutput`: `open` is logical visibility; `content_rect` is the unanimated
  global body rectangle while open or exiting, excluding its decorative arrow.
  It is not input geometry or an activation result.

Each registration belongs to one pass and viewport. Its painter must have the
trigger's context and layer. Enablement combines the explicit flag with the
ordinary response's enablement, visibility, clip and modal eligibility. Empty
text produces no hint. IDs identify triggers, not labels or layout positions.

## Reference behavior

The supplied pinned `registry/new-york-v4/ui/tooltip.tsx` is copied verbatim to
`reference/src/ui/tooltip.tsx`. Its wrapper changes Radix's default 700ms delay to
zero. `reference/?tooltips` uses that source and the installed Radix implementation.

The native observer follows the supplied Radix Tooltip implementation:

- Mouse movement opens once per trigger entry. Touch-bearing raw batches do not
  synthesize hover opening. Pointer-down focus does not open a hint.
- Keyboard/accessibility focus opens immediately; trigger blur closes. Escape,
  trigger click, outside pointerdown and window blur dismiss. Ordinary Enter and
  Space click semantics are observed, never consumed. Custom activation policies
  should call `dismiss` from their own activation handler.
- Opening a peer closes other hints in the explicitly supplied slice. Its scope
  skips delays while open and for 300ms after closing. There is no document-global
  broadcast or hidden provider registry across unrelated host slices.
- Hoverable content remains open. Transit uses Radix's nearest exit side, two
  five-point inward-padded exit points, and the convex hull with the destination
  rectangle. Leaving that polygon closes on the next pointer move, not after an
  arbitrary grace timeout. Transit also suppresses peer pointer opening.
- Scroll dismissal is about an **actual ancestor scroll**, not every wheel event.
  egui has no DOM ancestor scroll notification. Hosts supply it through `dismiss`
  or affected-layout retirement. Changed anchor/viewport geometry also cancels.
- Disabled, fully clipped and disappeared triggers cannot keep an old deadline or
  drawing alive. A skipped whole frame invalidates state on reappearance. A
  same-batch disappearance must use `unregister`, even if the same ID returns
  before the next frame. Mere omission from an event slice is not retirement.

## Drawing and accessibility

The minimum placement API prefers top/center, flips to bottom for space, and
shifts horizontally within the content viewport. There are no configurable side,
alignment, arrow, portal or rich-content APIs. Body text is 12pt with 16pt line
height and 12pt horizontal/6pt vertical padding; radius is 8pt. Like the pinned
Popper wrapper's `min-width: max-content`, the body keeps the text's natural
width, even when it exceeds the viewport. CSS ASCII whitespace is collapsed;
there is no added wrapping, width cap or ellipsis. Collision positioning shifts
the body to the viewport edge. Final shapes and the compositing Group bounds
are clipped to the visible viewport intersection, so offscreen natural width
does not allocate an oversized render target. The accessible description keeps
the complete original text.

The background pair is deliberately inverted: `theme.colors.background.foreground`
is the hint surface and arrow, and `.background` is its text. Body, text and the
rounded diamond arrow are assembled into **one composited opacity group**. The
150ms CSS-ease animation fades and zooms from 95%; entry slides eight points from
the appropriate side. Exit fades/zooms without a slide-out class. Parent painter
opacity is applied once to the assembled group. The drawing-only Tooltip layer
has no hit geometry, including during fade-out, so clicks pass through content.

An open hint appends a description to the trigger's existing AccessKit description
without changing its label or role. Closing restores the captured description.
Register after the host has supplied its metadata; retire before replacing that
metadata in the same batch. The host owns the accessibility bridge. Tooltip does
not make an inaccessible trigger accessible by itself.

## Limits and evidence

This is an egui adaptation, not a browser layout engine. Font rasterization, arrow tessellation and animated
pixels require visual comparison. The scope is explicit rather than Radix's
cross-document event broadcast. With no live scope members, no hidden provider
retains its skip-delay window. Synthetic pointer events lack a pointer type, so
mixed touch/mouse raw batches conservatively suppress hover opening for that batch.
Content hover is geometric and noninteractive; it is not a general overlay
occlusion or interactive popover facility. Do not put links, controls or essential
instructions in these hints. Existing `IconButton` stock hover text is unchanged;
do not attach both tooltip mechanisms to one trigger.

Public tests use mocked `RawInput.time`, not sleeps. They cover delays and skip
windows, peer identity/reordering, focus and Escape, activation pass-through,
bidirectional transit, disabled/missing/clipped cancellation, same-batch retirement,
window blur, touch suppression, scroll ownership, viewport bounds, group opacity
and description preservation. All 13 passed locally on macOS during implementation.

The Gallery Tooltips page supplies two independent activation counters, disable,
hide/reappear, reverse, delayed opening, long text, a permanently disabled trigger
and an outside-action counter. Compare it with `reference/?tooltips` at matching
light/dark theme and 100% scale; additionally inspect native 150%, narrow windows,
viewport edges and scrolling. The macOS/Metal candidate was observed with keyboard focus opening, Escape
dismissal, normal activation counters, long naturally sized content with viewport clipping, light/dark
themes and 100/150% native scale. Chrome at 100% provided the unmodified source
comparison: short body 97.49 by 28 CSS pixels, 10-pixel trigger gap, 8-pixel
radius and 12/16-pixel text. At a 960-pixel viewport the original long hint
remained 1377.4 pixels wide; the native implementation preserves that natural-width behavior and clips
offscreen drawing and Group allocation to the viewport.
These observations are not measured pixel-difference, animation-timing or
screen-reader acceptance. Windows/Linux native observations remain unverified.
