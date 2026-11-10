# [M] Single-selection RadioGroup

## Goal

A controlled group selects exactly one available item with reference-matching pointer, keyboard and accessibility behavior.

## Plan

Follow [the fixed parity contract](/docs/shadcn-parity.md) and `registry/new-york-v4/ui/radio-group.tsx` at shadcn commit `8a3536678a528cc012b9e62484290e79d0be410f`. Start from the accepted [Checkbox contract](/docs/checkboxes.md) for selection geometry and ordered input, and existing stable-ID group patterns; preserve the accepted egui-only core and host-owned input, frame and GPU responsibilities.

- Keep the selected value and options at the caller. Use stable item identities and preserve caller selection when items reorder; define removed/disabled selection behavior explicitly.
- Match the pinned radio indicator, label hit behavior, focus-visible and disabled states; implement the observed orientation, arrow and Home/End behavior rather than inheriting checkbox activation.
- Reuse ordered events and current group compositing. Do not add a general form state/validation framework.
- Cover empty/one/multiple options, disabled items, reorder and focus return. Include one simple setting-group Gallery example and a compiling caller-owned-value API example.

Complete implementation, the compiling public API example, component documentation, native Gallery case and meaningful regressions in one PR. Compare a mature candidate with the actual pinned reference at matching scale and theme. Report controlled tests, native observations and untested platforms separately. Review only the changed component and affected shared paths; reuse unchanged evidence. No host/vendor architecture change, new dependency identity or publication is included.
