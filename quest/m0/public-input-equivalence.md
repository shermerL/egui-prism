# [M] Test minimal public-API input equivalence

## Goal

Determine whether a phase-aware integration using unmodified egui public APIs can preserve the minimal Arrow/Tab/Arrow and modal open/close behaviors identified by [PR #39](https://github.com/shermerL/egui-prism/pull/39). Deliver reproducible equivalence evidence or a precise observed mismatch, not a production adapter or a claim about every possible public API design.

## Plan

This is M0 shared integration-boundary research, separate from the completed source assessment. The combined implementation baseline is `7ab2a044dc3d15754907bc47fc896319774d21dc`; [the assessment](/docs/integration-boundaries.md) retains its own fixed source baseline. Neither a failed compile against patched-only methods nor a passing patched regression answers the public-API question.

Reuse the isolated official-egui workspace in tests/upstream-boundaries and its registry/archive identity checker. Preserve the existing one-batch versus split-frame experiment as a baseline, not as proof that a deliberately phase-aware alternative cannot work. Keep the controlled and unmodified dependency graphs separate, pinned and reported; do not silently compare two patched builds.

Add bounded executable comparisons for:

1. Arrow, Tab, Arrow in one ordered batch: the first action belongs to the initially focused control, and the last belongs to the newly focused control with its current policy. Compare the relevant one_pass_arrow_tab_arrow behavior in crates/egui-prism/tests/input_dispatch.rs, including action order/count and final focus/value state.
2. A modal opened and closed within the ordered stream, followed by a background action: compare the relevant actual_modal_open_replace_close_then_background behavior in the same file. Observe current geometry, target eligibility and focus transitions; no background action may leak while the modal is active, and closed structure must not swallow a later eligible action.

Make application-side effects observable. Record action delivery/count, registration/layout versus business-callback invocations, logical frame/time advancement, and relevant plugin consumption/platform outputs. Repeating layout is not permission to replay arbitrary business work. If the prototype uses multiple public passes or hooks, identify their ownership, output handling and cost rather than reporting only final values. Use deterministic event streams and timestamps, not sleeps or native-window automation.

For each comparison, commit the minimal prototype and assertions, exact sources/features/toolchain, expected controlled behavior, observed upstream behavior and reproduction command. A successful alternative and a reproducible mismatch are both valid research outcomes; tests must assert the measured result without forcing a predetermined retain/remove conclusion. A mismatch identifies the tested design and causal failure, not universal impossibility. If repair is straightforward within this prototype, test it before concluding failure.

Integrate the bounded checks into the existing source-comparison CI path and update the assessment's corresponding unresolved rows. Preserve negative cases for duplicate actions, stale modal eligibility and callback replay. More capture/scroll/IME or plugin families require a separately justified scope; this Quest does not expand into a general compatibility framework or every-component audit. No production patch deletion, public API change, dependency upgrade, application migration or support declaration is included.

## Related

- [Integration assessment](/docs/integration-boundaries.md) - fixed evidence, current classifications and unresolved alternatives
- [Input protocol](/docs/input-compatibility.md) - behavior an alternative must preserve, not a requirement to copy its implementation
- [Compositing baseline](/quest/m0/compositing-baseline.md) - separate rendering-cost/equivalence work, not an input-prototype dependency
