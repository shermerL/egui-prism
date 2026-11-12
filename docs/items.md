# Items

Item is a passive composition of caller-owned title, description, optional leading
icon and explicit Button/IconButton actions. The caller supplies stable identities
for the Item and each action and retains application state. The container does
not become a button, infer selection or own a collection or background task.

## Public use

```rust
use egui_prism::{button::{Button, ButtonRole}, item::{Item, ItemAction}, theme::Theme};
# fn draw(ui: &mut egui_prism::egui::Ui, theme: &Theme) {
let output = Item::new(theme, "project-a")
    .title("Project A")
    .description("Caller-owned project information.")
    .action(ItemAction::new("open", Button::new(theme, "Open").role(ButtonRole::Outline)))
    .action(ItemAction::new("archive", Button::new(theme, "Archive")).enabled(false))
    .show(ui);
// Inspect output.actions[i].response.clicked() for ordinary egui input.
// The caller, not layout, performs application side effects.
output.paint(ui.painter()).expect("host compositor installed");
# }
```

For the [ordered input host](input-compatibility.md), use `register` during
registration, retain each `ButtonOutput`, route only its targeted events to
`handle`, call `cancel_unfocused` on every retained action for every routed event
(including untargeted window-focus events), and regenerate the final Item drawing after input dispatch. Keep the
same Item/action IDs through reordering. Retire removed controls using the host's
existing input lifecycle, using `output.ids` (including labels and child UI IDs),
not just the action IDs. Item does not secretly own an input router.

The output's single drawing includes the action drawings. Do not paint the same
actions separately. The host installs the [compositor](compositing.md) and owns
fonts, images, frames and GPU resources. Capture the parent painter before a
caller-disabled scope; action disabling is available through `enabled(false)`.

## Supported scope and limits

The finite interface supports the default/outline/muted shell variants, default
and small sizes, plain title/description, an optional icon tile and independent
Button/IconButton actions. The action controls retain their own accessible names,
focus and activation semantics. The Item itself has no row activation or selection.

Surface fill/border changes use the pinned 100ms duration and existing CSS-ease
convention with caller-hosted time. Retargeting starts from the current color.
No timer thread is created; settled, clipped and zero-UI-opacity surfaces do not
request transition frames. IDs absent for a full frame restart at current colors.

This is not the complete upstream slot system: arbitrary widget callbacks,
interactive `asChild` rows, linked descriptions, image media, multiple content
slots, header/footer and list/separator wrappers are not exported. Do not infer
support for them from the reference fixture, which imports the complete original
source. The Rust text engine is not a CSS engine: exact CSS `text-balance`,
line-breaking/min-content rules and raster parity are not established. The
current minimum-width estimate uses space-delimited words, so CJK, punctuation
and browser-specific break opportunities are not implemented as CSS equivalents.
Description balancing minimizes egui wrap width while retaining its natural line
count (up to six), then visibly clamps to two lines. This bounded algorithm is
not a claim of browser `text-balance` equivalence. Long unbreakable content and
action groups retain their natural width and contribute to the host scroll
extent instead of moving drawings away from their registered hit rectangles. Any known layout limitations
remain implementation limitations, not “tests passed” or merely missing screenshots.

## Evidence layers

`cargo run --locked -p egui-prism --example item` is a compiling collector-host
example, not a rendered window. Component regressions exercise the public API;
the native Gallery Items case exposes independent action counters, hide/reinsert,
reorder, disabled actions, long copy, optional media and both sizes. Use the host's
narrow-width, theme and scale controls.

Compare the actual fixed reference at `/?items`. Under the maintainer's scope
decision, real native/browser visual and timing comparison and assistive-technology
announcements are deferred specialist work, not hidden implementation-stage merge
gates. That deferral does not close known core layout or input defects. Record
exact build IDs, OS/backend, browser and AT version, scale, screenshots or video,
keyboard/action observations and complete-name announcements. Do not call a
serialized accessibility tree a real screen-reader test.
