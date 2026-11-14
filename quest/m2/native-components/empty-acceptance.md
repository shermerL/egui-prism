# [M] Empty state targeted test and native acceptance coverage

## Goal

Close the specifically missing automated, pinned-reference, native interaction and accessibility evidence for [PR #35](https://github.com/shermerL/egui-prism/pull/35), with reproducible fixtures and revision-bound evidence. This does not waive the original component Quest, authorize merging, or claim that a test plan is a passing result.

## Plan

Use the [fixed parity contract](/docs/shadcn-parity.md). The implementation and its known correctness gaps stay owned by /quest/m2/native-components/empty.md; coordinate fixes there rather than duplicate production changes in the acceptance PR. Add tests and evidence on the actual candidate, retaining a before-fix failing case for each defect. A separately reviewable test/evidence PR may wait for the fix without claiming acceptance.

The [shared acceptance protocol](/docs/component-acceptance.md) defines source identity, environment, oracle, evidence rows, integration invalidation and exit gates. Every result must name the tested head and distinguish tests, rendered browser/native observations, and AT. Use the unmodified pinned reference and matched font/theme/scale. Do not replace a required observation with a successful CI job.

Automatable work uses the actual pinned `?empty` fixture and public Empty output. First confirm the candidate's documented route; do not collect a different page as an oracle. Export original text rows, visible bounds and available width from both engines at matched fonts and scales, and compare with a reproducible headless-browser runner. Include paragraphs on both sides of the selected browser's balancing line-count limit, not only short two-line copy, plus the existing repeated-word/NBSP cases and punctuation/CJK. Record the engine-specific limit and raw results rather than assuming every browser implements the current native binary search. Missing matched fonts or failed browser startup is BLOCKED, not native-only PASS. Commit only supported passing assertions after the corresponding fix, retaining a before-fix failure; do not normalize away line-count or break-position mismatches.

Use controlled public-API input sequences for caller-state removal/reinsertion, disabled actions, page return and focus loss, checking one/zero intent counts and retired IDs without network or wall-clock sleeps. Preserve the separate actual-window and AT observations below. Map existing tests before adding cases, put runnable checks in CI and retain source/version-bound safe mismatch artifacts. This does not authorize a new production balancing algorithm or redefine CSS parity.

1. Cross viewport widths 767/768/769 logical points with containers 140/320/wide. Root padding changes between 24/48 at the viewport breakpoint, not container width. Record the actual viewport under 150% zoom rather than using CSS transforms. Cover light/dark, optional icons, empty/omitted sections, complete/long copy, 384pt header/content maximum width, 40/24pt icon geometry and natural height.
2. Compare `office office office`, NBSP, long words, CJK/punctuation and paragraphs exceeding the browser balancing line limit. Record count/text/width of each line; allow no whitespace-only lines or truncated valid content. Resolve measured deterministic-balancing/CSS differences; 2,304 geometry probes do not establish browser parity.
3. In an actual window repeat empty-to-loading-to-failed-to-empty 20 times. Click/keyboard changes state only through the caller; disappearing buttons leave input/AT trees, and old focus/IDs cannot activate hidden actions. Add no network/timer/business state. Disabled actions produce zero commands. Repeat after page return and focus loss/reentry.
4. Native AT keeps empty copy passive, with no fabricated alert/busy/live semantics or focus stealing. Check caller-provided loading/error presentations separately; Empty does not own announcements. Verify documented explicit-Group opacity and direct-paint boundaries separately, without claiming arbitrary GPU callback capture.
5. CSS balancing/Unicode defects, stale actions and incorrect breakpoints block landing. Unverified different-model independence remains a review gap unless explicitly excepted for that exact task.

Before completion, include the automatable supported-case assertions in CI, provide native/reference captures and measured comparisons, record AT/input observations, link the applicable exact-head CI and independent review, and map every original requirement to PASS or a specifically retained blocking owner. FAIL, NOT RUN and BLOCKED rows prevent this Quest being called complete. No silent scope reduction or after-merge deferral of known parity defects. Only an explicit maintainer scope decision can permit staged landing; record the deferred requirements and owners without marking them PASS or deleting incomplete Quests.

Refresh evidence after affected implementation, reference, host or integration changes. Preserve original implementation attribution and test/evidence attribution separately. Written by dot (AI).

## Waiting

- Validation executor: obtain the missing GUI/speech observations on an accessible native desktop with the pinned browser and relevant screen reader. Record OS/backend/window-system, browser/AT versions, fonts, scale, source SHA and artifacts; unavailable observations remain blocked.

## Related

- [Original component](/quest/m2/native-components/empty.md) - owns implementation correctness and retains its unresolved acceptance obligations
- [Native Gallery](/quest/m1/native-gallery.md) - owns broader cross-platform host and cross-component acceptance, not a substitute for these component checks
- [Native accessibility](/quest/m1/native-accessibility.md) - owns the OS bridge needed for real assistive-technology observations
