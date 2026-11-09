# [S] Tooltip

## Goal

Pointer and keyboard users can read a bounded hint for an explicitly identified trigger without changing focus or activation.

## Plan

Follow [the fixed parity contract](/docs/shadcn-parity.md) and `registry/new-york-v4/ui/tooltip.tsx` at shadcn commit `8a3536678a528cc012b9e62484290e79d0be410f`. Start from the current select popup placement and host ordered input registration; preserve the accepted egui-only core and host-owned input, frame and GPU responsibilities.

- Keep trigger identity, text and enabled state explicit; own only necessary transient visibility/timing state with cancellation when the trigger disappears.
- Match the pinned pointer/focus opening behavior, delay, dismissal, placement and viewport collision rules. A hint must not activate its trigger or claim modal input.
- Do not introduce a global overlay service, application shortcut, background task or new host lifecycle. Use existing layer and frame capabilities.
- Use controlled time for open/close tests. Cover focus, pointer transit, disabled triggers, viewport edges and two independent triggers; add a Gallery example with long plain text.

Complete implementation, the compiling public API example, component documentation, native Gallery case and meaningful regressions in one PR. Compare a mature candidate with the actual pinned reference at matching scale and theme. Report controlled tests, native observations and untested platforms separately. Review only the changed component and affected shared paths; reuse unchanged evidence. No host/vendor architecture change, new dependency identity or publication is included.
