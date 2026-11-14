# Integration boundary assessment

This is a source-backed assessment of the controlled egui 0.37.0 integration,
not proof that every extension is unavoidable. It evaluates the source baseline
`5f2c2280d5a4e221b6dd9c9a49d6fafe573c8bfb`. No input or rendering API changes
are proposed here. The [release support contract](releases.md#version-support-contract)
remains authoritative; this document explains its costs and open alternatives.

## Evidence and decision vocabulary

- **Retain** means the present implementation depends on the extension to preserve
  its documented behavior. It is not a claim that upstream can never provide it.
- **Narrow** identifies a responsibility that should stay local or internal rather
  than becoming a general framework API. It does not authorize removing code.
- **Replaceable** requires an equivalent demonstrated public-API implementation,
  including negative cases and lifecycle behavior. No input extension earns that
  classification from API resemblance alone in this assessment.
- **Unresolved** marks a potentially equivalent alternative needing an experiment.

The [source archive and patch inventory](../vendor/README.md) identify the exact
baseline. The matrix links existing patched regression specifications inspected as source.
The separate unmodified-egui experiment below was actually executed; its limited
result does not establish equivalence or non-equivalence of every possible adapter. A patched test passing is not an upstream failure demonstration. In
particular, renaming a missing patched method to make an upstream build fail
would establish source incompatibility, not semantic necessity.

## Necessity and compatibility matrix

Each scenario below is the smallest proposed equivalence challenge, not a claim
that an unmodified-egui reproduction was run. Links name existing observations
and regression cases that an alternative must preserve.

| Family and present disposition | Observable requirement and minimal challenge | Existing evidence and public alternative | Consumer/maintenance cost and exit condition |
| --- | --- | --- | --- |
| Ordered dispatch, `InputTrigger`, phase isolation: **retain**, alternative **unresolved** | In one batch, Arrow, Tab, Arrow must change two different focused controls; two click pairs must reach two current targets. An earlier action may change the tree before the later event. | [`one_pass_arrow_tab_arrow`](../crates/egui-prism/tests/input_dispatch.rs), [`two_clicks_in_one_batch_reach_both_targets`](../crates/egui-prism/tests/input_boundaries.rs). Upstream `RawInput.events`, `Context::run_ui`/`run_pass`, plugins and `request_discard` permit observation and extra passes, but do not alone establish equivalent current-target dispatch without replaying arbitrary application closures. | Host owns registration, targeted handling and final paint for every ordered viewport. Splitting a batch into host frames must be measured against frame-time, output and plugin semantics, not assumed equivalent. Exit only when one-pass and lifecycle cases pass without a second competing hit/capture authority. |
| Current-pass focus policy and interruption generations: **retain** | First Tab then Arrow must use the newly focused editor's policy; surrender during a press must survive its later release; interrupted IME must not commit to a new owner. | [`first_tab_then_arrow_uses_text_edit_policy_before_any_edit`](../crates/egui-prism/tests/input_lifecycle.rs), [`interrupted_composition_cannot_commit_to_new_focus`](../crates/egui-prism/tests/input_lifecycle.rs). Public focus requests and event filters are candidates, but previous-pass policy and ordered current-pass policy are not interchangeable. | Components register policy; host owns focus intent and native composition ordering. Retain per-viewport cleanup. Exit requires first-focus, capture release, modal eligibility and composition interruption equivalence, not only Tab navigation. |
| Composite owners and rounded authoritative hit geometry: **retain**, metadata scope **narrow** | Slider rail and thumb are one semantic owner but the empty enclosing corners are not clickable; release outside a round thumb reaches capture without activating it. | [`composite_rail_and_round_thumb_do_not_swallow_empty_corners`](../crates/egui-prism/tests/input_geometry.rs), [`release_in_a_rounded_corner_cancels_click_but_reaches_capture_owner`](../crates/egui-prism/tests/input_geometry.rs). Public `Ui::interact`, `Sense`, clipping and containment checks can implement parts, but rejecting a response after native target selection does not by itself give the next eligible target that event. | Register real parts, clip, transform and shape each pass; do not enlarge a hit rectangle or create a second retained tree. Removal requires equivalent overlap, layer, focus and capture behavior. Keep metadata pass-local. |
| Structural refresh, removal and modal registration: **retain** | Open, replace and close a modal, then click the background in one batch. First natural-sized modal must route using its placed geometry; a closed Area must stop occluding wheel input. | [`actual_modal_open_replace_close_then_background`](../crates/egui-prism/tests/input_dispatch.rs), [`actual_modal_first_sizing_routes_a_natural_button_in_the_opening_batch`](../crates/egui-prism/tests/input_dispatch.rs), [`closing_area_revokes_wheel_occlusion_until_reregistered`](../crates/egui-prism/tests/input_scroll.rs). Public Area/Modal layout and discard are candidates for placement, not evidence of immediate revocation within the same event stream. | Host removes structure before the next event; local registration may repeat layout and must not run business actions. Keep measurement callbacks local. Exit requires current geometry and focus/capture revocation, including both layout orders and persistent modals. |
| Scroll frame-time versus event impulses: **retain**, physics implementation **narrow** | Wheel, click, reversed wheel, click must use each updated position; an extra layout pass cannot advance inertia twice. | [`trackpad_scroll_click_scroll_click_refreshes_local_geometry`](../crates/egui-prism/tests/input_scroll.rs), [`discard_remeasurement_does_not_release_wheel_carry_or_advance_physics`](../crates/egui-prism/tests/input_scroll.rs), ordinary-mode comparison tests in that file. Native wheel/physics math is reused; public `ScrollArea::State` can retain offsets but does not grant a second frame-time budget. | Host runs relevant local scroll shells and refreshes affected geometry. Preserve private destination consumption tokens on state restore. Exit needs nested axes, cancellation, isolated ordinary trajectories and mixed-batch causality, not only matching final offsets. |
| Explicit untargeted Time opportunity: **retain**, deadlines **narrow** | An idle Tooltip deadline needs handling without a raw event, once per viewport frame, shared with live scroll consumers; removed consumers must not remain scheduled. | [`empty_frame_time_is_opt_in_and_must_be_renewed`](../crates/egui-prism/tests/input_time.rs), [`custom_consumers_and_scroll_area_share_one_opportunity`](../crates/egui-prism/tests/input_time.rs). `request_repaint_after` schedules a frame but is not an ordered handling opportunity. | Consumer owns deadline/cancellation; host forwards Time. No background registry is needed. Exit requires empty-frame, repeated-registration and discard non-replay equivalence. |
| Label selection event finalization and source renewal: **retain**, text cache **narrow** | Copy before Escape must snapshot the earlier selection. Re-registering an ID as nonselectable must revoke its old text, while appending a Response sense must not revoke valid text. | [`stock_multilabel_copy_snapshot_precedes_escape_or_selection_change`](../crates/egui-prism/tests/input_native_defaults.rs), [`current_registration_revokes_or_renews_same_id_label_text`](../crates/egui-prism/tests/input_native_defaults.rs), Response/Ui preservation cases in that file. Public custom labels/clipboard output could replace the entire feature, but that is not demonstrated equivalence to stock multi-label selection. | Keep text references pass-local and prune closed viewports. Upstream plugin callbacks alone need evaluation at the same causal point; do not retain stale text or build a separate hit tree. |
| Stock TextEdit and default input migration: **retain** | First-focus editor navigation, middle-click paste at its event position and keyboard shortcuts must respect handler consumption and not repeat in paint. | [`ordinary_text_edit_in_event_phase`](../crates/egui-prism/tests/input_dispatch.rs), [`middle_click_paste_uses_event_position_and_stock_editor`](../crates/egui-prism/tests/input_native_defaults.rs), [`ordered_default_shortcuts_match_stock_frame_semantics`](../crates/egui-prism/tests/input_native_defaults.rs). Public TextEdit state and plugins are extension points, but a legacy widget running only once during painting is not an ordered handler. | Mixed stock/custom controls require explicit migration. Native IME/clipboard acceptance remains separate. Remove only when public hooks preserve policy registration, composition ownership and consumed-event semantics together. |
| Distribution metadata and licenses: **retain**, not behavioral extensions | Consumer must resolve one controlled egui type graph and reproduce the source with notices intact. | [`check_egui_source.py`](../scripts/check_egui_source.py), [`check_input_consumer.py`](../scripts/check_input_consumer.py), root patch and standalone consumer manifest. | Current source is non-published. Publishing a fork or registry route is a separate decision; licenses are not optional patch overhead. |

The scope above covers the behavioral families in the vendor inventory, including
Response/Ui geometry updates as part of selection/source preservation. Compositing
is a separate public-extension consumer, not another vendor renderer patch.

## Pinned public-API comparison

The official archive was checksum-verified before reading its unmodified source.
These are concrete source differences, not executed counterexample results:

- Official `Context::read_response` documents interaction selected at pass start
  from previous-pass rectangles; native hit selection reads `prev_pass.widgets`.
- Official `Memory::set_focus_lock_filter` only changes the filter when
  `had_focus_last_frame(id) && has_focus(id)`. `request_focus` is available, but
  requesting focus is not pre-registering every current control's policy.
- `Response::union` combines rectangles and flags and disallows calling
  `interact` on the union. It is not composite ownership registration.
  `Context::set_dragged_id` is public, so a wrapper-based capture alternative
  must be investigated rather than dismissed as impossible.
- Official `Modal::show` and `Memory::set_modal_layer` exist, but
  `top_modal_layer` is documented in terms of the previous frame. This alone
  does not establish current-batch open/replace/close equivalence.

The corresponding official sources are
[`context.rs`](https://github.com/emilk/egui/blob/7c0b40402271e8d40cda69f87cf3c2617c2e4382/crates/egui/src/context.rs),
[`memory/mod.rs`](https://github.com/emilk/egui/blob/7c0b40402271e8d40cda69f87cf3c2617c2e4382/crates/egui/src/memory/mod.rs)
and [`response.rs`](https://github.com/emilk/egui/blob/7c0b40402271e8d40cda69f87cf3c2617c2e4382/crates/egui/src/response.rs).
The archive checksum, rather than this companion Git reference, remains the
controlled-source identity.

The comparison with shadcn-egui is fixed at
[`2e9b530`](https://github.com/oetiker/shadcn-egui/tree/2e9b53058657fa26f70e27a8d0fa8f63ab1761fc).
Its [manifest](https://github.com/oetiker/shadcn-egui/blob/2e9b53058657fa26f70e27a8d0fa8f63ab1761fc/Cargo.toml)
uses official egui 0.37; its
[Button](https://github.com/oetiker/shadcn-egui/blob/2e9b53058657fa26f70e27a8d0fa8f63ab1761fc/crates/shadcn-egui/src/components/button.rs)
shows useful ordinary `Widget`/`Ui::interact` encapsulation. These are examples
of public styling/layout extension points, not a behavioral replacement proof.

Its [Slider](https://github.com/oetiker/shadcn-egui/blob/2e9b53058657fa26f70e27a8d0fa8f63ab1761fc/crates/shadcn-egui/src/components/slider.rs)
uses an enclosing rail/thumb union rectangle and aggregate key consumption.
That differs from Prism's part-geometry and original-event-order contracts.
Its [Select](https://github.com/oetiker/shadcn-egui/blob/2e9b53058657fa26f70e27a8d0fa8f63ab1761fc/crates/shadcn-egui/src/components/select.rs)
uses stock Popup and state-based key/text processing; matching visible controls
is not evidence of same-batch causality. Its
[Dialog](https://github.com/oetiker/shadcn-egui/blob/2e9b53058657fa26f70e27a8d0fa8f63ab1761fc/crates/shadcn-egui/src/components/dialog.rs)
uses stock Modal and explicitly excludes backdrop blur. Borrow the explicit
boundaries, not an undeclared weakening of Prism's pinned shadcn/ui contract.

## Three responsibilities, not one compatibility switch

### Component

A component owns its stable IDs, current part geometry, local policy, visuals and
interpretation of targeted responses. It does not own the application's event
loop, domain state, native capture, permissions, media sessions or queue submission.
Text entry and popup consumers still depend on the host completing the protocol.
An ordinary `Widget` spelling cannot supply missing application-phase ownership.

### Explicit integration scope

An ordered Context has one registration/event/paint authority per viewport.
Local component or scroll scopes can refresh themselves through `input_layout`,
but cannot silently replay arbitrary application callbacks. `Modal::register`
may measure local content twice; callers must separate layout from business work.
Plugins consuming input at begin-pass remain authoritative for that consumption;
legacy plugins that perform interaction there are not automatically ordered.
Different controls in a shared Context cannot independently select incompatible
input models and assume isolation. A separate Context is a separate integration,
not a free way to share focus, capture or compositor state.

### Application frame

The host selects sources/features, converts platform events, completes ordered
phases and Time delivery, refreshes changed structures, routes viewports, and
owns render resources and submission. See the complete
[input protocol](input-compatibility.md) rather than a second abbreviated API.
Compilation and execution are different obligations: the core's references to
`register_focus_policy`, owner/rounding registration and `InputTrigger` require
the controlled source even when ordered execution is not enabled. Setting
`ordered_input = false` is not support for unmodified egui, nor proof that an
existing application survives upgrading from an earlier egui release.

The standalone [consumer manifest](../tests/input-consumer/Cargo.toml) and
[type-graph checker](../scripts/check_input_consumer.py) cover one egui/epaint graph
with official integration crates. Cargo reads `[patch.crates-io]` from the
consumer root, not Prism's manifest. A successful workspace build or this
compile-only harness is not the runnable external recipe or a registry package.

## Rendering boundary and real integration costs

The [compositor guide](../crates/egui-prism-wgpu/README.md) is the canonical
contract. Core stays egui-only; the adapter uses unmodified egui-wgpu, not a
renderer fork. Group opacity differs observably from multiplying every shape's
alpha: an opaque glyph over an opaque fill must be assembled before fading.

Public egui paint callbacks and renderer texture registration provide useful
extension points. The implemented adapter uses them, so a new renderer patch is
not justified by this requirement. They do not make installation into any host
free: the host must retain untessellated shapes, upload current textures before
preparation, encode offscreen groups, render, submit returned callback buffers
before its main buffer, and release frame/texture resources in the documented
order. A producer writing a consumed image during final paint is too late.

The guide excludes stock eframe install-only integration because the required
after-upload preparation point is not exposed by that route. This is a limit of
the shipped adapter integration, not proof that no future public eframe API can
work. No eframe fork is proposed. Unsupported sRGB attachments, predictable
filtering, multi-sampling, cross-device resources and custom callbacks inside
groups do not become supported by successful compilation.

Maintenance cost follows boundaries: carrying an input patch requires source
reconstruction and upstream comparison; adopting ordered mode changes host and
plugin flow; group compositing requires explicit device/frame ownership. Runtime
cost also includes local re-layout, group targets, offscreen passes, texture
registration and cleanup. Patch line count measures none of these costs, and
this assessment provides no timing or memory benchmark. Pooling exact target
sizes is a lifecycle policy, not a demonstrated performance advantage.

## Support ownership and status

Use the [library-owned schema](releases.md#version-support-contract) for each
candidate, including matching core/adapter source, patch checksum, dependency
features, toolchains, input protocol, rendering constraints and platform evidence.
The manifest declares Rust 1.96 and the toolchain pins 1.96.0; neither is by itself
an executed minimum-version result. Never copy a Gallery platform build result
into a consumer's native GUI, IME, clipboard, accessibility or media acceptance.

The external recipe demonstrates a selected combination; source-preview freezes
its evidence; consumer-fit selects an adoption route. Applications own product
validation. None of these transfers ownership of Prism's support contract to an
application. Ordered mode, fonts, accessibility bridges and renderer settings
must be recorded as requirements or observations, not an undifferentiated
"supported" checkbox.

## Executed public-API experiment

The standalone [upstream comparison](../tests/upstream-boundaries/src/lib.rs)
uses official unpatched egui 0.37.0 in its own workspace. Its checker verifies
registry identity and the same archive checksum as the vendor baseline, so the
consumer root patch cannot silently turn this into another patched-only test.
It runs in all native cargo CI jobs.

After one warm-up pass registers two ordinary Buttons, the same ordered pointer
stream clicks the first then the second. With one `Context::run_ui` call,
observed per-button activation counts are `[0, 1]` and the UI callback runs once. Splitting at the click
boundary into two public calls at the same timestamp produces `[1, 1]`, but the
UI callback runs twice. This is an observed mismatch, not a compile-error proxy.

The experiment shows that splitting frames is not a transparent substitution
for an arbitrary existing callback: it changes callback count, while the ordinary
single batch does not expose both activations. It **does not** prove a carefully
phase-separated public adapter impossible. Modal, capture, scroll, plugin/output
and IME equivalence remain unresolved. No native window or GPU is involved.

Reproduce with `python3 scripts/check_upstream_boundaries.py` through the normal
resource gate. The inputs use fixed timestamps, not sleeps. Existing patched
`two_clicks_in_one_batch_reach_both_targets` remains the corresponding required
behavior, not a second claim about all public API designs.

## Remaining equivalence work and decisions

The [minimal public-API input equivalence Quest](../quest/m0/public-input-equivalence.md)
tracks the next Arrow/Tab/Arrow and modal experiments. It is planned work, not
additional executed evidence for this assessment.

No existing extension is approved for removal by this report. The next unresolved experiment is an unmodified-egui phase-aware prototype using
public event/pass hooks for the minimal Arrow/Tab/Arrow and modal open/close sequences, while
measuring action replay, plugin consumption, frame-time and output behavior.
If that fails, record the precise observed failure and source, not a blanket
claim that public APIs cannot work. If it passes, extend the comparison to
capture, structural refresh, scroll and composition before proposing replacement.

Any new or expanded extension requires a real consumer, a minimal failure case,
equivalent public-API alternatives, regression coverage, migration burden and a
specific upstream-equivalence exit condition. A different interaction contract
must be proposed as a scope change, not called an equivalent implementation.
This assessment delivers the matrix, a bounded public-API experiment and the
consumer responsibility record. Further phase-aware alternative prototypes remain
research options, not hidden requirements for merging this report. No removal or
production architecture change follows automatically from its recommendations.
