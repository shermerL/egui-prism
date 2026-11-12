# Ordered input integration

The controlled egui 0.37.0 patch provides current-pass focus policy, ordered event
routing and composite capture for M1 consumers. The core remains egui-only;
window creation, frame execution, application state and official renderer/platform
integration belong to the host. Existing components require their own migrations
and parity acceptance before opting into this mode.

See the [integration boundary assessment](integration-boundaries.md) for the
source requirements, public-API alternatives and unresolved equivalence work.

## One source per application

Cargo patches apply at the consuming application root, not in a dependency's
manifest. Every application using this capability must select this source:

```toml
[dependencies]
egui = { version = "=0.37.0", default-features = false }
egui-prism = { path = "path/to/egui-prism/crates/egui-prism" }
egui-winit = "=0.37.0"
egui-wgpu = "=0.37.0"

[patch.crates-io]
egui = { path = "path/to/egui-prism/vendor/egui" }
```

Pin the repository revision in your source checkout and retain the vendored
licenses. Do not patch epaint, egui-winit, egui-wgpu or wgpu. The standalone
`tests/input-consumer` package compiles platform Context, renderer primitives and
an optional official system font provider across these boundaries. Its dependency
graph check requires exactly one egui and one epaint. This is not a published
package configuration, renderer fork or platform runtime test.
The same check validates the actual workspace graph, including the native Gallery
and compositing adapter, against the vendored egui and official integration crates.

## Host-owned phases

Set `ctx.options_mut(|options| options.ordered_input = true)` once when creating
a Context. The default remains the ordinary egui execution model. Ordered mode
requires every application viewport callback to complete the explicit phases:

1. Call `ctx.begin_input()` once inside `run_ui`, before registering controls.
2. Register the current layout using ordinary egui widgets or `ui.interact`.
   This phase must only lay out controls, register policies and discover geometry.
   Apply business actions only in the event phase. Widget responses do not report
   activations during registration.
3. Call `ctx.next_input_event()` until it returns `None`. Each result contains an
   explicit `InputTrigger` and its authoritative target ID. An `Event { index,
   event }` trigger carries one original event and its position in the stream
   left by plugins; the handler sees that event in `InputState.events`. A `Time`
   trigger delivers native long-touch or drag transitions, or an untargeted
   time-consumer opportunity, with an empty event stream. Run the target's widget
   handler against its registered rectangle. Apply
   resulting application state changes before requesting another event. Untargeted
   events still reach the host through the returned event, including navigation
   already handled by the dispatcher and application/platform events.
4. Paint the resulting state after the iterator is exhausted. Activation, drag
   transition and pointer-coordinate response fields are suppressed during paint;
   hover, pointer-held appearance and keyboard focus remain available. Do not
   execute a second business-input path from aggregate pointer state during paint.

The native Gallery uses these phases within its existing single `run_ui`. It
keeps actual response IDs, local control layout and measured scroll shells for the
current pass. A page, theme or width change refreshes the corresponding layout.
Scroll updates and editor changes refresh the affected case before the next event.
Compositor result collection and restart belong to the update, and screenshot
requests use the final drawing geometry.

A future popup consumer can inspect the raw release in this same loop, including
an untargeted release. Commit an action only if its live response is enabled and
contains the pointer.

A compiling host example is in
[`ordered_input.rs`](../crates/egui-prism/examples/ordered_input.rs). It demonstrates
registered geometry, ordered actions and final paint without a second whole-UI
pass or a second retained widget tree. Registration and targeted handling may
still perform local widget layout; this is a migration cost, not a claim of
zero additional work. No performance advantage is claimed.

The dispatcher advances frame time once before processing the raw batch. Any
resulting interaction transitions receive `InputTrigger::Time` handling first,
including on frames with no raw events. Each affected owner is offered one handling
opportunity, subject to current visibility, enablement and modal eligibility.
Time triggers do not invent pointer events or consume a raw-event index. The final
paint suppresses their action flags just as it suppresses raw-event activations.

Register a live custom time consumer with `ctx.request_input_time()` after
`begin_input` and before the first `next_input_event` of each frame in which it
needs time handling. Repeated requests and all `ScrollArea` registrations share
at most one untargeted `Time` opportunity per viewport frame, including frames
without raw input. Without either kind of request there is no untargeted
opportunity; native targeted transitions remain independent. The host forwards
untargeted `Time` to its active consumers, which own their deadlines and
cancellation. This is registration only: no consumer registry, background timer
or business action is added. `request_repaint_after` schedules a future frame but
does not itself request ordered time handling.

`request_input_time` asserts in ordinary mode, before `begin_input`, during
event handling or after dispatch has finished. Calls inside `input_layout`
after dispatch began safely do nothing, so refreshing a live consumer cannot
rearm the opportunity. Discard continuations are layout only: requests do
nothing and cannot consume a second frame-time budget.

`RawInput.events` stays available for observation in each phase. The live
`InputState.events` stream is empty during layout/paint and contains only the
current event during raw-event handling; time handling has no synthetic event.
Plugin `input_hook` and `on_begin_pass` retain
normal ordering and see the incoming batch. A plugin's deliberate consumption in
`on_begin_pass` is honored. At `on_end_pass`, original raw events and final aggregate
pointer/key state are retained, while the consumed event stream stays empty.
Interactive plugins must join the host's registration/event/paint flow; a legacy
plugin that performs actions only in `on_begin_pass` is not an ordered consumer.

Built-in Label selection handles remaining events after each target handler, before
the next event changes the selection. Copy/Cut emits an owned text snapshot at
that point, including selections spanning several Labels; Escape cancels the
selection. The selection plugin retains current-pass `Arc<Galley>` text sources,
IDs, text transforms, clip rectangles and local multi-widget selection policy.
These are not another hit tree: eligibility and pointer containment still come
from current egui widgets. Widget re-registration revokes the previous text
source until a selectable Label renews it, including when the same ID remains
visible and enabled. Event finalization removes unrenewed or vanished sources
and invalidates their selection endpoints. `Response::interact` appends interaction
without replacing a text source, including for the same sense or a response read
with `Context::read_response`. Ui geometry finalization preserves appended senses
while applying current enabled/visible state. Neither update can renew a text
source revoked by a real replacement. Sources are cleared at input begin/pass
end; closed viewports are pruned. Registration and final painting do not process
selection input again. Keyboard changes preserve the active endpoint and local
selection policy. Pointer release ends dragging while retaining a word/line anchor;
each new press updates its granularity or clears it using the stock selection rules.

Remaining key events are saved privately until the stock frame-level zoom/quit
step, after built-in plugin processing. Handler consumption remains authoritative;
beginning-of-pass consumption is never reconstructed from raw input. Zoom keeps
stock reset priority, repeat coalescing and next-pass application. Ordered mode
does not make arbitrary legacy `on_end_pass` input plugins compatible.

`MiddleClickPaste` uses its own position and current geometry, including for an
unfocused stock TextEdit. Its target and response containment agree; the editor
owns insertion and focus. Ordinary keyboard paste retains focus routing.

Drain the input cursor even when no event is relevant. Failing to begin or finish
an enabled pass is a programming error and asserts. Existing egui discard passes
remain supported: raw events are taken once by `run_ui`, so a caller-requested
second layout pass does not repeat actions. One-pass operation is independently
tested; `request_discard` is not a prerequisite for input correctness.

## Local scrolling

Register each live `ScrollArea` with its normal scroll sources. When a `Time`
trigger arrives, run the live local scroll shells so inertia, spring motion and
pending wheel smoothing can advance, including on frames with no raw events.
On `MouseWheel`, run the relevant nested shell chain in its normal inner-first
consumption order. Scrollbar/background drag recipients likewise run their local
shell. Shells declare the real content extent; they need not traverse the contents.
Refresh affected control rectangles and clips with `input_layout` before asking
for the next event. The example demonstrates a wheel followed by a click using
updated geometry in the same callback.

Registration, layout refresh and final paint do not consume scroll input or
integrate physics. Each container privately remembers its last input opportunity,
so repeated shell calls, including targeted pointer `Time` deliveries, cannot
repeat its interaction. A discard continuation only remeasures layout; it does not
release wheel carry or spend another frame budget. Explicitly restoring a public
`scroll_area::State` preserves the destination Context/ID's consumption token;
imported state cannot transfer or reset that authority. Removed containers have
no update callback, and registration requests no scrolling opportunity when no
containers remain. Native persisted offsets retain their existing lifetime.

Old wheel carry is smoothed once at the frame-time opportunity using the existing
repaint-aware `stable_dt`. Each new wheel impulse receives its initial slice with
that same smoothing factor, without smoothing old carry again. Pointer/key events
do not release even sub-point carry. Native momentum uses the frame dt for its
velocity calculation, while physics integration and target animation run only at
the time opportunity. The stock `WheelState` and `AxisPhysics` math is unchanged;
ordinary, non-ordered execution retains its original algorithm.

Ordered input preserves causal effects rather than promising the ordinary mode's
whole-batch coalescing for every mixed batch. For example, `wheel(-40), click, End`
releases the wheel's first slice and routes the click at its updated rectangle;
End cancels future carry, without undoing that click or offset. Ordinary mode can
coalesce End with the wheel before any widget sees it. Reversed impulses, changes
of source/modifiers and the native sub-point flush threshold also obey event
order. Each impulse uses the current native gesture classification; future carry
retains native last-source/modifier state. Simple isolated wheel input, empty-frame
release and isolated inertia/spring trajectories are compared with ordinary mode.

Nested areas retain stock whole-axis consumption: an inner area that consumes an
axis clears it before its parent runs; this is not proportional overflow sharing.
Clips, transformed layers, top Areas and modal barriers remain authoritative.
Ordered Area containment uses the same current layer order as event routing, so
same-batch `move_to_top` changes wheel ownership immediately.
Stock scrolling is not blocked merely by a same-layer child button. No separate
scroll hit tree, retained container registry or public phase query is introduced.

## Focus, geometry and capture

Register `memory.register_focus_policy(id, layer, filter)` for each enabled
control before dispatch. Policies belong to a live widget in one viewport and
must be renewed every pass. Current focus setters can update the owner's policy
without waiting for a previous-frame focus record. Stock TextEdit registers its
own policy in ordered mode. Tab, Shift+Tab, directional navigation, Escape and
AccessKit focus requests run in event order against current eligible controls.
Registration keeps a blocked control's policy available if a modal closes later
in the same pass. Directional navigation uses the newly eligible rectangles.
`SurrenderFocusOn` preserves its configured press, completed-click or never
semantics. A newer host focus decision during a press/release sequence takes
precedence over surrender on that release, including in stock TextEdit handlers.

For controls such as Slider, register rail and thumb independently, then call
`ctx.set_input_owner(rail_id, thumb_id)`. They must be in the same layer, and
owners cannot themselves be parts. Hit-testing runs on each part's actual clipped
geometry before mapping the result to the semantic owner. egui capture holds
the owner ID; its rectangle remains the navigation rectangle. This does not create
a clickable enclosing rectangle. Renew bindings each pass.

Call `ctx.set_input_rounding(id, radius)` after registering a rounded part. The
radius is in local layer coordinates and clamped to half the shortest side.
A square with that radius is a circular thumb. Clipping, transforms, layer order
and overlapping widgets still participate in the normal authoritative hit test.
Release reaches the capture owner even outside the shape, but cannot activate a
click outside the owner's hit parts. Release never requests focus: an explicit
host surrender or transfer stays effective.
When unmodified Escape performs focus navigation and stops a drag, its target is
the drag owner so that handler can observe `drag_stopped()` before the next event.
Focus surrender still occurs, even if another control held keyboard focus.

Disabled, disappeared or modal-blocked owners cancel egui capture before the
next event, including on empty frames. Pointer disappearance/window focus loss
cancel egui capture. Window-outside event delivery and OS-level pointer capture
remain the official platform host's responsibility and require real platform
acceptance. Geometry and owner bindings are pass-local. A viewport's egui
input, focus policy and composition state are isolated; official deferred viewport
removal/recreation and a fresh Context are tested independently of native windows.

## Structural changes and text

Use `ctx.input_layout(|| ...)` when an event changes layout. It temporarily
suppresses input responses while registering the changed structure and restores
the active event afterward. Use `ctx.remove_input_widget(id)` for a removed control (including its bound
parts), or `ctx.remove_input_layer(layer)` when closing or replacing a layer,
before dispatching another event; it removes registered
geometry and clears that layer's focus/capture. Explicit layer removal also revokes
its current and prior-pass Area visibility, so a closed Area cannot block later
wheel input. Stored Area geometry and order remain reusable; normal registration
restores visibility. The host owns modal state and
chooses whether to restore an older modal or host focus.

A new automatically sized modal needs measurement before its first target can
receive input. `Modal::register` measures and places the stock modal using a
layout-only `FnMut` content callback, which may execute twice on first display.
It does not execute another application `run_ui` pass. Use ordinary `Modal::show`
for a targeted handler and final paint. A first-show sizing pass is never treated
as interactive geometry. The modal regressions use natural stock button layout,
leave a modal open across frames and execute both layout orders.

Stock TextEdit can handle targeted events with its stable ID and registered
rectangle, then paint after dispatch. Selection, copy, undo, synthetic preedit /
commit, first-focus arrows and focus interruption are regression-tested. A
composition is bound to the viewport's focus owner and interruption generation;
a terminal event from an interrupted composition is observable but cannot edit a
new focus owner. An empty preedit retains the previous owner for a following
commit, including across frames; commit clears that binding. A new nonempty
preedit after the clear starts a new binding for the current owner. Raw IME
contains no independent native session ID, so native integrations must preserve
the platform's ordered composition/termination sequence.

## Verification and maintenance

Run the repository checks through `nix develop --command quest gate python3
scripts/check.py`. They include source reconstruction, stock egui unit tests,
framework regressions and the independent consumer. The patch inventory and
source checksum are in `vendor/README.md`; `vendor/egui.patch` is the reviewable
difference from the official crate, including explicit non-publication metadata.

The automated evidence is synthetic input and compilation. It does not establish
macOS, Windows or Linux native input/IME, accessibility, clipboard or GPU playback
acceptance. The native host and component Quests own those end-to-end checks.
When upgrading egui, reconstruct the new upstream source, review each boundary,
port the patch and run all regressions. Remove a patch only after the unpatched
upstream passes its relevant contract; do not retain an unnecessary workaround.
