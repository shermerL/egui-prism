# [S] Separator targeted test and native acceptance coverage

## Goal

Close the specifically missing automated, pinned-reference, native interaction and accessibility evidence for [PR #32](https://github.com/shermerL/egui-prism/pull/32), with reproducible fixtures and revision-bound evidence. This does not waive the original component Quest, authorize merging, or claim that a test plan is a passing result.

## Plan

Use the [fixed parity contract](/docs/shadcn-parity.md). The implementation and its known correctness gaps stay owned by /quest/m2/native-components/separator.md; coordinate fixes there rather than duplicate production changes in the acceptance PR. Add tests and evidence on the actual candidate, retaining a before-fix failing case for each defect. A separately reviewable test/evidence PR may wait for the fix without claiming acceptance.

The [shared acceptance protocol](/docs/component-acceptance.md) defines source identity, environment, oracle, evidence rows, integration invalidation and exit gates. Every result must name the tested head and distinguish tests, rendered browser/native observations, and AT. Use the unmodified pinned reference and matched font/theme/scale. Do not replace a required observation with a successful CI job.

1. Preserve the seven regressions. Compare native `Separators` and reference `?separators`: both orientations, decorative/semantic, ordinary/justified parents, normal/narrow/zero extents, partial clipping, tinted surfaces at 60% parent opacity, and 100%/150% scale. Lines are 1pt with caller-provided extent, unchanged hover/accessibility bounds, no unintended AA clipping and no extra margin.
2. Retain the 96-case automated framebuffer regression and mutation evidence documented in [Separator evidence](/docs/separators.md). This covers compositor pixels beyond geometry assertions using an independent egui polygon oracle; it does not establish pinned-browser/native parity. Complete matched fractional-scale thin-line and clip-edge observations against the pinned browser. Decorative lines are not interactive Tab stops; semantic lines expose correct role/orientation without activate/drag/focus actions. Observe native AT exposure/reading; platform phrasing may vary, but decorative lines must not become controls.
3. Matched visual/semantic evidence and integration review remain requirements for full acceptance of `79b860a` or a newer candidate. No new implementation defect is established; do not invent a fix requirement. Existing controlled macOS tests are not GUI evidence.

Implementation-stage scope: the Separator implementation may land before the specialist browser/native visual and actual AT observations. The validation executor owns those retained evidence rows, with the Gallery maintainer owning desktop/AT availability through the related Quests. This Quest remains open and the unobserved rows remain NOT RUN, not PASS. The stage boundary does not waive any discovered implementation defect or declare full component/Gallery acceptance. The landing review records its exact candidate and applicable review qualification separately.

Before completion, include the automatable supported-case assertions in CI, provide native/reference captures and measured comparisons, record AT/input observations, link the applicable exact-head CI and independent review, and map every original requirement to PASS or a specifically retained blocking owner. FAIL, NOT RUN and BLOCKED rows prevent this Quest being called complete. No silent scope reduction or after-merge deferral of known parity defects. Only an explicit maintainer scope decision can permit staged landing; record the deferred requirements and owners without marking them PASS or deleting incomplete Quests.

Refresh evidence after affected implementation, reference, host or integration changes. Preserve original implementation attribution and test/evidence attribution separately. Written by dot (AI).

## Waiting

- Validation executor: obtain the missing GUI/speech observations on an accessible native desktop with the pinned browser and relevant screen reader. Record OS/backend/window-system, browser/AT versions, fonts, scale, source SHA and artifacts; unavailable observations remain blocked.

## Related

- [Original component](/quest/m2/native-components/separator.md) - owns implementation correctness and retains its unresolved acceptance obligations
- [Native Gallery](/quest/m1/native-gallery.md) - owns broader cross-platform host and cross-component acceptance, not a substitute for these component checks
- [Native accessibility](/quest/m1/native-accessibility.md) - owns the OS bridge needed for real assistive-technology observations
