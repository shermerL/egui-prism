# Inline Alert

`alert::Alert` is an inline notice with CSS minimum sizing and visible overflow. The caller owns its theme, stable
identity, title, description, optional image and application policy. It retains
no state, timer, focus or dismissal lifecycle. `AlertVariant::Destructive` is a
visual choice, not a network or application error detector.

## Use and ownership

Install fonts with `theme::install_fonts` and install the host compositor.

```rust
use egui_prism::{alert::{Alert, AlertVariant}, compositing, egui, theme::Theme};

fn notice(ui: &mut egui::Ui, theme: &Theme) -> Result<egui::Response, compositing::Error> {
    Alert::new(theme, "save-error")
        .title("Unable to save")
        .description("Your changes are still available in this window.")
        .variant(AlertVariant::Destructive)
        .show(ui)
        .paint(ui.painter())
}
```

The identity salt is scoped to the parent UI. Keep it stable while text changes
and give simultaneous notices distinct salts. Text and theme are borrowed; the
image follows egui's `Image` ownership and loading contract. The host owns image
loaders and textures, including cancellation/repaint for asynchronous loading.
Icons are decorative; the complete meaning must be present in title/description.

`show` allocates and prepares an unfaded frame-local drawing. It does not submit
paint. Call `paint` with a same-context, same-layer painter, or include `drawing`
in a parent compositing group. No drawing is prepared when outside the clip.
Capture the painter before a disabled scope: inline notices do not have a
disabled fade. Parent opacity applies to the completed drawing once.

Compose caller actions using the existing `button::Button` API next to the
notice. The Alert does not start retries or own callbacks. The executable
`cargo run --locked -p egui-prism --example alert` illustrates separate alert
and action submission without a window or GPU claim.

## Pinned appearance and bounds

Reference: `registry/new-york-v4/ui/alert.tsx` at shadcn
`8a3536678a528cc012b9e62484290e79d0be410f`, unchanged in the reference fixture.

- Available width fills the finite parent allocation with a 34-point minimum.
  Padding is 16 horizontal
  and 12 vertical points, plus a one-point semantic border. Radius is 10 points,
  independent of control metrics.
- A decorative icon takes a 16-point slot, 12-point column gap and two-point
  downward translation. A white/default image tint inherits the title color.
- Title: Geist Medium, 14 points, 20-point line height, -0.35-point tracking,
  single-line ellipsis and a 16-point minimum for empty text.
- Description: plain-text Geist Regular, 14 points and 20-point line height.
  Its grid column preserves the widest legal unbreakable segment, including
  nonbreaking spaces, before greedy line wrapping. Collapsed line-end spaces
  do not consume width. Title/description rows have a two-point gap.
  Either can be omitted, with or without an icon. Empty descriptions have no line.
- Default: card surface and foreground, muted description. Destructive: the
  same surface, destructive title/icon and 90%-alpha destructive description.

ASCII whitespace collapses like CSS normal whitespace. Nonbreaking spaces are
preserved. This API accepts plain text, corresponding to direct text children
of `AlertDescription`, not rich HTML, paragraph children, links or nested lists.
The reference's paragraph-specific relaxed typography is outside this text API.
An internal `unicode-linebreak` dependency supplies Unicode15 opportunities.
Printable ASCII follows Chromium's compatibility rules, which differ from
untailored UAX14 for URLs and opening punctuation. This is not a complete CSS
text engine: dictionary-based scripts, newer Unicode rules, matched CJK fallback
fonts and browser-specific differences still need explicit evidence.

Provide finite available width, including within horizontal scroll areas.
The border box cannot be narrower than its 34-point padding/border footprint.
Description min-content may extend beyond it; drawing/group bounds include the
visible overflow and remain bounded by the parent UI clip. Set that parent clip
explicitly when a container must hide overflowing text. This replaces the earlier
Draft's self-clipping behavior without changing the builder or output types.
A title's ellipsis and a parent's clip never shorten accessible content.

## Accessibility

Every notice exposes `Role::Alert`, assertive live-region metadata and atomic
updates, with the complete title and description joined into the live-region
label. Description-only notices and body-only updates therefore retain
announcement content; the description is not duplicated as supplementary help.
This matches the pinned root's `role="alert"`; visual variant does not alter
announcement urgency. Use this component for alert semantics, not routine
polite status updates. It is never a click, drag or keyboard-focus target.

The host must publish AccessKit updates through its native accessibility bridge.
Metadata tests do not demonstrate announcement behavior in a screen reader;
there is no component-owned announcement queue or per-frame notification.

## Fixtures, checks and remaining evidence

The native Alerts page and browser `?alerts` fixture include the same text,
448/160/320-point layouts, icon/title/description combinations, clipping and
60% group opacity over a tinted surface. Use matching light/dark theme and
100%/150% scale. The icon is caller-supplied, not a built-in icon set.

Controlled tests cover geometry/colors/typography, optional content combinations,
empty text and whitespace, ellipsis and wrapping, complete stable-ID accessibility
metadata, noninteraction, narrow/fractional-scale clipping, justified layouts
and explicit compositor submission. The public example is headless.

No real native GUI, matching native/browser visual comparison, pixel-difference,
assistive-technology or cross-platform font-rasterization acceptance is claimed.
Those observations remain in the existing Inline Alert Quest. The native
accessibility bridge work is separate from this component's metadata.
