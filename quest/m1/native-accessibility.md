# [M] Connect the native accessibility bridge

## Goal

The native Gallery exposes existing component semantics and actions through the official platform accessibility bridge.

## Plan

The Gallery now enables egui-winit AccessKit, initializes the adapter before showing the window, forwards actions through the existing ordered input queue and publishes updates through the official platform output path. The runtime owns the adapter; events from replaced windows are rejected.

Controlled bridge checks cover activation/deactivation, stale-window rejection, ordered payload delivery and single consumption. Existing component tests provide modal, disabled and stable-ID behavior evidence. Core component semantics and the renderer lifecycle are unchanged.

The bridge implementation may land separately from specialist acceptance. This Quest remains open: the Gallery maintainer owns the remaining platform observations and must not report the bridge as fully accepted until they are recorded. Issue #29 remains a separate component defect, not resolved by host transport.

Remaining acceptance: record a representative real screen-reader interaction with exact platform/build identity. macOS VoiceOver, Windows UIA clients and Linux AT-SPI availability must be recorded separately. Unavailable platforms are explicit handoff gaps; build and synthetic events do not establish system acceptance. M1 Gallery aggregates the final platform matrix.

## Waiting

- Maintainer: complete one VoiceOver interaction on the identified macOS build: navigate to Interaction, announce and activate A once, confirm disabled actions cannot activate, then open and close Dialog A and check focus restoration. Automated GUI control was unavailable; the direct AX client verified delivery but cannot establish VoiceOver announcements/navigation. Record the result in docs/gallery.md. Windows/Linux observations remain separate platform gaps.

## Related

- [Dialog header semantics #29](https://github.com/shermerL/egui-prism/issues/29) - confirmed component semantic gap, separate from host adapter wiring
