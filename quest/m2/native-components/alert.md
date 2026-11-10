# [S] Inline Alert

## Goal

A bounded inline notice presents a title and description with the pinned shadcn appearance and clear accessibility semantics.

## Plan

Follow [the fixed parity contract](/docs/shadcn-parity.md) and `registry/new-york-v4/ui/alert.tsx` at shadcn commit `8a3536678a528cc012b9e62484290e79d0be410f`. Start from theme tokens and existing text/control composition; preserve the accepted egui-only core and host-owned input, frame and GPU responsibilities.

- The caller selects semantic presentation and provides text. Rendering the notice must not infer network success, retry operations or dismiss application errors.
- Match the pinned layout, icon slot, variant colors, wrapping and description typography. Compose any caller action using existing Button rather than owning its policy.
- Cover long text, constrained width, light/dark and the supported icon/description combinations. Include an example and relevant layout/metadata checks; this is not a timed toast manager.

Complete implementation, the compiling public API example, component documentation, native Gallery case and meaningful regressions in one PR. Compare a mature candidate with the actual pinned reference at matching scale and theme. Report controlled tests, native observations and untested platforms separately. Review only the changed component and affected shared paths; reuse unchanged evidence. No host/vendor architecture change, new dependency identity or publication is included.
