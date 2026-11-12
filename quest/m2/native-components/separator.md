# [XS] Separator

## Goal

A horizontal or vertical separator follows pinned geometry and distinguishes decorative from semantic use.

## Plan

Follow [the fixed parity contract](/docs/shadcn-parity.md) and `registry/new-york-v4/ui/separator.tsx` at shadcn commit `8a3536678a528cc012b9e62484290e79d0be410f`. Start from theme border tokens and existing layout primitives; preserve the accepted egui-only core and host-owned input, frame and GPU responsibilities.

- Expose orientation and semantics only as supported by the reference. Derive available extent from the caller layout; do not add global spacing policy.
- Preserve clipping, scale and accessibility semantics. Include a small compiling example with both orientations and focused geometry/metadata checks.

Complete implementation, the compiling public API example, component documentation, native Gallery case and meaningful regressions in one PR. Compare a mature candidate with the actual pinned reference at matching scale and theme. Report controlled tests, native observations and untested platforms separately. Review only the changed component and affected shared paths; reuse unchanged evidence. No host/vendor architecture change, new dependency identity or publication is included.

## Related

- [Targeted acceptance](/quest/m2/native-components/separator-acceptance.md) - trace missing tests and native evidence without waiving this component quest.
