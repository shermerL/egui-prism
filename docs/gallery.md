# Acceptance gallery

The native macOS, Windows and Linux hosts use the same Rust gallery and library. The gallery
demonstrates the theme API, Prism buttons, text inputs, checkboxes, switches, sliders,
selects, dialogs, empty states, and static badges, alongside standard egui harness controls. Other Prism widgets are planned.
A harness check is not a component acceptance result.

## Run and identify a build

```sh
nix develop --command cargo run --locked -p egui-prism-gallery
nix develop --command quest gate python3 scripts/check.py
```

The native host uses Metal on macOS, DX12 on Windows and Vulkan on Linux.
Initialization and device failures are reported by the native process. The pinned
CSS reference runs separately; browser product deployment is deferred.

The footer reports the development package version, renderer, and debug/release
build kind. Local builds say `local development`; CI sets `EGUI_PRISM_BUILD_ID`
to its full workflow commit SHA, embedding that ID in native artifacts. An artifact
built for a PR may identify GitHub's test merge commit rather than the branch tip.
No release or Pages deployment is performed by these commands or CI.

## Repeatable cases

The controls above the scrolling content remain available while navigating cases.
Theme and case changes preserve caller text, checkboxes, selected values, and action counts.

| Case | What to inspect |
|---|---|
| Theme | Light/dark foreground/background pairs and selected foreground |
| Empty states | Caller-selected empty/loading/error, long localized copy, disabled action, viewport padding, narrow overflow and opacity |
| Badges | Six static variants, long text scrolling, light/dark, 100%/150% scale, and parent opacity |
| Buttons | Prism roles, sizes, icon names, disabled actions, wrapping, and independent counts |
| Inputs | Caller-owned text, editing, errors, disabled state, stable IDs, and IME diagnostics |
| Selects | Independent controlled values, popup shielding, keyboard/typeahead, list variants, and idle counts |
| Tooltips | Pointer/focus hints, controlled delay, long plain text, disabled triggers and viewport edges |
| Dialogs | Controlled A/B modals, neutral dismissal, body actions, focus isolation/restoration, and bounded long content |
| Interaction | Independent A/B values, pointer activation, Tab/Enter, disabled actions, and idle counts |
| Layout | The same instances and IDs with long wrapping action labels |

Use `320pt content` to constrain the content without resizing the host, and the
100%/150% scale controls to exercise zoom. The host window can also be
resized. The native default viewport is 960 by 900 logical points. A fixed content
width is not a simulation of another device's DPI, font rasterization, or input.

For a screenshot and manual interaction pass:

1. Record the platform, displayed build, window/viewport size, scale, theme, and case.
2. In Interaction, enter distinct text into A and B, change their checkboxes, and
   activate A once. Its count should increase once; B should remain unchanged.
3. Leave the controls idle, then use Tab until an action has visible focus and
   press/release Enter. The focused action should activate once.
4. Enable `Disable actions` and try pointer/keyboard activation. Counts must stay
   unchanged. Text and checkbox editing remain enabled deliberately.
5. Reverse the instances and switch themes/cases. Values stay with their A/B
   identities. Both instances appear only once in each frame.
6. In Layout, enable narrow content and 150% scale. Long action labels should wrap;
   scroll to the second instance and footer. Return to 100% and full width.
7. Capture the actual window with the platform screenshot tool in each
   theme, including focused/selected text and a constrained layout. Keep screenshot
   observations separate from synthetic-input test output, and redact unrelated UI
   before sharing captures.
8. Test native copy/paste, IME composition and cancellation, focus loss, resize,
   minimize/restore and close on each OS. Record Linux X11/Wayland separately.

### Select comparison

The Selects page mirrors the cases in `reference/src/select-cases.tsx`, using
short bounded labels and wrapping settings to fit the native viewport. Compare
at 100% first; do not add CSS zoom to the browser reference.

- A starts at None and B at Cherry. Both use stable semantic IDs and independent
  change counters. The third trigger is permanently disabled and reflects A.
  Reverse the instances and verify values stay with A/B, not their positions.
- Triggers are 192 points wide, 36 points high by default and 32 with `Small`.
  Check light/dark, `Invalid`, `Disabled`, and the blue `Tinted` surface. Repeat
  in narrow content and at native 150% scale, scrolling where necessary.
- The short list has Apple, Apricot, disabled Banana, Blueberry, and Cherry.
  `Long list` has 30 options with 7 and 14 disabled. `Reset values` sets A to
  None and B to Cherry or Option 30 without resetting or incrementing counters.
  Switching lists does not silently replace a caller-owned value absent from
  the new list. `Empty` and `All disabled` must remain safely dismissible.
- Open by primary press or a focused trigger's Enter, Space, or vertical arrow.
  Navigate with arrows and Home/End, type to search, and commit with Enter.
  Navigation alone must not increment a counter. Selecting the existing value
  must not increment it either. Escape dismisses and restores trigger focus.
- Try a trigger-to-row drag and release, pointer departure, and window blur.
  Reopen after dismissal. Check that disabled rows cannot commit and that
  keyboard/typeahead skip them. Check closed-trigger typeahead separately.
- With a popup open, click `Outside action`. That click dismisses without
  incrementing the outside counter; a subsequent fresh click increments it
  once. Wheels over the popup or its backdrop must not scroll the page behind
  it, including at a popup scroll endpoint.
- In the long list, hover the scroll indicators and let them repeat without
  moving the pointer. Leave the indicator, dismiss, change pages, or blur the
  window. Scrolling must stop, and after the animation tail the viewport must
  return to idle rather than repaint continuously.

See [Select integration](selects.md) for the caller-owned value, frame-local
borrow, all-event forwarding, and unregister-before-replacement contracts.

## Automated coverage

The private interaction module is called by both the app and the tests. Synthetic
frames use egui's real input events and deterministic simulated time. They cover
pointer press/release, Tab traversal and Enter activation, disabled actions,
non-repeating idle frames, stable IDs after instance reordering, retained values
across theme changes, and long labels within a restricted width. Responses and
rectangles remain frame-local; the gallery only stores user values and settings.
Explicit UI scopes keep automatic child IDs stable when instances move.

Run these through the standard check command. They establish behavior of the
current egui harness and provide a pattern for future public Prism widget tests.
They do not establish native input delivery, IME composition, screen-reader
integration, touch, or another operating system's behavior. Actual GUI checks and
the tested OS/backend are reported in each PR. Unobserved Windows/Linux GUI,
CJK font coverage, IME, assistive technology, and a full DPI matrix remain open.

The Checkboxes case tests box/label clicks, Space-release activation, Enter no-op,
keyboard focus, disabled and invalid states, independent values after reordering,
long-label layout, and disabled compositing over three backgrounds. Change counters must
advance once per activation and stay unchanged while idle or disabled.
The Switches case checks track/label activation, keyboard focus, explicit value
changes, disabled state, independent instances, and wrapped-label layout.

The Sliders case checks two ranges and step sizes, pointer/keyboard edits,
disabled values, reordering, and wrapped labels above constrained rails.
The Compositing case independently checks group opacity. Run the automatic
final-framebuffer matrix with `--compositing-check`; see
[host compositing](compositing.md) for resource and ordering contracts.

The Select tests in `gallery/src/selects.rs` are marked
`#[ignore = "requires a native wgpu adapter"]` and execute production
`Gallery::ui` with an actual native adapter and wgpu Host. They use deterministic
time, raw events, and AccessKit requests rather than a parallel input host.
They inspect drawing callbacks without submitting them to a rendered framebuffer.
They cover pointer clicks and split press/release with intervening idle frames,
same-batch page navigation/open/navigation/commit, independent values
and one count per real change, reordering, configured/hidden trigger retirement,
outside-click shielding, reopening/focus, same-batch theme/list variants,
empty/all-disabled lists, and no repeated actions during registration, final
paint, or idle frames. They also inspect viewport `repaint_delay` after dismissal
and long-list indicator-hover cancellation, including blur.

The existing native-adapter CI command includes these tests:

```sh
cargo test --locked -p egui-prism-gallery -- --ignored --nocapture
```

The standard check script also runs the core `select` example's assertions.
These automated checks are not native GUI or Windows/Linux acceptance and do
not establish screenshot parity or real assistive-technology behavior.

### Tooltip comparison

The Tooltips page attaches hints to ordinary harness triggers with explicit IDs.
Compare keyboard focus, Escape, pointer transit, disabled triggers, long text,
instance removal and activation counts against `reference/?tooltips`. The hints
do not own trigger input or introduce a modal barrier. See [Tooltips](tooltips.md)
for natural text geometry, viewport drawing bounds and observation limits.
