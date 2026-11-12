# Empty-state composition

`empty::Empty` composes plain-text copy, an optional decorative icon and caller
content. It stores no application state and does not infer empty, loading or
failed states. The caller decides when to show it and owns all actions.

## Use and ownership

Install fonts with `theme::install_fonts` and install the host compositor. Supply
a stable identity salt per instance, retaining it when copy or sibling order
changes. The salt scopes text and content identities under the parent UI.

```rust
use egui_prism::{empty::Empty, compositing, egui, theme::Theme};

fn notice(ui: &mut egui::Ui, theme: &Theme) -> Result<(), compositing::Error> {
    let painter = ui.painter().clone();
    Empty::new(theme, "projects-empty")
        .title("No projects yet")
        .description("Create a project to get started.")
        .show(ui, |_| {})
        .paint(&painter)?;
    Ok(())
}
```

`show` calls the content closure once using ordinary egui layout. `EmptyUi::ui`
borrows that centered UI. Return a Button registration or other caller result
through `EmptyOutput::inner`, then route events using the existing host contract.
`EmptyUi::draw` includes an explicit unfaded child group. No callbacks, focus
policy or action dispatcher are owned by Empty. The executable
`cargo run --locked -p egui-prism --example empty` demonstrates caller-selected
empty/loading/failed presentations, action handling, retirement of disappeared
controls and final painting. It has a recording compositor, not a window or GPU.

Text and theme are borrowed. Images follow egui's `Image` contract; the host owns
loaders, repaint, texture retention through rendering and resource cleanup.
The icon is decorative, so describe its meaning in the copy. Only the icon-tile
media variant is implemented; arbitrary plain media, rich linked descriptions,
CSS class overrides and a filled-height flex parent are not public variants.

## Pinned appearance and bounds

Reference: unmodified `new-york-v4/ui/empty.tsx` at shadcn
`8a3536678a528cc012b9e62484290e79d0be410f` in `reference/src/ui/empty.tsx`.

- Root: finite available width, natural height, no default border or background.
  The source's `border-dashed` does not add border width. Padding is 24 points
  below a **viewport** width of 768 logical points and 48 points at or above it.
  A 320-point container in a 960-point viewport still has 48-point padding.
  Native zoom changes viewport logical width; compare with actual browser zoom,
  not CSS `zoom`, which does not change the media query viewport in the same way.
- Header and content: centered, at most 384 points wide. Header children have an
  eight-point gap, plus the media's eight-point bottom margin. Present empty
  strings have zero text height but keep their element gaps. Unused content is
  omitted; populated content follows the header after 24 points and has a
  16-point vertical item gap. An empty root is padding only, not viewport-height.
- Title: host Button family (installed Geist Medium), 18-point size, 28-point
  line height, -0.45-point tracking, page foreground. Description: host Body
  family (installed Geist Regular), 14-point size, 22.75-point line height,
  muted foreground. Component geometry is independent of theme control metrics.
- Icon: 40-point muted tile with 10-point radius, centered 24-point image.
  An uncustomized white tint inherits page foreground.
- Copy collapses CSS ASCII whitespace, retains the full accessible text, never
  elides, and preserves space-delimited unbreakable runs including NBSP. A
  bounded width search balances wrapping while preserving the initial line
  count. Header text can overflow a very narrow column; allocation remains
  bounded and the parent's clip is authoritative. Glyph bounds preserve accents.

The balance algorithm is deterministic but is **not verified CSS `text-balance`
parity**. Browser line breaking around hyphens, punctuation, CJK and other scripts,
UA balancing limits and font shaping may differ. Longer localized copy remains
available without truncation, but actual wrapping, font coverage and script
behavior need matched browser/native observation. These remain in the existing
Quest, not an accepted substitute for its parity requirement.

## Drawing, input and semantics

`show` prepares a frame-local `drawing`, which is absent without visible shapes.
`paint` requires the same context and layer and inserts the completed group
before ordinary direct child painting. Alternatively include `drawing` in a
parent group. Group opacity applies once, including explicitly submitted child
groups. Ordinary egui painting and GPU callbacks remain outside Empty's group.
Capture the painter before disabled scopes to avoid an unwanted passive-copy
fade; child controls own their disabled appearance and activation contract.

The shell is hover-only. Title and description expose full passive Label values
with stable IDs, not heading, alert, live-region, busy or status semantics. They
never take keyboard focus or activation. A caller requiring loading announcements
or error announcements supplies them separately. Metadata and synthetic input
checks do not establish screen-reader speech or native accessibility transport.

## Evidence and remaining acceptance

The public tests cover viewport versus container size at 100%/150%, typography,
light/dark colors, icon and section gaps, empty/absent content, balanced and long
copy, whitespace/NBSP, clipping and accents, passive accessible values and stable
IDs, caller Button pointer/keyboard/disabled actions, explicit group opacity,
paint ordering and missing-host errors. The public example exercises caller state
switching without network/media dependencies. Check results belong to the tested
candidate recorded in the PR; the presence of a test is not evidence it ran.

The Gallery's **Empty states** page and reference `/?empty` have matching short
and localized copy, action disabling, separate loading/error presentations,
140-point overflow and tinted opacity samples. Inspect viewport widths on both
sides of 768, 320-point content, light/dark and actual 100%/150% zoom. Check icon,
text line boxes, long-copy line breaks and action hit targets together.

No matched native/browser visual comparison, native pointer/keyboard run,
screen-reader observation or macOS/Windows/Linux desktop acceptance is recorded
for this component. CI and headless tests do not close these gaps. Keep the Quest
and draft PR open until its remaining acceptance and reviews are resolved.
