# [S] Spinner

## Goal

A caller-controlled busy indicator matches the pinned shadcn geometry and motion without implying measurable progress.

## Plan

Follow [the fixed parity contract](/docs/shadcn-parity.md) and `registry/new-york-v4/ui/spinner.tsx` at shadcn commit `8a3536678a528cc012b9e62484290e79d0be410f`. Start from the existing transition timing and theme conventions; preserve the accepted egui-only core and host-owned input, frame and GPU responsibilities.

- Keep busy state and accessible status text at the caller. Own only frame-local drawing and any minimal stable timing required by animation.
- Schedule repaint only while the indicator is visible and active, respecting an available reduced-motion policy. Do not create a timer thread or interpret background work.
- Use controlled time to test motion and idle repaint behavior. Include the same indicator in an inline and button-adjacent Gallery example; do not fabricate percentage progress.

Complete implementation, the compiling public API example, component documentation, native Gallery case and meaningful regressions in one PR. Compare a mature candidate with the actual pinned reference at matching scale and theme. Report controlled tests, native observations and untested platforms separately. Review only the changed component and affected shared paths; reuse unchanged evidence. No host/vendor architecture change, new dependency identity or publication is included.
