# [M] Card targeted test and native acceptance coverage

## Goal

Close the specifically missing automated, pinned-reference, native interaction and accessibility evidence for [PR #34](https://github.com/shermerL/egui-prism/pull/34), with reproducible fixtures and revision-bound evidence. This does not waive the original component Quest, authorize merging, or claim that a test plan is a passing result.

## Plan

Use the [fixed parity contract](/docs/shadcn-parity.md). The implementation and its known correctness gaps stay owned by /quest/m2/native-components/card.md; coordinate fixes there rather than duplicate production changes in the acceptance PR. Add tests and evidence on the actual candidate, retaining a before-fix failing case for each defect. A separately reviewable test/evidence PR may wait for the fix without claiming acceptance.

The [shared acceptance protocol](/docs/component-acceptance.md) defines source identity, environment, oracle, evidence rows, integration invalidation and exit gates. Every result must name the tested head and distinguish tests, rendered browser/native observations, and AT. Use the unmodified pinned reference and matched font/theme/scale. Do not replace a required observation with a successful CI job.

1. Use the same long-word/Unicode corpus for header wrapping. Cover empty/short/long sections, omitted description, overflowing content, short/mixed/fractional footer heights and accented glyphs. Measure header/content/footer coordinates, 24pt padding/gap and 14pt radius; overflow must not enlarge later sections, and each closure runs once. Preserve the 15 regressions and add CSS-oracle comparisons, not implementation self-comparisons.
2. Compare GPU and actual visual output for two-layer shadow-sm in light/dark, light/dark/tinted backgrounds and 60% Group opacity; check parent clipping and exclusion inside the rounded border. Compare Button shadow-xs alongside it to detect shared-helper regressions. Two shadow geometries do not prove matching blur pixels.
3. Exercise nested Save Button/editor through native Tab/Shift-Tab, Enter/Space/click, disabled state, repeated rendering, page return and focus loss/reentry. Each valid input produces one action, disabled produces none, editor values survive drawing/page changes, and hit/visual bounds agree. The shell does not impersonate child click/focus. AT exposes child labels/order without fabricating an interactive container.
4. A stock egui editor is not Prism TextInput; its appearance need not match a shadcn input. CardAction is not an implemented public slot and is not added by this acceptance work. Header defects, visibly incorrect shadows and displaced input block landing.

Before completion, include the automatable supported-case assertions in CI, provide native/reference captures and measured comparisons, record AT/input observations, link the applicable exact-head CI and independent review, and map every original requirement to PASS or a specifically retained blocking owner. FAIL, NOT RUN and BLOCKED rows prevent this Quest being called complete. No silent scope reduction or after-merge deferral of known parity defects. Only an explicit maintainer scope decision can permit staged landing; record the deferred requirements and owners without marking them PASS or deleting incomplete Quests.

Refresh evidence after affected implementation, reference, host or integration changes. Preserve original implementation attribution and test/evidence attribution separately. Written by dot (AI).

## Waiting

- Validation executor: obtain the missing GUI/speech observations on an accessible native desktop with the pinned browser and relevant screen reader. Record OS/backend/window-system, browser/AT versions, fonts, scale, source SHA and artifacts; unavailable observations remain blocked.

## Related

- [Original component](/quest/m2/native-components/card.md) - owns implementation correctness and retains its unresolved acceptance obligations
- [Native Gallery](/quest/m1/native-gallery.md) - owns broader cross-platform host and cross-component acceptance, not a substitute for these component checks
- [Native accessibility](/quest/m1/native-accessibility.md) - owns the OS bridge needed for real assistive-technology observations
