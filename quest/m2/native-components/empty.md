# [S] Empty-state composition

## Goal

A generic empty state presents a title, explanation and optional caller action without fabricating loading or error semantics.

## Plan

The candidate provides Empty/EmptyUi/EmptyOutput, a public caller-state example,
controlled regressions, Gallery and unchanged pinned reference. Retain this
Quest until the following acceptance gaps close. No network/media lifecycle,
application state manager, dependency upgrade or host/vendor change is included.

Compare mature native and browser candidates at matching theme, font, viewport
and scale, including viewport widths around 768, narrow containers, long localized
copy, icon geometry, text line breaks, overflow and caller action hit targets.
The native squared-unused-width balancing and space-delimited min-content model
are not verified full CSS text-balance or Unicode line-breaking parity. Preserve
the saved Chrome 157 regressions at 216pt (ten rows), 244pt (seven rows) and 275pt
(six rows with matching words). The latter two originally failed because trailing
spaces affected fitting and minimum-width greedy balancing chose different breaks.
Reconcile hyphen, punctuation and matched-font CJK opportunities. Do not treat
remaining differences as an accepted substitute for the pinned source.

Keep controlled test, real native input/GUI, assistive-technology and untested
platform evidence distinct in /docs/empty-states.md. Review the final changed
code independently, obtain applicable GitHub code/security review when separately
authorized, and verify CI for the published head. Retain caller ownership and the
existing egui-only input/drawing boundaries; a materially new public boundary
requires a concrete prototype and maintainer decision.

## Waiting

- Native validation: run the Gallery and pinned reference on an accessible desktop with matching width/theme/font/scale and record real input and visual observations. The current headless execution environment has no native display; headless tests cannot establish these observations.
- Validation coordinator: keep remaining specialist evidence separate from implementation-stage landing. Record the review evidence used for each candidate in its PR; merging the implementation does not complete these acceptance checks.

## Related

- [Targeted acceptance](/quest/m2/native-components/empty-acceptance.md) - trace missing tests and native evidence without waiving this component quest.
