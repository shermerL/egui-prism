# [S] Badge

## Goal

A compact badge presents caller-provided text and supported variants with exact pinned typography, spacing and states.

## Plan

Follow [the fixed parity contract](/docs/shadcn-parity.md) and `registry/new-york-v4/ui/badge.tsx` at shadcn commit `8a3536678a528cc012b9e62484290e79d0be410f`. Start from crates/egui-prism/src/theme.rs and the current button drawing contract; preserve the accepted egui-only core and host-owned input, frame and GPU responsibilities.

- Follow the pinned reference variants and focus/link semantics only where exposed; do not invent application-specific connection or playback status enums.
- The caller owns the text and meaning. Expose only capabilities used by the minimal example; preserve long text layout and accessible names.
- Verify logical geometry, variant colors and light/dark rendering. Include a compact native Gallery row; do not add screenshot-only or implementation-mirroring tests.

Complete implementation, the compiling public API example, component documentation, native Gallery case and meaningful regressions in one PR. Compare a mature candidate with the actual pinned reference at matching scale and theme. Report controlled tests, native observations and untested platforms separately. Review only the changed component and affected shared paths; reuse unchanged evidence. No host/vendor architecture change, new dependency identity or publication is included.
