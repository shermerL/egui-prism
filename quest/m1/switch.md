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

The previous draft is being replaced with fixed default/small geometry, reference
focus-visible and keyboard timing, and controlled reference/Gallery comparisons.
Completion still requires actual browser and native comparison, independent
review, latest-head GitHub reviews, and CI. Keep this Quest until those gates pass.

## Required

- [Host-owned group compositing](/quest/m1/shadcn-parity/host-compositing.md) - shared opacity and transition rendering

## Waiting

- Input routing: resolve batched pointer events using each event position while
  preserving the host's widget hit order. Final-position response flags lose
  earlier clicks, while rectangle/layer-only routing permits same-layer
  click-through. Keep the draft open until a complete approach is reviewed.
- Host compositing foundation: integrate complete-control opacity after the
  shared host capability lands. Per-shape opacity does not satisfy CSS group
  compositing. Then rerun actual native/browser comparisons on light, dark,
  gray, and colored containers before finishing this Quest.
