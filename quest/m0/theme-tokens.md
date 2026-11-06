# [M] Semantic light and dark theme tokens

## Goal

Library and gallery share one source for semantic foreground/background pairs,
spacing, sizing, typography, and interaction state styling in light and dark modes.

## Plan

M0 owns the design foundation. Propose the small theme API before exposing it;
callers retain font configuration and application branding. Select a limited
set of sizes from measured gallery needs, not an application's fixed dimensions.

Keep selected, disabled, destructive, and focus colors paired with their intended
foreground. Demonstrate theme changes without resetting caller-owned values.
Document where page, container, and widget spacing are owned to avoid double
padding. Verify both themes in the native harness with actual screenshots and
record contrast observations, including selected text. State any untested input
or platform behavior explicitly.

## Required

- [Repository foundation](/quest/m0/repository-foundation.md)
