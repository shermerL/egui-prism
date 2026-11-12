# [M] Item composition with independent actions

## Goal

A generic item lays out leading content, text and trailing actions without overlapping hit regions or losing long-text access.

## Plan

Reference item.tsx and accepted Button/IconButton/Tooltip contracts. The caller supplies content, stable identity and actions. Do not introduce device/source models, inferred selection rules or a retained collection. Passive layout must not turn the whole row into an action when a nested control already owns input.

Cover long text and accessible full names, optional leading content, trailing buttons, disabled actions and constrained width. Distinguish a passive Item from an explicitly interactive Item and from selection controls. Reserve action space using real content measurements. Use the fixed reference presentation; any needed interactive extension requires a concrete reference and declared public scope.

Complete the implementation, compiling caller example, component documentation, native Gallery case and meaningful regressions in the same PR. Use /docs/shadcn-parity.md and the fixed shadcn source at 8a3536678a528cc012b9e62484290e79d0be410f. Include supported states, stable identities, caller ownership, clipping and accessible semantics; do not claim unsupported variants. Compare a mature candidate at matching theme and scale, then revisit only affected behavior after fixes. Keep source, controlled tests, real native input/GUI and untested platforms distinct. Reuse unchanged review evidence; final changed code still needs applicable independent/GitHub review and CI. Core stays egui-only; the host owns event order, fonts, frame/GPU lifetime and application policy. A materially new public boundary requires a concrete prototype and maintainer decision. No generic widget framework, application state manager or publication is included.
