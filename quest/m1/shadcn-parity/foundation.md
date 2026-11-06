# [M] Pinned shadcn reference, tokens and typography

## Goal

A reproducible real shadcn fixture and matching Rust theme/font foundation make
component parity measurable instead of subjective.

## Plan

Use the source, tokens, font configuration and scale in /docs/shadcn-parity.md.
Pin the fixture's behavior primitives and dependencies; render actual source,
not a hand-written visual approximation. Retain all adapted source/font licenses.
Make the comparison cover neutral light/dark colors, alpha compositing, font
weights/metrics, focus-visible rings, shadows, and component-specific sizes.
Remove the earlier arbitrary blue focus/selection and generic-height assumptions.
Keep host ownership and explicit custom-theme behavior documented. Improve Gallery
presentation without presenting unfinished controls as parity-complete.

The earlier approximation entered in PR #3 (theme) and PR #4 (Gallery). Their
behavior/build evidence remains valid only for those earlier candidates. Report
any public API migration and actual visual comparison separately from tests.
