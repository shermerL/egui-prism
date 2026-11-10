# [M] Field and Label composition

## Goal

Labels, help text, validation text and caller-owned controls form accessible, consistently aligned settings fields.

## Plan

Reference field.tsx and label.tsx; start at the accepted [TextInput contract](/docs/text-input.md) for label/error metadata and explicit drawing, the [Checkbox contract](/docs/checkboxes.md) for label activation and disabled behavior, and theme typography. A field owns its semantic association and layout, while the caller owns value, validation and submit policy. Support the reference horizontal/vertical composition needed by the Gallery setting example; long descriptions/errors must wrap without covering controls.

Do not copy a product-specific 56/84-point settings row, add a schema/validation engine, or make Label steal child input. Label focus/activation must target the intended live control and preserve disabled state. Cover multiple fields, invalid/help combinations, long localized text, focus and narrow content. Verify accessible relationships, not only visible text.

Complete the implementation, compiling caller example, component documentation, native Gallery case and meaningful regressions in the same PR. Use /docs/shadcn-parity.md and the fixed shadcn source at 8a3536678a528cc012b9e62484290e79d0be410f. Include supported states, stable identities, caller ownership, clipping and accessible semantics; do not claim unsupported variants. Compare a mature candidate at matching theme and scale, then revisit only affected behavior after fixes. Keep source, controlled tests, real native input/GUI and untested platforms distinct. Reuse unchanged review evidence; final changed code still needs applicable independent/GitHub review and CI. Core stays egui-only; the host owns event order, fonts, frame/GPU lifetime and application policy. A materially new public boundary requires a concrete prototype and maintainer decision. No generic widget framework, application state manager or publication is included.
