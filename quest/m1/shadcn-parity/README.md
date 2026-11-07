# [L] Strict shadcn parity for existing components

## Goal

Every shipped component matches the pinned shadcn appearance, geometry, states,
and interaction rather than presenting themed egui defaults.

## Plan

M1 owns core component correctness, including correction of already merged
components. The maintainer explicitly requires this for all components. Follow
[the parity contract](/docs/shadcn-parity.md). Complete each correction with real
reference comparison and preserve caller ownership, accessibility, and IME safety.
The existing Switch, Slider, Select and Dialog Quests own their component work;
this epic owns the shared foundation and corrections to completed components.

## Required

- [Input compatibility](/quest/m1/shadcn-parity/input-compatibility.md) - ordered input and authoritative focus, hit and capture behavior

- [Button and IconButton parity](/quest/m1/shadcn-parity/button.md)
- [TextInput parity](/quest/m1/shadcn-parity/text-input.md)
- [Checkbox parity](/quest/m1/shadcn-parity/checkbox.md)
