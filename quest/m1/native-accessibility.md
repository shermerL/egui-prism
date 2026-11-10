# [M] Connect the native accessibility bridge

## Goal

The native Gallery exposes existing component semantics and actions through the official platform accessibility bridge.

## Plan

Baseline inspection shows controlled AccessKit tree/action tests but no egui-winit accesskit feature or native adapter initialization. Start at gallery/Cargo.toml, gallery/src/native.rs, gallery/src/input.rs and the official egui-winit 0.37 State::init_accesskit/on_accesskit_action_request integration. Use the official bridge; the host owns adapter/event-loop lifetime and application restart/close cleanup.

Forward accessibility actions into the same ordered input path as other controls, and publish updates using the official platform output path. Preserve modal eligibility, disabled state and stable IDs. Component-owned roles/names/actions remain in their component work; fix only missing integration encountered here. No custom accessibility framework, parallel event dispatcher or blanket source audit.

Acceptance: a controlled native adapter/action integration check, a compiling official host path, and an actual representative screen-reader interaction record with exact platform/build identity. Record macOS VoiceOver, Windows UIA client and Linux AT-SPI availability separately; one observed platform does not prove the others. Unavailable environments remain explicit handoff gaps. M1 Gallery aggregates the final platform matrix.
