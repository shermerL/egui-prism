# [M] Editable text input

## Goal

A labeled text input supports real text editing, error descriptions, disabled state, and focus.

## Plan

M1 owns core components. Build on egui TextEdit, retaining selection, paste, undo, and IME behavior. Do not paint a string as a substitute for editing. Verify long content, independent IDs, value preservation, keyboard focus and disabled editing. Test actual IME on an available native platform and browser; explicitly list any remaining platform gaps.

Include the public API, compiling minimal Rust example, light/dark gallery cases,
behavior regressions, actual visual evidence, and explicit platform limitations
in the same PR. Propose material API choices before implementing them.
