# [M] Button and IconButton shadcn parity

## Goal

Button and IconButton matches its pinned shadcn source in appearance, geometry and interaction.

## Plan

Follow /docs/shadcn-parity.md and `registry/new-york-v4/ui/button.tsx`.
Match all exposed variants and exact size/padding/gap/icon metrics, typography, hover/pressed/disabled states, shadows and focus-visible rings. Add the reference variants needed for a complete supported surface. Preserve explicit icon colors and accessible names; match button keyboard activation timing.

The previous approximate implementation entered in PR #5. Correct it in this
Quest, with public API migration notes where needed, reference/Rust state captures,
meaningful behavior/geometry regressions, independent and GitHub review, and CI.
Do not mark complete from passing compilation or the previous tests alone.

The role/size implementation and input regressions are present, but the Quest is
not complete. Button and IconButton now return their response and an unfaded Group
through `show`, and the caller submits it with a Painter captured before disabled
scopes. This replaces per-primitive fading with shared host compositing, keeping
the core renderer-independent. Real Button GPU regressions cover colored parent
backgrounds, translucent image/text overlap, clipping and parent opacity.
Repeat native/reference acceptance, independent review, latest-head GitHub reviews
and CI before completing this Quest; automated pixels alone are not full parity
acceptance.

Also close the focus-acquisition gap: egui cannot install a focus filter on the
pass that acquires focus, so immediate Escape/arrow input can navigate away.
Handle this at the input/framework boundary without reclaiming focus that the
caller deliberately released or canceling a pending Tab traversal. Preserve the
existing hit-target, input-lifetime and inherited-text-color regression fixes.

## Required

- [Input compatibility](/quest/m1/shadcn-parity/input-compatibility.md) - ordered input and authoritative focus, hit and capture behavior
