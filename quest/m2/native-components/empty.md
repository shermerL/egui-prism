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
The deterministic native balancing search and space-delimited min-content model
are not verified CSS text-balance or full Unicode line-breaking parity. Reconcile
hyphen/punctuation/CJK opportunities and actual browser balancing where observed.
Do not treat those differences as an accepted substitute for the pinned source.

Keep controlled test, real native input/GUI, assistive-technology and untested
platform evidence distinct in /docs/empty-states.md. Review the final changed
code independently, obtain applicable GitHub code/security review when separately
authorized, and verify CI for the published head. Retain caller ownership and the
existing egui-only input/drawing boundaries; a materially new public boundary
requires a concrete prototype and maintainer decision.

## Waiting

- Native validation: run the Gallery and pinned reference on an accessible desktop with matching width/theme/font/scale and record real input and visual observations. The current headless execution environment has no native display; headless tests cannot establish these observations.
- Maintainer: authorize any desired GitHub automated-review task separately. The user has only authorized draft PR publication and prohibited merging; no Codex or automated-review task is started here.

## Related

- [Targeted acceptance](/quest/m2/native-components/empty-acceptance.md) - trace missing tests and native evidence without waiving this component quest.
