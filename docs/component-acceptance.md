# Component acceptance protocol

Written by dot (AI).

This is an acceptance specification, not evidence of completed validation.

## Evidence layers

Record each layer separately: unit/layout/metadata/controlled input; GPU
framebuffer/compositor; actual browser reference; native window/input/rendering;
native assistive technology; and cross-component/platform integration. Use
`PASS / FAIL / NOT RUN / BLOCKED / N/A`. Justify N/A against the contract;
unavailable equipment is not N/A. CI, screenshots and metadata do not substitute
for other evidence layers.

## Environment and matched samples

- Record head/base/tree, reference manifest/hashes, OS/version, GPU/backend,
  window system, browser/AT versions, font files/weights, logical and physical
  viewport/container sizes, DPR, OS scale and application zoom. Use unmodified
  shadcn source at `8a3536678a528cc012b9e62484290e79d0be410f` and Geist fonts.
  Missing localized glyphs are a failure or font blocker, not successful rendering.
- Build and run Gallery and reference on an accessible acceptance desktop. Verify
  the actual URL before comparing; hand-drawn HTML mocks are not the reference.
  Windowless software Vulkan does not establish Linux desktop acceptance.
- Cover light/dark at 100%/150% for key samples. At 100%, match one CSS pixel to
  one egui logical point. Record actual DPR/zoom at 150%; CSS transforms do not
  reproduce viewport media-query changes. Do not demand fictitious whole-image
  identity across different font rasterizers.
- Preserve native/reference originals, matched side-by-side/difference images,
  measurements and actions. Contract padding/gap/border/radius must match in
  logical points. Explain individual one-physical-pixel rasterization errors;
  tolerances cannot excuse wrong line counts, missing glyphs, truncation,
  misalignment or visibly incorrect shadows. Match supported reference line
  counts, word boundaries and complete content. Unresolved visual differences
  fail acceptance.
- Use `nix develop --command quest gate python3 scripts/check.py`. Platform-limited
  cargo/policy checks may be split and reported separately. Targeted commands are
  `cargo test --locked -p egui-prism --test COMPONENT` and
  `cargo run --locked -p egui-prism --example COMPONENT`. Component examples use a
  recording compositor; they are not GUI acceptance.
- Reproduce failures before fixing them and include automatable regressions in
  CI. Schedule slow pixel/platform work according to repository policy, not as
  unexecuted local-only scripts. A new dependency/host/vendor boundary needs its
  own concrete decision rather than silently expanding component acceptance.

## Landing and completion

Before component landing, resolve known supported-case CSS/behavior defects,
include new regressions in CI, compare the actual pinned browser with an
identified native platform, and perform applicable semantic/input observations.
Check the exact integrated head with full checks and affected-path review. Every
acceptance row needs evidence or an explicit blocker. Permission to land code is
not evidence that its Quest is complete.

Other OS/display-protocol observations and general host/IME/cross-component work
may remain in existing M1 tasks only with explicit ownership, revalidation
triggers and justification that they are not part of this component fix. Staged
landing needs an explicit scope decision with remaining obligations retained;
keep incomplete Quests and do not claim full parity/platform acceptance. Default
to draft when the applicable gates are unmet.

After authorized merging, smoke-test the final main SHA for installation,
Gallery navigation/exports, reference routes/hashes and evidence links, plus
affected tests. This confirms integration and does not replace pre-merge checks.
Acceptance plans grant no merge, auto-merge or queue authority; those actions
follow current task authorization and repository gates.

## Traceability and invalidation

Each evidence row records: requirement ID and original Quest clause, evidence
layer, sample/steps, expected/actual result, status, head/base, environment,
artifact links, executor/reviewer and blocking owner. Keep PR/Quest Waiting
items and evidence documents traceable. A follow-up task is not a PASS.

After implementation, fixture, host or base changes, identify affected behavior
and refresh its evidence. Reused evidence names its original head and why the
relevant paths remain valid. Old CI/review is not automatically exact-head
acceptance. Merge base changes locally rather than rebase/force-push; preserve
other writers' ownership according to repository workflow.

The component branches share exports, Gallery navigation/input, reference
routes/manifest and documentation indexes. Alert/Empty touch shared Button
helpers; Card/Empty touch Gallery input. On actual integration, preserve all
already-integrated components and Tooltip/Badge, source hashes and routes;
run affected regressions and repository checks. Require no missing exports,
duplicate registration/activation or new lint/test failures. A branch need not
provide components that have not been integrated into its candidate.

Record review gaps separately from acceptance results. Do not launch quota-using
reviews without applicable authorization. Establish different-model independence
for the normal review gate; unknown identity is BLOCKED, not inferred approval.
Any explicitly authorized review exception must be recorded for its exact scope.
