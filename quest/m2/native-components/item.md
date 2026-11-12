# [M] Item composition with independent actions

## Goal

A generic item lays out leading content, text and trailing actions without overlapping hit regions or losing long-text access.

## Plan

Reference item.tsx and accepted Button/IconButton/Tooltip contracts. The caller supplies content, stable identity and actions. Do not introduce device/source models, inferred selection rules or a retained collection. Passive layout must not turn the whole row into an action when a nested control already owns input.

Cover long text and accessible full names, optional leading content, trailing buttons, disabled actions and constrained width. Distinguish a passive Item from an explicitly interactive Item and from selection controls. Reserve action space using real content measurements. Use the fixed reference presentation; any needed interactive extension requires a concrete reference and declared public scope.

Complete the implementation, compiling caller example, component documentation, native Gallery case and meaningful regressions in the same PR. Use /docs/shadcn-parity.md and the fixed shadcn source at 8a3536678a528cc012b9e62484290e79d0be410f. Include supported states, stable identities, caller ownership, clipping and accessible semantics; do not claim unsupported variants. Real desktop/browser/AT specialist checks are deferred under the maintainer's scope decision; their absence is not a hidden implementation-stage gate. Known implementation limitations remain separately stated. Keep source, controlled tests, real native input/GUI and untested platforms distinct. Reuse unchanged review evidence; final changed code still needs applicable independent/GitHub review and CI. Core stays egui-only; the host owns event order, fonts, frame/GPU lifetime and application policy. A materially new public boundary requires a concrete prototype and maintainer decision. No generic widget framework, application state manager or publication is included.

### Design checkpoint

The [public-path and internal-preparation probes](/docs/item-design.md) record actual
nested-action sizes and registration effects. The source's flex-wrap/min-content
and balanced two-line description remain implementation work, not just missing
visual tests. Under the maintainer's scope decision, real desktop/browser/AT
specialist validation can follow separately; basic geometry and action correctness
remain this implementation's responsibility.

A private measurement prototype and the existing Button regressions pass: reuse
that internal path rather than adding a public Button preparation API. No
maintainer decision is required merely because the original public interface
lacked measurement. The finite Item candidate now integrates that path. Basic tests cover final hit
rectangles, stable IDs, independent pointer/keyboard actions, removal and blur,
disabled controls, full accessible names, theme/scale and parent composition.
The related Button cancellation helper now observes the current ordered
WindowFocused(false) event; a standalone regression preserves valid intents
before and after blur/refocus in the same event batch.

Known implementation limits are not test-only follow-ups: the current
space-delimited minimum-width estimate and egui-based balancing/clamping are not
CSS text-balance/min-content equivalents. The 100ms shell fill/border transition
is implemented with controlled-time retargeting and idle/clipped repaint tests. The [supported finite slots and exact
limits](/docs/items.md) remain explicit. Do not mark complete fixed-reference
parity merely because the implementation or CI builds.

### Deferred specialist verification

At an exact candidate head, compare Gallery Items against pinned `/?items` in
light/dark themes at 100%/150% scale and wide/narrow widths. Record OS/backend,
browser and AT versions, screenshots of title/description wrapping, media offset,
action group placement, transparent borders and muted colors; exercise keyboard
traversal, hide/reinsert, disabled actions and per-action counts. Confirm full
caller text through real AT without container activation or lost button names.
Attach observations and failures separately per platform. These unperformed
specialist checks do not block the explicitly scoped basic implementation stage,
but do not resolve the documented text implementation limitations.
