# Spinners

`Spinner` is a passive 16-point status indicator, not measurable progress. The
caller owns busy state, a stable ID, accessible text and reduced-motion policy.
Omit `show` when idle; use `animate(false)` for a static reduced-motion indicator.
There is no worker, timer thread, platform preference lookup or percent value.

```rust
use egui_prism::{spinner::Spinner, theme::Theme};
# fn draw(ui: &mut egui::Ui, theme: &Theme, busy: bool, reduced_motion: bool) {
if busy {
    Spinner::new(theme, "loading-projects", "Loading projects")
        .animate(!reduced_motion)
        .show(ui)
        .paint(ui.painter())
        .expect("host compositor installed");
}
# }
```

Install the [host compositor](compositing.md) before painting. The output contains
an unfaded `Group` for composition; parent opacity is applied once by submission.
The host supplies finite frame time. Each ID has its own one-second clockwise
rotation epoch. Omission for a frame and re-enabling motion restart it; clipping
pauses repaint requests, not elapsed time. Repeated layout passes do not restart
motion. Repaint is requested only for visible animated drawing with nonzero UI
painter opacity. A parent `Group` assembled later is not visible to the widget;
if that parent is hidden or transparent, the caller must omit `show` or disable
animation rather than expect the widget to infer its final opacity.

The pinned Lucide LoaderCircle arc has a two-unit rounded stroke in a 24-unit
viewbox, scaled to 16 points. Geometry is one antialiased mesh, avoiding overlapping
stroke/cap alpha. Colors use the theme foreground. The accessible node is a
caller-named polite, atomic Status, without focus, activation or a numeric value.
Actual screen-reader delivery remains a host/platform acceptance check.

## Evidence and remaining acceptance

`cargo test --locked -p egui-prism --test spinner` checks controlled quarter-turn
positions, cycle duration, independent epochs, omitted frames, clipping, opacity,
reduced motion, repeated passes, theme/scale, stable status semantics and explicit
host submission. `cargo run --locked -p egui-prism --example spinner` is a compiling
collector-host example, not a rendered window.

Open the Gallery Spinners case and the actual pinned reference `/?spinners`.
The [Spinner Quest](../quest/m2/native-components/spinner.md) defines the remaining
real-browser/native/assistive-technology acceptance. Automated shape and tree
checks do not establish pixel parity, native frame pacing or announcement quality.
