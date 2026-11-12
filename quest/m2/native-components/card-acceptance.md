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

### Executor handoff

Run the targeted `cargo test --locked -p egui-prism --test card`, then run
Gallery in the pinned development shell and select Cards. The compiling
`card` example and its recording compositor are not a real window. Build and
serve `reference/` following its README and open `/?cards` for the actual
pinned component. Keep the fixture's own scale control at 100% while testing
actual browser zoom at 100%/150%: its 150% button applies CSS `zoom`, not browser
zoom. Record viewport, DPR, application scale, fonts and both source SHAs.

For each theme and narrow/normal container, capture the ordinary and long-copy
Card plus the empty Card and tinted 60%-opacity sample. Record the same logical
bounds in native and browser output before judging padding, radius and shadows.
Use the existing native Save counter and browser Saved counter as independent
observations: one valid click/Enter/Space increments once, repeated idle frames
and disabled activation do not. Preserve the editor value through focus/page
return. Capture a short input trace or recording with expected/actual counts;
a screenshot alone does not establish keyboard behavior.

The current fixture does not supply every Unicode/min-content or mixed-height
case above. Add those missing samples to the harness without modifying the
pinned Card source, then record raw line/bounds measurements and original
screenshots for comparison. Do not cite the Alert fixture as Card evidence:
Card has different fonts, widths and section layout. Record missing sample,
font, shadow or AT observations as NOT RUN/BLOCKED with an executor and next
step; keep known header and input defects as blockers. Report the exact tested
head (and synthetic merge SHA if CI was used), artifacts and residual risk
before requesting any maintainer staged-landing scope decision.

Before completion, include the automatable supported-case assertions in CI, provide native/reference captures and measured comparisons, record AT/input observations, link the applicable exact-head CI and independent review, and map every original requirement to PASS or a specifically retained blocking owner. FAIL, NOT RUN and BLOCKED rows prevent this Quest being called complete. No silent scope reduction or after-merge deferral of known parity defects. Only an explicit maintainer scope decision can permit staged landing; record the deferred requirements and owners without marking them PASS or deleting incomplete Quests.

Refresh evidence after affected implementation, reference, host or integration changes. Preserve original implementation attribution and test/evidence attribution separately. Written by dot (AI).

## Waiting

- Validation executor: obtain the missing GUI/speech observations on an accessible native desktop with the pinned browser and relevant screen reader. Record OS/backend/window-system, browser/AT versions, fonts, scale, source SHA and artifacts; unavailable observations remain blocked.

## Related

- [Original component](/quest/m2/native-components/card.md) - owns implementation correctness and retains its unresolved acceptance obligations
- [Native Gallery](/quest/m1/native-gallery.md) - owns broader cross-platform host and cross-component acceptance, not a substitute for these component checks
- [Native accessibility](/quest/m1/native-accessibility.md) - owns the OS bridge needed for real assistive-technology observations
