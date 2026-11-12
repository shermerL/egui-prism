# [S] Empty-state composition

## Goal

A generic empty state presents a title, explanation and optional caller action without fabricating loading or error semantics.

## Plan

Reference empty.tsx; start at theme typography and accepted Button content. The caller supplies the meaning, icon and action. Keep empty, loading and failed states distinct. Support short/long localized copy and bounded content without forcing a one-line message to fill the screen. An example demonstrates caller-controlled switching among empty/loading/inline error, without network or media objects.

Complete the implementation, compiling caller example, component documentation, native Gallery case and meaningful regressions in the same PR. Use /docs/shadcn-parity.md and the fixed shadcn source at 8a3536678a528cc012b9e62484290e79d0be410f. Include supported states, stable identities, caller ownership, clipping and accessible semantics; do not claim unsupported variants. Compare a mature candidate at matching theme and scale, then revisit only affected behavior after fixes. Keep source, controlled tests, real native input/GUI and untested platforms distinct. Reuse unchanged review evidence; final changed code still needs applicable independent/GitHub review and CI. Core stays egui-only; the host owns event order, fonts, frame/GPU lifetime and application policy. A materially new public boundary requires a concrete prototype and maintainer decision. No generic widget framework, application state manager or publication is included.

## Related

- [Targeted acceptance](/quest/m2/native-components/empty-acceptance.md) - trace missing tests and native evidence without waiving this component quest.
