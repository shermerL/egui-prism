# Dialogs

`Dialog` is a controlled modal shell. The application owns its `open` boolean,
values, save/discard policy and trigger. Escape, a primary backdrop press (except Ctrl+click)
and the close icon report a neutral `DialogClose`; the application may accept
or decline that request. A body button does not implicitly close the dialog.

## Host and content contract

Enable the [ordered input capability](input-compatibility.md) at the application
root and install a [compositing backend](compositing.md). There is one host-owned
input cursor and one renderer lifecycle. Register dialogs after their background
controls, offer every routed event to the live dialogs before background handlers,
and refresh changed open state or body geometry before advancing the cursor.

Content closures execute immediately within the modal's layout scope. They may
run more than once to measure and place natural-height content. Register widgets,
return their event handles and contribute immutable drawing groups; perform
business actions only when handling the routed event. A closure is never stored.
The final paint closure rebuilds drawings from the current application values.

`DialogUi::ui` supplies scoped layout and interaction. `draw` includes a child's
`Group` in the dialog's opacity/scale animation. `focusable` enrolls each body control, including clipped or disabled controls,
so refresh can retire removed targets; initial focus selects an eligible control. An explicit initial focus override can select a
particular enrolled control. Composite controls enroll their actual focus targets.

Ordinary egui widgets that paint directly are **not** automatically captured into
this drawing group. Use controls with explicit drawing output, such as Prism
Button, or contribute text/image shapes through a `Group`. Referenced textures
remain owned by the application until the host renders them. The dialog does not
capture arbitrary painter calls or add a second renderer path.

Keep each dialog ID stable and distinct. Registrations are local to the current
viewport/pass and must not be stored in application state. Call `unregister`
before removing a registration during a live input batch. Theme and geometry
replacements must retire old registrations before registering replacements.
Continue registering with `open == false` while the exit animation finishes.
Accepting a close immediately disables body controls. The modal barrier remains
for the 200 ms content exit; eligible opener focus is restored when that presence
ends. Reopening during exit preserves the mounted instance and its original
opener. Explicit `unregister` immediately retires the old page's registrations
without moving focus back to controls the host may be removing.

The shell provides the title, optional description and close icon. The body is
naturally measured and bounded by the viewport; long content scrolls. Application
actions live in that body and are not a required set of footer buttons. This
bounded-body policy is an explicit extension to the pinned base Dialog fixture.

## Minimal host integration

The runnable [Dialog example](../crates/egui-prism/examples/dialog.rs) registers
one explicit-drawing Button, dispatches its action, accepts an Escape close
request, and refreshes before requesting another event:

```sh
nix develop --command quest gate cargo run --locked -p egui-prism --example dialog
```

Its collecting backend checks ownership and event flow without opening a window.
Native applications install their renderer-backed compositor instead. Keep action
handling outside the content callback: measurement can invoke that callback more
than once, while each routed event has only one business effect.

## Gallery and reference

The native Dialogs page and the pinned `/?dialogs` fixture provide independent
A/B instances, action and dismissal counters, long content, instance reversal,
a policy that declines dismissal, and a tinted background. Check body actions,
Escape, close icon, backdrop, Tab/Shift-Tab, opening again, and background action
counts in both themes. The browser fixture uses the unchanged pinned component
source and adds a bounded scrolling body in its calling example.

Native and browser viewports must match in logical size and scale. The desktop
shell has a 512-point maximum width; below the 640-point responsive breakpoint it
uses the viewport width minus 32 points. The Gallery's narrow *page content*
setting does not resize a modal's viewport.

## Evidence and limits

On 2026-11-10, the native Gallery was exercised on macOS with the Metal backend
against the pinned Chrome reference fixture. Observed main flows included
independent A/B instances, Apply without closing, Done, Escape, X, primary
backdrop dismissal, ignored context clicks, caller-declined dismissal, keyboard
focus and restoration, background action isolation, dark long-content scrolling,
and a narrowed native window with centered headings and vertical actions.

The desktop short panel measured 512 by 148 logical points; the 500-point browser
viewport produced a 468 by 192 panel. The native window at approximately 501
logical points showed the corresponding responsive layout. CSS source and
computed styles establish the 200 ms content and 150 ms overlay easing contracts;
settled screenshots are not frame-by-frame animation measurements.

Twenty public-API Dialog tests cover ordered dispatch, controlled close policy,
focus, viewport/scroll behavior, drawing ownership and host errors. Workspace
tests, strict clippy, documentation and the Gallery/reference builds passed.
Default workspace tests leave native-adapter tests explicitly ignored; this
observation does not claim those ignored suites ran.

Ordinary direct-painting egui controls remain outside group opacity isolation.
Without `theme::install_fonts`, the title uses the host proportional font instead
of claiming a semibold face. Windows/Linux desktop behavior, screen readers,
real IME interaction inside dialog bodies, and pixel-identical scrollbar or
animation rasterization remain unverified.
