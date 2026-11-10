# [M] Explicit confirmation AlertDialog

## Goal

A confirmation dialog exposes explicit neutral/cancel/confirm results and safe focus behavior using the accepted modal implementation.

## Plan

Reference alert-dialog.tsx and the completed Dialog API. Match the reference ambient-dismissal and initial-focus behavior; a generic Dialog is not automatically a confirmation contract. Caller owns whether an action is destructive, whether confirmation is enabled and what it does. Never stop resources merely because the presentation closes.

Reuse Dialog content and group drawing rather than create another modal engine. Cover Escape/backdrop behavior, default focus on a safe action, long content, disabled confirmation, reopen and disappearing opener. Include a generic confirmation example without application-specific commands.

Complete the implementation, compiling caller example, component documentation, native Gallery case and meaningful regressions in the same PR. Use /docs/shadcn-parity.md and the fixed shadcn source at 8a3536678a528cc012b9e62484290e79d0be410f. Include supported states, stable identities, caller ownership, clipping and accessible semantics; do not claim unsupported variants. Compare a mature candidate at matching theme and scale, then revisit only affected behavior after fixes. Keep source, controlled tests, real native input/GUI and untested platforms distinct. Reuse unchanged review evidence; final changed code still needs applicable independent/GitHub review and CI. Core stays egui-only; the host owns event order, fonts, frame/GPU lifetime and application policy. A materially new public boundary requires a concrete prototype and maintainer decision. No generic widget framework, application state manager or publication is included.
