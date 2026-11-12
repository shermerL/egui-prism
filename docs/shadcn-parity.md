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
submission. Backend setup failures are explicit. Component contracts and evidence remain in the
[current evidence index](#current-evidence-index). The shared capability has native Metal
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
and an independent consumer verify the shared implementation. Each component documents its adoption and
strict interaction/visual parity evidence separately. Actual platform input/IME
acceptance remains in the native Gallery work; the patch does not establish
OS-level pointer capture.

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

## Current evidence index

All eight core controls are delivered at component level. This index connects
their contracts and existing evidence; shared documentation completion does not
establish acceptance on every target platform. Results and observations retain
the revision, platform and scope recorded in their sources. A documented check
or fixture is not evidence that it ran for a later revision.

The component documents define caller ownership, stable identity, frame-local
registrations and outputs, ordered host integration, and explicit drawing and
resource lifetimes. Their migration sections replace the earlier egui-style APIs.

| Controls | Contract and evidence | Scope and limits |
| --- | --- | --- |
| Button and IconButton | [Buttons](buttons.md), [compositing and ordered-input evidence](buttons.md#compositing-and-ordered-input-evidence), [delivery PR](https://github.com/shermerL/egui-prism/pull/13) | Caller-owned actions and images, fixed roles/sizes and explicit drawing. The PR records macOS/Metal observations separately from automated checks. |
| TextInput | [Text input](text-input.md), [evidence and limits](text-input.md#evidence-and-limits) | Caller-owned text and validation, stable editor identity and explicit drawing. Recorded macOS/Metal and browser comparisons cover editing, geometry and states; they are not measured pixel differences. |
| Checkbox | [Checkbox](checkboxes.md), [evidence and limits](checkboxes.md#evidence-and-limits) | Caller-owned boolean, ordered activation and explicit drawing. Revision-specific checks and macOS/Metal and Chrome observations include the rich-label correction and 150% comparison. |
| Switch | [Switch](switches.md), [reference and validation](switches.md#reference-and-validation), [delivery PR](https://github.com/shermerL/egui-prism/pull/8) | Caller-owned boolean, default/small geometry, ordered activation and transitions. The PR records browser and macOS/Metal observations and framebuffer checks separately. |
| Slider | [Slider](sliders.md), [comparison and evidence](sliders.md#comparison-and-evidence), [delivery PR](https://github.com/shermerL/egui-prism/pull/9) | Caller-owned value, ordered input and explicit drawing. Single-thumb horizontal LTR scope includes a documented 16-point Rust layout minimum. Browser, macOS/Metal and framebuffer evidence are separate. |
| Select | [Selects](selects.md), [evidence and limits](selects.md#evidence-and-limits) | Caller-owned optional value, frame-local registration and ordered popup input. Adapter submission tests do not render framebuffer pixels. The browser geometry reference is at 100%; native 150% and narrow layouts are additional checks. |
| Dialog | [Dialogs](dialogs.md), [evidence and limits](dialogs.md#evidence-and-limits) | Caller-owned open state and dismissal policy, scoped content and explicit drawing groups. The bounded scrolling body is an explicit extension; ordinary direct-painting egui controls are outside group opacity isolation. |
| Empty | [Empty states](empty-states.md) | Passive copy and icon with caller-owned content/actions, viewport padding and explicit drawing. CSS balancing/line-breaking and matched visual/native acceptance remain open. |
| Badge | [Badges](badges.md) | Static caller-owned text with six variants, natural width, accessible text and explicit drawing. Platform observations are recorded separately from contract tests. |

### Shared capabilities

- [Semantic themes](themes.md#evidence-and-limits) and the
  [pinned reference fixture](../reference/README.md) define tokens, typography
  and comparison sources. Hosts own font installation and theme choice.
  Token/font checks do not establish component parity or accessibility certification.
- [Input compatibility](input-compatibility.md#verification-and-maintenance) and
  the [source inventory](../vendor/README.md) document the opt-in application-root
  Cargo patch, unified dependency source and host-owned input phases. Boundary
  and consumer checks do not establish OS capture or native input/IME acceptance.
- [Group compositing](compositing.md) defines renderer initialization,
  texture/resource lifetime, submission and shutdown ownership. Metal framebuffer
  regressions and software Vulkan contract checks do not establish every
  component's pixel parity or Windows/Linux desktop acceptance.
- [Gallery cases](gallery.md) and the [native Gallery acceptance work](../quest/m1/native-gallery.md)
  retain cross-component integration and the platform evidence matrix. macOS,
  Windows and Linux need separate build, controlled GPU and real desktop
  evidence, with X11/Wayland identified for Linux.

### Remaining acceptance limits

Windows/Linux GUI and assistive-technology acceptance remain unverified in the
component evidence above. Tests, CI, reviews, accessibility metadata and synthetic
actions do not substitute for visual comparison, native event delivery or
screen-reader observation.

TextInput's recorded macOS IME preedit and commit do not establish Chinese glyph
coverage or candidate-panel placement: the Gallery displayed missing glyph boxes.
Dialog's real IME interaction inside bodies remains unverified. Settled screenshots
do not establish frame-by-frame animation behavior or pixel-identical rasterization.

The native Gallery work retains cross-component focus/capture, modal/popover
layering, clipboard and real IME, multilingual glyphs, resize/scale and OS DPI,
focus loss/reentry, host lifecycle, compositor error propagation and accessibility
actions. Existing component observations do not close missing integration or
platform evidence.

## Historical baseline before parity corrections

The [original audit](https://github.com/shermerL/egui-prism/pull/11) was a
non-exhaustive review of pre-correction candidates, not an acceptance result.
It recorded different palette tokens, blue focus/selection, Ubuntu Light and a
generic 32-point control-height baseline. Button sizing scaled from that baseline,
TextInput relied on themed default TextEdit appearance, Switch used a 36 by 20
track, and Slider used egui handle geometry and keyboard mapping. Select was
absent from that early baseline, and standard egui harness controls dominated
the Gallery presentation.

These findings explain the foundation and component corrections; they are not a
current defect list. Use the current evidence index for delivered contracts,
recorded observations and remaining acceptance limits.

The original behavior tests remain useful regression evidence, but they must not
preserve egui-specific behavior when it conflicts with the reference. Keep caller
ownership, instance identity, disabled safety, and IME correctness while changing
the component contract. Document any public API migration.

Retain copyright and license notices for adapted code, icons, and fonts. Links do
not replace the required notices. Releases and deployment remain separate from
component implementation.
