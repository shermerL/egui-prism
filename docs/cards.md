# Cards

`card::{Card, CardUi, CardSection, CardOutput}` composes caller-owned content with
ordinary egui child UIs. Card holds no selection, navigation, application values,
connection state, retained layout tree, input dispatcher or resource lifecycle.
The host owns fonts, events, rendering and textures. The executable
[`card` example](../crates/egui-prism/examples/card.rs) uses only public APIs and a
recording compositor; run `cargo run --locked -p egui-prism --example card`.

## Layout contract

All dimensions are logical points, corresponding to CSS pixels at matching scale.
The source is the unchanged pinned
[`card.tsx`](../reference/src/ui/card.tsx), with theme and Tailwind versions in the
[reference manifest](../reference/manifest.json).

- Full available width, one-point border, 14-point `rounded-xl` radius. The pinned
  theme defines `--radius` as 10px and `--radius-xl` as 1.4 times that value.
- Root padding is 24 points vertically. Header, content and footer reserve 24
  points horizontally inside the border. They do not double-pad nested controls.
- A 24-point gap separates each explicitly added section. Empty sections still
  participate in that gap. An empty Card is 50 points high including its borders.
- Header has two automatic rows with an eight-point gap, including when the
  optional description is absent. Title is 16/16-point Geist SemiBold and
  description 14/20-point regular text in the muted foreground. Empty strings
  have no line height. Install the host font definitions with `theme::install_fonts`.
- Content uses ordinary top-down egui layout. `footer(height, closure)` uses a
  caller-declared row height, vertical centering and zero horizontal gap. Pass
  the tallest child height (36 for the default Button, or zero for an empty row).
  This makes every child's drawing and input bounds correct in one pass, without
  replaying arbitrary caller closures. Invalid or insufficient height is rejected.
  Callers add spacing explicitly; content retains its existing typography and spacing.
- Card uses the theme surface pair and border. Its two-layer `shadow-sm` is
  `0 1px 3px 0` and `0 1px 2px -1px`, both black at 10%, clipped outside the
  unshifted rounded border box. It is not the single-layer `shadow-xs` used by
  Buttons. The shared exclusion helper preserves the existing Button shadow.

Constrain width with a parent UI, such as `ui.set_max_width(320.0)`. Width must be
finite. Card never scrolls or truncates actions. Content can grow the height;
children wider than the Card may overflow under the parent's clip, matching the
reference's visible overflow. Put the whole Card in a caller-owned ScrollArea
when the viewport needs scrolling. The passive Card response is not focusable,
clickable or draggable, and does not replace any child response.

The API covers the Quest's default header, description, content and footer.
It does not expose the upstream optional CardAction grid slot, header/footer
border utility classes or arbitrary CSS class overrides. Compose application
actions in the content or footer. Header wrapping uses egui's text layout, so
browser Unicode line-breaking and very long unbreakable-word behavior are not
claimed to be identical.

## Drawing and input ownership

`Card::show` runs the caller's section construction once, returning its values and
an unfaded drawing. Call `paint` with a same-context, same-layer painter in that
pass. Capture the painter before entering a disabled scope if the passive shell
should stay unfaded. The compositor is required; a missing host returns an error.

Card reserves a paint slot before constructing children. `paint` fills that slot
so ordinary child painting remains above its shell. Header text and explicit
`CardSection::draw` groups join the shell in one opacity isolation boundary.
The inherited painter opacity is applied once. Child groups preserve their own
opacity and clipping. Keep their texture handles alive until rendering finishes.

Ordinary egui shapes, directly painted Prism controls and arbitrary GPU callbacks
remain separate from that isolated group. They are not captured, replayed or
made opacity-equivalent to CSS group opacity. Use explicit child groups when
whole-Card isolation is needed; arbitrary callbacks are unsupported in a Group.
A Card drawing may itself be placed in a parent Group instead of calling `paint`.

Retain child results to route ordered input through their existing APIs. Card
never performs a caller action. The Gallery uses its established registration,
routed handling and final-paint phases for the nested Save button and standard
egui text editor. The editor's native appearance is not a Prism TextInput parity
sample. Values and activation counts survive unrelated frames and page changes.

## Evidence and remaining acceptance

The public regression suite covers light/dark geometry at 100% and 150%, empty
sections, wrapping, content/footer padding, visible overflow, parent clipping,
passive response semantics, explicit group opacity, direct-paint ordering, shadow
exclusion, fixed widths after overflowing content, short/mixed-height footers,
accented glyph bounds and nested pointer/keyboard/disabled/editor behavior. It checks layout and
semantic drawing data, not browser/native pixel equivalence.

The native Cards page and `/?cards` reference contain short/long text, nested
input and actions, an empty Card and an isolated header over a tinted surface.
The reference uses the actual unmodified source. Production compilation and
source hashes are checks, not visual observations.

No matched native/browser screenshot comparison, real native event delivery,
OS DPI observation or screen-reader acceptance has been performed for this
component. The shadow uses egui's tessellation/blur, not a browser blur kernel;
matching visual appearance still needs observation. Header CSS line breaking,
including long words and non-Latin text, remains an explicit parity gap. Keep
[the Card Quest](../quest/m2/native-components/card.md) open until these applicable
acceptance conditions have been reconciled. Existing platform evidence for
other controls does not establish Card acceptance.
