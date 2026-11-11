# [XS] Separator

## Goal

A horizontal or vertical separator follows pinned geometry and distinguishes decorative from semantic use.

## Plan

The component, public example, focused contract tests, documentation, native Gallery page and pinned reference fixture are implemented. The public contract is in [Separators](/docs/separators.md). Preserve the egui-only core and the existing host-owned input, frame and GPU responsibilities.

Finish the targeted rendered comparison with shadcn commit `8a3536678a528cc012b9e62484290e79d0be410f` at matching theme and scale. Check both directions, decorative and semantic uses, clipping and parent opacity. Keep automated checks, actual native/browser observations and untested platforms separate; reuse unaffected component evidence.

## Waiting

- Project coordinator: review the exact implementation head and reconcile independent and GitHub code/security review results. Native/reference visual comparison still needs an environment where both fixtures are reachable; this implementation environment has no native display and its remote browser cannot reach the local reference server.
- Maintainer: PR #32 is an evaluation draft with an explicit merge prohibition. Keep it draft and do not merge or enable auto-merge without a later explicit decision; review success does not lift the prohibition.
