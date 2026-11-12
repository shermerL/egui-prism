# [M] Inline Alert targeted test and native acceptance coverage

## Goal

Close the specifically missing automated, pinned-reference, native interaction and accessibility evidence for [PR #33](https://github.com/shermerL/egui-prism/pull/33), with reproducible fixtures and revision-bound evidence. This does not waive the original component Quest, authorize merging, or claim that a test plan is a passing result.

## Plan

Use the [fixed parity contract](/docs/shadcn-parity.md). The implementation and its known correctness gaps stay owned by /quest/m2/native-components/alert.md; coordinate fixes there rather than duplicate production changes in the acceptance PR. Add tests and evidence on the actual candidate, retaining a before-fix failing case for each defect. A separately reviewable test/evidence PR may wait for the fix without claiming acceptance.

The [shared acceptance protocol](/docs/component-acceptance.md) defines source identity, environment, oracle, evidence rows, integration invalidation and exit gates. Every result must name the tested head and distinguish tests, rendered browser/native observations, and AT. Use the unmodified pinned reference and matched font/theme/scale. Do not replace a required observation with a successful CI job.

1. Compare wrapping at 448/320/160pt and below padding-plus-icon width: `alpha-beta/gamma`, URLs/long unbroken words, NBSP, CJK punctuation/mixed text and accented characters. Record browser min-content, line counts and word boundaries against native output before fixing supported hyphen/punctuation/CJK behavior. Forced clipping that differs from CSS overflow needs resolution or an explicit scope decision; existing documentation is not a pass.
2. Cover default/destructive, optional icon/title/description combinations, long-title ellipsis with complete accessible labels, light/dark and opacity. Map the existing nine tests and add only missing assertions.
3. With native AT active, insert a notice, change only description/title, repeat stable frames, remove/recreate and use two independent IDs. Expect complete title/body notifications for real content changes, no duplicate stable-frame notifications or stolen focus, and no omitted/repeated description. Compare browser role=alert in the same environment; record AT coalescing/interruption rather than imposing exact speech timing. Investigate bridge/host failures without adding a component-owned announcement queue.
4. Exercise a neighboring caller Button with actual click/Enter/Space: one action per valid activation, zero while disabled. The notice does not retry/dismiss itself. CSS gaps and omitted required notifications block landing; unavailable bridging remains an explicit native-accessibility dependency.

Before completion, include the automatable supported-case assertions in CI, provide native/reference captures and measured comparisons, record AT/input observations, link the applicable exact-head CI and independent review, and map every original requirement to PASS or a specifically retained blocking owner. FAIL, NOT RUN and BLOCKED rows prevent this Quest being called complete. No silent scope reduction or after-merge deferral of known parity defects. Only an explicit maintainer scope decision can permit staged landing; record the deferred requirements and owners without marking them PASS or deleting incomplete Quests.

Refresh evidence after affected implementation, reference, host or integration changes. Preserve original implementation attribution and test/evidence attribution separately. Written by dot (AI).

## Waiting

- Validation executor: obtain the missing GUI/speech observations on an accessible native desktop with the pinned browser and relevant screen reader. Record OS/backend/window-system, browser/AT versions, fonts, scale, source SHA and artifacts; unavailable observations remain blocked.

## Related

- [Original component](/quest/m2/native-components/alert.md) - owns implementation correctness and retains its unresolved acceptance obligations
- [Native Gallery](/quest/m1/native-gallery.md) - owns broader cross-platform host and cross-component acceptance, not a substitute for these component checks
- [Native accessibility](/quest/m1/native-accessibility.md) - owns the OS bridge needed for real assistive-technology observations
