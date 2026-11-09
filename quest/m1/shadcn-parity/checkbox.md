# [M] Checkbox shadcn parity

## Goal

A caller-owned boolean checkbox has the fixed geometry, label activation, keyboard behavior and whole-control disabled opacity.

## Plan

Start at crates/egui-prism/src/checkbox.rs, crates/egui-prism/tests/checkbox.rs, gallery/src/checkboxes.rs and reference checkbox.tsx. Follow the accepted Switch register/handle/show/paint pattern and existing compositor. Support bool states; adding an indeterminate value type is outside this task.

Match the 16-point control, 4-point radius, 14-point indicator, focus-visible/invalid states and pinned keyboard activation. Clickable labels must not duplicate activation. Maintain stable identities under reorder and the caller's value ownership. Share only the existing proven trigger-shadow clipping responsibility with Select; preserve Select output.

The whole drawing bound must include actual RichText glyph bounds, not only the default row height. Include a large explicit font case that would expose clipping, long/wrapped labels, parent opacity and multiple backgrounds. Test same-batch key/pointer ordering and disabled transitions. Mature native/reference evidence and affected shared-shadow checks are required; repeated full unrelated component reviews are not. PR #7 introduced the approximate component.

Complete the implementation, compiling caller example, component documentation, native Gallery case and meaningful regressions in the same PR. Use /docs/shadcn-parity.md and the fixed shadcn source at 8a3536678a528cc012b9e62484290e79d0be410f. Include supported states, stable identities, caller ownership, clipping and accessible semantics; do not claim unsupported variants. Compare a mature candidate at matching theme and scale, then revisit only affected behavior after fixes. Keep source, controlled tests, real native input/GUI and untested platforms distinct. Reuse unchanged review evidence; final changed code still needs applicable independent/GitHub review and CI. Core stays egui-only; the host owns event order, fonts, frame/GPU lifetime and application policy. A materially new public boundary requires a concrete prototype and maintainer decision. No generic widget framework, application state manager or publication is included.
