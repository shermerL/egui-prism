# [M] Assess a native consumer integration route

## Goal

A representative native consumer has a concrete compatibility assessment and a bounded first-adoption proposal before any production migration.

## Plan

Use the canonical consumer example and docs/input-compatibility.md. A first
source-backed assessment can precede release preparation; a production pilot
must still identify its exact candidate and applicable platform evidence. Inventory the target application's egui version, host backend, ordered input migration, compositor ownership, fonts, accessibility and application-specific styling. Keep proprietary application details outside public library plans and external model materials.

Compare adapting the consumer to the supported dependency graph, a separately justified compatibility route and deferring adoption. Do not create an old-version support branch or upgrade the application implicitly. Identify one low-risk non-media control path, changes to host setup and actual migration cost. If the host change spans the whole application, say so; a one-control visual demo does not prove a local migration.

Use the example's demonstrated ownership and mixed-widget workflow instead of
creating another generic demo. Inventory actual consumer setup and migration
steps separately for dependency resolution, Context-wide input and renderer
submission. Distinguish library gaps, unavoidable host responsibilities and
application-specific work; do not infer video or device acceptance from a
settings example.

Acceptance is a source-backed compatibility matrix, exact library snapshot and documented pilot scope/rollback with an explicit decision request. It does not edit the consumer, replace its design system, introduce application policy into Prism or claim product/device acceptance.

## Required

- [Consumer examples](/quest/m1/consumer-examples.md) - fixed external-consumption evidence available before final source-preview preparation

## Related

- [Integration boundaries](/quest/m0/integration-boundaries.md) - necessity and supported public integration options
- [Source preview](/quest/m1/source-preview.md) - candidate preparation remains separate from this assessment
