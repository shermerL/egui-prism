# [S] Card composition

## Goal

Caller-owned content can be composed in a Card with the pinned header, description, content and footer geometry.

## Plan

Follow [the fixed parity contract](/docs/shadcn-parity.md) and `registry/new-york-v4/ui/card.tsx` at shadcn commit `8a3536678a528cc012b9e62484290e79d0be410f`. Start from existing theme tokens and ordinary egui child-layout ownership; preserve the accepted egui-only core and host-owned input, frame and GPU responsibilities.

- Match pinned padding, border, radius, spacing and typography. Keep content layout and lifetime explicit at the caller; do not introduce a retained layout tree.
- Card carries no application selection, connection, navigation or session state. Keep arbitrary GPU callbacks outside any opacity grouping contract.
- Cover empty/short/long content and nested controls without double padding or clipped actions. Document dimensions as logical points and provide a compiling Gallery example.

Complete implementation, the compiling public API example, component documentation, native Gallery case and meaningful regressions in one PR. Compare a mature candidate with the actual pinned reference at matching scale and theme. Report controlled tests, native observations and untested platforms separately. Review only the changed component and affected shared paths; reuse unchanged evidence. No host/vendor architecture change, new dependency identity or publication is included.

## Waiting

- Implementation: reconcile header wrapping with pinned CSS, including long unbreakable words and non-Latin line-break opportunities. Keep the current egui wrapping limitation explicit until controlled reference comparison and regressions establish the intended behavior.
- Native validation: compare the Cards Gallery and actual pinned reference at matching width, font, scale and theme, including the two-layer shadow and nested controls. The current headless environment cannot establish real native or assistive-technology observations.
- Maintainer: authorize any GitHub automated code/security review separately. Keep the PR draft and do not merge without a later explicit decision.

## Related

- [Targeted acceptance](/quest/m2/native-components/card-acceptance.md) - trace missing tests and native evidence without waiving this component quest.
