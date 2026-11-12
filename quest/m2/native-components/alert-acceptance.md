# [M] Inline Alert targeted test and native acceptance coverage

## Goal

Close the specifically missing automated, pinned-reference, native interaction and accessibility evidence for [PR #33](https://github.com/shermerL/egui-prism/pull/33), with reproducible fixtures and revision-bound evidence. This does not waive the original component Quest, authorize merging, or claim that a test plan is a passing result.

## Plan

Use the [fixed parity contract](/docs/shadcn-parity.md). The implementation and its known correctness gaps stay owned by /quest/m2/native-components/alert.md; coordinate fixes there rather than duplicate production changes in the acceptance PR. Add tests and evidence on the actual candidate, retaining a before-fix failing case for each defect. A separately reviewable test/evidence PR may wait for the fix without claiming acceptance.

The [shared acceptance protocol](/docs/component-acceptance.md) defines source identity, environment, oracle, evidence rows, integration invalidation and exit gates. Every result must name the tested head and distinguish tests, rendered browser/native observations, and AT. Use the unmodified pinned reference and matched font/theme/scale. Do not replace a required observation with a successful CI job.

1. Use the shared [wrapping measurement harness](/docs/alert-wrapping-evidence.md): its controlled native corpus runs in CI, but browser execution and comparison remain blocked, not passed. Compare wrapping at 448/320/160pt and below padding-plus-icon width: `alpha-beta/gamma`, URLs/long unbroken words, NBSP, CJK punctuation/mixed text and accented characters. Record browser min-content, line counts and word boundaries against native output before fixing supported hyphen/punctuation/CJK behavior. Forced clipping that differs from CSS overflow needs resolution or an explicit scope decision; existing documentation is not a pass.
2. Cover default/destructive, optional icon/title/description combinations, long-title ellipsis with complete accessible labels, light/dark and opacity. Map the existing nine tests and add only missing assertions.
3. With native AT active, insert a notice, change only description/title, repeat stable frames, remove/recreate and use two independent IDs. Expect complete title/body notifications for real content changes, no duplicate stable-frame notifications or stolen focus, and no omitted/repeated description. Compare browser role=alert in the same environment; record AT coalescing/interruption rather than imposing exact speech timing. Investigate bridge/host failures without adding a component-owned announcement queue.
4. Exercise a neighboring caller Button with actual click/Enter/Space: one action per valid activation, zero while disabled. The notice does not retry/dismiss itself. CSS gaps and omitted required notifications block landing; unavailable bridging remains an explicit native-accessibility dependency.

### Executor handoff and evidence

Use an executor that can run the pinned development shell, a sandboxed browser
and the native Gallery. Cloud loopback or socket restrictions are not permission
to disable Chromium's sandbox or substitute a mock. If native desktop access is
unavailable, a read-only CI job may collect headless browser geometry separately,
but it must first be implemented and validated; no such CI collector is delivered
by this Quest. It does not close native-window, real zoom or AT requirements.

For the existing description-only harness, run
`cargo test --locked -p egui-prism --test alert_wrapping -- --nocapture`, then build
and serve the unmodified pinned reference following
[the measurement instructions](/docs/alert-wrapping-evidence.md). Open
`/?alert-wrapping` in the actual browser and select **Measure loaded reference**
after Geist loads. Preserve the native TSV, browser JSON, screenshots and source
identities together. Capture light/dark at actual 100% and 150% browser zoom;
record DPR separately, since emulated DPR alone is not evidence of zoom.
Expect 28 browser samples per capture and 56 controlled native samples across
1x/1.5x. Missing samples, page errors or unloaded fonts fail collection.

Compare line count, preserved words/NBSP and break positions for each shared case,
not just total height or successful rendering. Keep raw character rectangles to
resolve ambiguous inferred lines. Inspect glyph coverage before comparing mixed
CJK; record unmatched fallback fonts as BLOCKED, not PASS. Record 24-point
NO_TEXT_PAINT as an observed native result, not a desired reference expectation.
A collected result is MEASURED, never automatically accepted; a difference needs
a fix with regression evidence or an explicit maintainer scope decision.

Name exact tested head/base and environment in the evidence manifest. When CI
checks a synthetic merge commit, retain that SHA as well as the PR head rather
than claiming they are identical. Attach durable evidence links and map each
sample to PASS, FAIL, NOT RUN or BLOCKED with its reason. Keep the native-window
and AT steps above as separate rows even when browser collection succeeds.

Before completion, include the automatable supported-case assertions in CI, provide native/reference captures and measured comparisons, record AT/input observations, link the applicable exact-head CI and independent review, and map every original requirement to PASS or a specifically retained blocking owner. FAIL, NOT RUN and BLOCKED rows prevent this Quest being called complete. No silent scope reduction or after-merge deferral of known parity defects. Only an explicit maintainer scope decision can permit staged landing; record the deferred requirements and owners without marking them PASS or deleting incomplete Quests.

Refresh evidence after affected implementation, reference, host or integration changes. Preserve original implementation attribution and test/evidence attribution separately. Written by dot (AI).

## Waiting

- Validation executor: obtain the missing GUI/speech observations on an accessible native desktop with the pinned browser and relevant screen reader. Record OS/backend/window-system, browser/AT versions, fonts, scale, source SHA and artifacts; unavailable observations remain blocked.

## Related

- [Original component](/quest/m2/native-components/alert.md) - owns implementation correctness and retains its unresolved acceptance obligations
- [Native Gallery](/quest/m1/native-gallery.md) - owns broader cross-platform host and cross-component acceptance, not a substitute for these component checks
- [Native accessibility](/quest/m1/native-accessibility.md) - owns the OS bridge needed for real assistive-technology observations
