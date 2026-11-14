# [M] Complete native accessibility bridge acceptance

## Goal

Complete the remaining system-client and screen-reader evidence for the native bridge delivered by [PR #28](https://github.com/shermerL/egui-prism/pull/28), without treating its implementation-stage merge as full accessibility acceptance.

## Plan

The Gallery now enables egui-winit AccessKit, initializes the adapter before showing the window, forwards actions through the existing ordered input queue and publishes updates through the official platform output path. The runtime owns the adapter; events from replaced windows are rejected.

Implementation baseline: merged `d56080c0a777a917d77bc5f83f0d0378d45f5a2b`; the combined PR28/32/39 baseline is `7ab2a044dc3d15754907bc47fc896319774d21dc`. Record the actual tested revision and embedded build ID if using a later candidate. Keep the three controlled bridge regressions for activation/deactivation, stale-window rejection, ordered payload delivery and single consumption. Existing component tests provide modal, disabled and stable-ID behavior evidence; rerun affected checks if a discovered integration defect needs repair, rather than rebuilding an unrelated acceptance matrix.

The bridge implementation may land separately from specialist acceptance. This Quest remains open: the Gallery maintainer owns the remaining platform observations and must not report the bridge as fully accepted until they are recorded. Issue #29 remains a separate component defect, not resolved by host transport.

Remaining acceptance, owned by the Gallery maintainer and a validation executor with the relevant desktop/AT available:

1. On macOS VoiceOver, navigate to Interaction, record the announced action name/role, activate A once and verify A increments once while B is unchanged. Disabled actions must not activate. Open and close Dialog A; record modal background isolation, the actual focus destination and whether focus returns to the opener. An AXPress result or an application focus ID is not a VoiceOver speech/navigation or system-focus result.
2. Record Windows UIA and Linux AT-SPI availability and corresponding name/role/action, disabled/modal and focus observations separately. Name the actual client and distinguish a programmatic platform-client observation from a representative screen-reader pass. Do not infer either platform from macOS or a successful build. Record Linux window system where relevant; an unavailable environment remains BLOCKED, not PASS.
3. Update the observation record in docs/gallery.md with tested source/build, OS/backend, AT/client versions, cases, actual outcomes and safe artifact references. Preserve the earlier AX-client result as historical evidence at its original revision. Issue #29 remains a known missing Dialog title/description result, not a passed header check; this transport Quest does not close it or certify final Gallery acceptance.

Record PASS, FAIL, NOT RUN or BLOCKED for each evidence row and an owner for every outstanding row. Refresh only evidence affected by subsequent host/component changes. No platform permission changes, substitute semantic tree or custom action dispatcher are part of this work. M1 Gallery aggregates the final platform matrix.

## Waiting

- Maintainer: complete one VoiceOver interaction on the recorded current candidate build: navigate to Interaction, announce and activate A once, confirm disabled actions cannot activate, then open and close Dialog A and check focus restoration. Automated GUI control was unavailable; the direct AX client verified delivery but cannot establish VoiceOver announcements/navigation. Record the result in docs/gallery.md. Windows/Linux observations remain separate platform gaps.

## Related

- [Dialog header semantics #29](https://github.com/shermerL/egui-prism/issues/29) - confirmed component semantic gap, separate from host adapter wiring
