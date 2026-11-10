# [M] Bounded ScrollArea

## Goal

A bounded scroll viewport follows the pinned shadcn scrollbar geometry and interaction while its caller keeps surrounding actions fixed.

## Plan

Follow [the fixed parity contract](/docs/shadcn-parity.md) and `registry/new-york-v4/ui/scroll-area.tsx` at shadcn commit `8a3536678a528cc012b9e62484290e79d0be410f`. Start from gallery/src/input.rs, gallery/src/interaction.rs, and the existing select scroll path; preserve the accepted egui-only core and host-owned input, frame and GPU responsibilities.

- Keep viewport size, stable identity and content at the caller. Reuse the current ordered scroll and authoritative input routing instead of adding another wheel dispatcher.
- Match track, thumb, hover/drag state and clipping from the pinned reference. Preserve wheel, touchpad, keyboard reachability and focus scrolling; define behavior for empty, short and overflowing content.
- Do not add virtualization, recycled rows, a retained list model or automatic scrolling of sibling controls. Provide the reference default vertical viewport and explicit horizontal scrollbar composition, including a corner when both axes are used.
- Check nested scroll ownership, scroll limits, drag cancellation, last-item reachability and fixed sibling actions. Include a compiling example and a native Gallery case with controlled long content.

Complete implementation, the compiling public API example, component documentation, native Gallery case and meaningful regressions in one PR. Compare a mature candidate with the actual pinned reference at matching scale and theme. Report controlled tests, native observations and untested platforms separately. Review only the changed component and affected shared paths; reuse unchanged evidence. No host/vendor architecture change, new dependency identity or publication is included.
