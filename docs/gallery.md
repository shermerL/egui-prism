# Acceptance gallery

The native macOS, Windows and Linux hosts use the same Rust gallery and library. The gallery
demonstrates the theme API, Prism buttons and text inputs, and standard egui harness controls.
Other Prism widgets are planned. A harness check is not a component acceptance result.

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
Theme and case changes preserve caller text, checkboxes, and action counts.

| Case | What to inspect |
|---|---|
| Theme | Light/dark foreground/background pairs and selected foreground |
| Buttons | Prism roles, sizes, icon names, disabled actions, wrapping, and independent counts |
| Inputs | Caller-owned text, editing, errors, disabled state, stable IDs, and IME diagnostics |
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

The Checkboxes case tests box/label clicks, keyboard toggles, disabled editing,
independent values after reordering, and long-label layout. Change counters must
advance once per activation and stay unchanged while idle or disabled.
The Switches case checks track/label activation, keyboard focus, explicit value
changes, disabled state, independent instances, and wrapped-label layout.

The Compositing case independently checks group opacity. Run the automatic
final-framebuffer matrix with `--compositing-check`; see
[host compositing](compositing.md) for resource and ordering contracts.
