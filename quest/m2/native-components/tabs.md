# [M] Controlled Tabs

## Goal

A controlled tab list selects caller-owned content with the pinned active styling and keyboard focus behavior.

## Plan

Follow [the fixed parity contract](/docs/shadcn-parity.md) and `registry/new-york-v4/ui/tabs.tsx` at shadcn commit `8a3536678a528cc012b9e62484290e79d0be410f`. Start from stable-ID group handling, accepted Button behavior and the Gallery layout; preserve the accepted egui-only core and host-owned input, frame and GPU responsibilities.

- The caller owns selected ID, contents and whether inactive content remains alive. Switching a tab reports intent; it must not start or stop application sessions.
- Match the pinned orientation, activation mode, roving focus, disabled state and indicator geometry. Document content visibility and focus retention on removal/reorder.
- Use existing host input and drawing capabilities. Do not introduce routing, docking, retained page state or application keyboard shortcuts.
- Cover multiple independent lists, disabled tabs, reorder/removal and focus entering/leaving content. Include a Gallery example showing caller-owned input values surviving navigation.

Complete implementation, the compiling public API example, component documentation, native Gallery case and meaningful regressions in one PR. Compare a mature candidate with the actual pinned reference at matching scale and theme. Report controlled tests, native observations and untested platforms separately. Review only the changed component and affected shared paths; reuse unchanged evidence. No host/vendor architecture change, new dependency identity or publication is included.
