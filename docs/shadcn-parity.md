# shadcn parity contract

Every egui-prism component must follow the pinned shadcn reference in appearance,
geometry, interaction, and states. This includes components already merged and all
future components. egui supplies input events, text services, accessibility
integration, and drawing; its default widgets are not the specification.

## Reference

Use shadcn/ui commit `8a3536678a528cc012b9e62484290e79d0be410f`, with the `new-york-v4`
component implementations, the accompanying reference application's light/dark
tokens, and its Geist font configuration. Do not mix different shadcn styles or
silently follow a moving upstream branch.

- [Component source](https://github.com/shadcn-ui/ui/blob/8a3536678a528cc012b9e62484290e79d0be410f/apps/v4/registry/new-york-v4/ui)
- [Theme and radius tokens](https://github.com/shadcn-ui/ui/blob/8a3536678a528cc012b9e62484290e79d0be410f/apps/v4/app/globals.css)
- [Font configuration](https://github.com/shadcn-ui/ui/blob/8a3536678a528cc012b9e62484290e79d0be410f/apps/v4/lib/fonts.ts)
- [MIT license](https://github.com/shadcn-ui/ui/blob/8a3536678a528cc012b9e62484290e79d0be410f/LICENSE.md)

The reference uses a 16 CSS-pixel rem. Map CSS pixels to egui logical points at
100% zoom; compare at the same physical scale. Component geometry comes from its
source, not a multiplier of egui's default control height. Resolve Tailwind sizes,
colors, alpha, radii, line heights, weights, shadows, rings, and transitions from
the pinned source. Use the same licensed font and weights in comparison captures.

Pin the behavior primitive used by the reference fixture as well. Match pointer,
keyboard, focus-visible, disabled, invalid, popup, dismissal, and value semantics
from the actual reference. Reusing an egui operation is acceptable only when the
observed behavior satisfies that contract. A platform limitation is an open gap,
not permission to substitute egui behavior or claim parity.

## Shared compositing boundary

CSS opacity applies to a completed group. Multiplying each overlapping egui
shape's alpha produces different text, image, border and shadow colors. Strict
parity therefore requires host-owned group compositing over the actual surface;
a fixed background-color approximation is insufficient.

The [host compositing capability](/docs/compositing.md)
keeps the core egui-only and adds a separately installed wgpu adapter. The native
Gallery directly integrates official winit, egui-winit and egui-wgpu; the host
owns windows, GPU resources, current-frame texture preparation and ordered command
submission. Backend setup failures are explicit. Component implementation and
acceptance stay with their own Quests. The shared capability has native Metal
framebuffer regressions and software Vulkan GPU contract coverage; these do not
establish component parity or Windows/Linux desktop acceptance. Full native
platform input, IME, clipboard and OS DPI observations remain in the
[native Gallery Quest](/quest/m1/native-gallery.md).

## Shared input boundary

The opt-in [input compatibility capability](/docs/input-compatibility.md) uses a
controlled egui 0.37.0 patch for current-pass focus policy, ordered events and
authoritative hit/capture ownership inside egui. Consumers select one egui source
from their application root Cargo configuration and migrate to host-owned
registration, event handling and painting. Source reconstruction, boundary tests
and an independent consumer verify the shared implementation. Component Quests
still own adoption, strict interaction/visual parity and actual platform input/IME
acceptance; the patch does not establish OS-level pointer capture.

## Platform scope

Native macOS, Windows and Linux are the primary targets. The Gallery uses official
platform input bridges; a real pinned shadcn browser fixture supplies the visual
and interaction reference. Shipping the Rust library in browsers is a separate,
deferred product decision, and Pages does not block native M1 completion.

## Acceptance for every component

Capture the reference and Rust implementation at matching size, scale, font,
light/dark theme, and state. Compare resting, hover, pressed, keyboard focus,
disabled, selected/checked, invalid where supported, and long-content layouts.
Check visible bounds and hit targets together. Verify keyboard behavior against
the reference, including Home/End and modified steps where applicable.

Record source identifiers, automated checks, reference-browser observations,
actual native platform/backend observations, IME evidence and remaining gaps
separately. A macOS result cannot establish Windows or Linux acceptance.
Tests, CI, and code reviews do not establish visual parity. A component remains
unfinished while required visual or behavioral gaps
remain. Keep the real rendered reference available for comparison; do not replace
it with an independently approximated HTML mock.

## Audit of the previous candidates

These are known gaps, not an exhaustive acceptance result.

| Area | Pinned reference | Previous implementation | Work |
| --- | --- | --- | --- |
| Shared theme and type | Reference neutral tokens, Geist, component-specific dimensions, translucent neutral focus rings | Different background/colors, blue focus/selection, Ubuntu Light, generic 32-point height | Foundation parity Quest |
| Button and IconButton | Default height 36; xs 24, sm 32, lg 40; exact padding, role colors, shadows, 3-point focus ring | Height scaling from 32, approximate role/hover styling, missing variants | Button parity Quest |
| Input | Height 36, input border, transparent/light and alpha/dark fills, padding, primary selection, invalid/focus rings | Themed default TextEdit appearance and geometry | Input parity Quest |
| Checkbox | 16-point square, 4-point radius, 14-point indicator, exact checked/invalid/disabled states | Native egui checkbox painting and activation semantics | Checkbox parity Quest |
| Switch | Default 32 by 18.4 track and 16-point thumb; small 24 by 14 and 12-point thumb; exact translation/states | 36 by 20 track, approximate colors and outline | Existing Switch Quest/PR |
| Slider | 6-point rounded rail, 16-point round white thumb, primary fill, hover/focus ring and reference keyboard/value mapping | Rectangular egui handle, blue fill, egui geometry and keyboard mapping | Existing Slider Quest/PR |
| Select and Dialog | Reference popup/modal geometry, states, focus and dismissal | No completed implementation | Existing Quests |
| Gallery | Consistent reference typography and component presentation | Default egui harness controls dominate the visible page | Foundation and Gallery Quests |

The original behavior tests remain useful regression evidence, but they must not
preserve egui-specific behavior when it conflicts with the reference. Keep caller
ownership, instance identity, disabled safety, and IME correctness while changing
the component contract. Document any public API migration.

Retain copyright and license notices for adapted code, icons, and fonts. Links do
not replace the required notices. Releases and deployment remain separate from
component implementation.
