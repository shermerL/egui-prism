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

## Required

- [Host-owned group compositing](/quest/m1/shadcn-parity/host-compositing.md) - shared opacity and transition rendering
