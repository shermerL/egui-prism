# [M] Connect the native accessibility bridge

## Goal

The native Gallery exposes existing component semantics and actions through the official platform accessibility bridge.

## Plan

The Gallery now enables egui-winit AccessKit, initializes the adapter before showing the window, forwards actions through the existing ordered input queue and publishes updates through the official platform output path. The runtime owns the adapter; events from replaced windows are rejected.

Controlled bridge checks cover activation/deactivation, stale-window rejection, ordered payload delivery and single consumption. Existing component tests provide modal, disabled and stable-ID behavior evidence. Core component semantics and the renderer lifecycle are unchanged.

Remaining acceptance: record a representative real screen-reader interaction with exact platform/build identity. macOS VoiceOver, Windows UIA clients and Linux AT-SPI availability must be recorded separately. Unavailable platforms are explicit handoff gaps; build and synthetic events do not establish system acceptance. M1 Gallery aggregates the final platform matrix.
