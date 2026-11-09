# [M] TextInput shadcn parity

## Goal

TextInput matches the fixed input appearance and interaction while retaining caller-owned text and native editing behavior.

## Plan

Start at crates/egui-prism/src/text_input.rs, tests/text_input.rs under that crate, gallery/src/text_inputs.rs, gallery/src/input.rs and reference input.tsx.

Settled design: TextInput::show returns response/layout results plus drawing, then the caller explicitly paints using the correct pre-disabled painter. Remove the Widget entry point rather than silently discarding drawing. Keep TextEdit responsible for editing, selection, caret, undo, clipboard and composition. A private synchronous extraction may collect only shapes emitted by this one TextEdit invocation on its layer, preserving clips/transforms and preventing duplicate drawing. No cross-frame paint range, generic capture service or renderer patch is authorized. This boundary achieves one 50% disabled group composition while retaining parent opacity.

Keep labels, error text and placeholder semantics deliberate, with caller-owned validation. Match reference height, padding, radius, fills, border, selection, invalid and focus-visible states. Preserve stable IDs and current-pass input ownership, including multiple editors and same-batch focus changes.

Acceptance includes selection/caret/preedit retention, no neighboring-shape capture or double paint, disabled first-frame safety, parent opacity, narrow width and large text bounds, and a real ordered Tab/Text/Tab/Paste sequence. Reuse existing meaningful tests, add cases that distinguish the changed behavior, and verify actual native composition after editing changes. Record macOS/Windows/Linux evidence separately. Include migration examples and the mature native/reference comparison. PR #6 introduced the approximate component; preserve that provenance in the completion PR.

Complete the implementation, compiling caller example, component documentation, native Gallery case and meaningful regressions in the same PR. Use /docs/shadcn-parity.md and the fixed shadcn source at 8a3536678a528cc012b9e62484290e79d0be410f. Include supported states, stable identities, caller ownership, clipping and accessible semantics; do not claim unsupported variants. Compare a mature candidate at matching theme and scale, then revisit only affected behavior after fixes. Keep source, controlled tests, real native input/GUI and untested platforms distinct. Reuse unchanged review evidence; final changed code still needs applicable independent/GitHub review and CI. Core stays egui-only; the host owns event order, fonts, frame/GPU lifetime and application policy. A materially new public boundary requires a concrete prototype and maintainer decision. No generic widget framework, application state manager or publication is included.
