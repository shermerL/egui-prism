# [M] TextInput shadcn parity

## Goal

TextInput matches its pinned shadcn source in appearance, geometry and interaction.

## Plan

Follow /docs/shadcn-parity.md and `registry/new-york-v4/ui/input.tsx`.
Match height, padding, border/background, placeholder, selection, invalid and focus-visible states, including dark alpha fills. Preserve text selection, undo, composition, disabled first-frame safety and caller-owned validation. Verify actual browser/native IME after relevant editing changes.

The previous approximate implementation entered in PR #6. Correct it in this
Quest, with public API migration notes where needed, reference/Rust state captures,
meaningful behavior/geometry regressions, independent and GitHub review, and CI.
Do not mark complete from passing compilation or the previous tests alone.

## Required

- [Shared reference, tokens and typography](/quest/m1/shadcn-parity/foundation.md)
