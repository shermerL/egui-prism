# [M] Checkbox shadcn parity

## Goal

Checkbox matches its pinned shadcn source in appearance, geometry and interaction.

## Plan

Follow /docs/shadcn-parity.md and `registry/new-york-v4/ui/checkbox.tsx`.
Replace native checkbox painting with the reference square, indicator, radii, shadows, selected/invalid/disabled states and focus-visible ring. Match reference keyboard activation and label behavior, including indeterminate behavior if exposed; do not retain egui defaults as the contract.

The previous approximate implementation entered in PR #7. Correct it in this
Quest, with public API migration notes where needed, reference/Rust state captures,
meaningful behavior/geometry regressions, independent and GitHub review, and CI.
Do not mark complete from passing compilation or the previous tests alone.

## Required

- [Input compatibility](/quest/m1/shadcn-parity/input-compatibility.md) - ordered input and authoritative focus, hit and capture behavior
