# [M] Controlled modal Dialog

## Goal

A caller-controlled dialog isolates input, reports neutral close intent and preserves focus and bounded natural content across open/close cycles.

## Plan

Start at the patched egui Modal and ScrollArea implementation, crates/egui-prism/src/compositing.rs, gallery/src/input.rs, gallery/src/selects.rs and reference dialog.tsx. The library has no completed Dialog implementation at this baseline.

Settled interface: use an immediately executed, non-retained scoped content closure and explicit child drawing Groups. The caller owns open state and business values. Registration returns frame-local inner child registrations; event handling returns close reason/layout intent. The caller applies decisions and refreshes geometry before the next event, then submits final drawing. DialogUi exposes content layout, Group contributions and eligible focus candidates. The callback is justified by natural content measurement inside the existing Modal lifecycle; an owned Ui populated after Modal returns cannot establish the parent size correctly.

Dialog owns title, description, close icon, modal behavior and a bounded scrolling body. Business actions belong to caller content; no mandatory save/cancel footer or duplicate close button. Ordinary egui widget output is not implicitly captured into group opacity/scale. Clearly document the supported content path and do not silently claim arbitrary widget/GPU callback animation. Reuse the existing Modal/ScrollArea/input/compositor, without a general overlay framework or new renderer lifecycle.

Match pinned width/viewport bounds, padding, typography, shadow, backdrop and motion. Cover short/long content, same-batch open/close, Escape/backdrop/explicit close reasons, background isolation, focus restoration to a still-live eligible opener, nested modal eligibility, disabled/disappearing triggers and multiple IDs. Business actions must not run during layout or final paint.

Complete the implementation, compiling caller example, component documentation, native Gallery case and meaningful regressions in the same PR. Use /docs/shadcn-parity.md and the fixed shadcn source at 8a3536678a528cc012b9e62484290e79d0be410f. Include supported states, stable identities, caller ownership, clipping and accessible semantics; do not claim unsupported variants. Compare a mature candidate at matching theme and scale, then revisit only affected behavior after fixes. Keep source, controlled tests, real native input/GUI and untested platforms distinct. Reuse unchanged review evidence; final changed code still needs applicable independent/GitHub review and CI. Core stays egui-only; the host owns event order, fonts, frame/GPU lifetime and application policy. A materially new public boundary requires a concrete prototype and maintainer decision. No generic widget framework, application state manager or publication is included.
