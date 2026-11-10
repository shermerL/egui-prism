# [M] Controlled Popover

## Goal

A caller-controlled nonmodal panel anchors to a trigger, remains inside the viewport and reports dismissal with predictable focus behavior.

## Plan

Follow [the fixed parity contract](/docs/shadcn-parity.md) and `registry/new-york-v4/ui/popover.tsx` at shadcn commit `8a3536678a528cc012b9e62484290e79d0be410f`. Start from the accepted Select contract, the [Dialog contract](/docs/dialogs.md) for scoped content and modal focus boundaries where applicable, and gallery/src/input.rs; preserve the accepted egui-only core and host-owned input, frame and GPU responsibilities.

- The application owns open state and content values. Opening, outside dismissal, Escape and focus restoration must have explicit results; navigation does not own application sessions.
- Reuse current layers, ordered events and group compositing. Match pinned placement, offsets, border, shadow, animation and supported content constraints.
- Do not build a generalized overlay framework or add hidden window/renderer lifetime. If arbitrary child painting requires a new public drawing boundary, present a concrete prototype before implementation.
- Cover same-batch open/close events, outside presses without click-through, viewport edges, disabled/disappearing triggers and multiple instances. Include an example using a few accepted form controls.

Complete implementation, the compiling public API example, component documentation, native Gallery case and meaningful regressions in one PR. Compare a mature candidate with the actual pinned reference at matching scale and theme. Report controlled tests, native observations and untested platforms separately. Review only the changed component and affected shared paths; reuse unchanged evidence. No host/vendor architecture change, new dependency identity or publication is included.
