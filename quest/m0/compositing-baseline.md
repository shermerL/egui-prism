# [M] Establish compositing cost and equivalence baselines

## Goal

A reproducible native workload identifies the compositor's actual preparation and rendering costs, while targeted pixel evidence determines which simplifications could preserve the existing Group contract.

## Plan

The [standalone settings recipe](/examples/settings/README.md) supplies fixed public-consumer source and basic-test evidence; its real-platform follow-up remains in native-gallery.

Start from crates/egui-prism-wgpu, the host integration guide, real Button drawing and existing framebuffer conformance checks. Preserve caller ownership, texture-update ordering, transforms, clipping, nested opacity and error handling. Reuse existing correctness evidence instead of repeating the entire component acceptance matrix.

Use bounded representative workloads covering ordinary opaque controls, faded and nested groups, fixed and changing target sizes, fractional scale/translation and continuous repaint. Record exact source, platform/backend, device and workload. Separate cold preparation, stable frames and resource changes; distinguish texture-pool reuse from texture registrations, buffer creation, uploads and offscreen passes. Measure CPU preparation and GPU duration where supported, with sample method and variability. Report unavailable measurements explicitly rather than inferring GPU speed from CPU submission time.

Check the suspected Button antialiasing-boundary issue with actual Button ordinary-versus-group framebuffer output, including fractional post-rounding translation and an appearance without a shadow expanding its bounds. Treat it as a hypothesis until reproduced. Resolve a confirmed local bounds defect with a regression in this task; a broader component or rendering-contract change requires a concrete design decision. Do not weaken tolerances to hide a discrepancy.

Assess opaque-group direct drawing, batching and resource/shadow caching as candidates, not predetermined implementations. Effective parent opacity, nested isolation, paint order, clipping, texture sampling and cache invalidation must remain correct. An opacity field equal to one is not sufficient evidence for an equivalent execution path. A cached shadow alone does not implement whole-control group opacity.

Acceptance includes runnable baseline checks, scoped regression coverage for any confirmed bounds fix, measured results and a justified recommendation of worthwhile changes or no change. Keep variable performance data out of brittle fast-CI thresholds. Record what the measured backend does and does not establish; software Vulkan is not physical-GPU or video-playback acceptance.

This task does not introduce a new renderer, select a new public host API, implement speculative optimization suites or establish an application performance claim. Choose later implementation from measured benefit and correctness evidence rather than adding every possible optimization.

## Related

- [Integration boundaries](/quest/m0/integration-boundaries.md) - consumer and extension necessity assessment
