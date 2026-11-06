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
not complete. Real native and browser comparison found that disabled content
fades each overlapping primitive separately instead of compositing the button
once at 50% opacity. Integrate the shared host-compositing capability when it is
available; keep the core renderer-independent and do not approximate the backdrop
with a theme color. Validate final pixels on arbitrary parent backgrounds,
translucent images and overlapping shadows, then repeat native/browser acceptance,
independent review, latest-head GitHub reviews and CI before completing this Quest.


Also close the focus-acquisition gap: egui cannot install a focus filter on the
pass that acquires focus, so immediate Escape/arrow input can navigate away.
Handle this at the input/framework boundary without reclaiming focus that the
caller deliberately released or canceling a pending Tab traversal. Preserve the
existing hit-target, input-lifetime and inherited-text-color regression fixes.
