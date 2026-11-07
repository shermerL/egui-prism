# [S] Controlled switch

## Goal

A switch reports an explicit caller-owned boolean change with clear on/off, focus, and disabled feedback.

## Plan

M1 owns core components. Verify pointer and keyboard activation, no disabled event, no repeated-frame event, and instance isolation. The widget reports intent only: it does not start networking, run asynchronous work, or infer successful application operations.

Include the public API, compiling minimal Rust example, light/dark gallery cases,
behavior regressions, actual visual evidence, and explicit platform limitations
in the same PR. Propose material API choices before implementing them.

Follow [the strict shadcn contract](/docs/shadcn-parity.md), including actual
reference comparison for every supported state. Earlier themed-egui candidates
do not satisfy this requirement.

## Required

- [Input compatibility](/quest/m1/shadcn-parity/input-compatibility.md) - ordered input and authoritative focus, hit and capture behavior
