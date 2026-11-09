# [L] Strict shadcn parity for existing components

## Goal

Every shipped component matches the pinned shadcn appearance, geometry, states,
and interaction rather than presenting themed egui defaults.

## Plan

M1 owns core component correctness, including correction of already merged
components. The maintainer explicitly requires this for all components. Follow
[the parity contract](/docs/shadcn-parity.md). Complete each correction with real
reference comparison and preserve caller ownership, accessibility, and IME safety.
Switch, Slider and Select were delivered by their component Quests; the
remaining Dialog Quest owns its component work;
this epic owns the shared foundation and corrections to completed components.
