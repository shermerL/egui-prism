# [M] Assess a native consumer integration route

## Goal

A representative native consumer has a concrete compatibility assessment and a bounded first-adoption proposal before any production migration.

## Plan

Use the accepted source consumer and docs/input-compatibility.md. Inventory the target application's egui version, host backend, ordered input migration, compositor ownership, fonts, accessibility and application-specific styling. Keep proprietary application details outside public library plans and external model materials.

Compare adapting the consumer to the supported dependency graph, a separately justified compatibility route and deferring adoption. Do not create an old-version support branch or upgrade the application implicitly. Identify one low-risk non-media control path, changes to host setup and actual migration cost. If the host change spans the whole application, say so; a one-control visual demo does not prove a local migration.

Acceptance is a source-backed compatibility matrix, exact library snapshot and documented pilot scope/rollback with an explicit decision request. It does not edit the consumer, replace its design system, introduce application policy into Prism or claim product/device acceptance.

## Required

- [Source preview](/quest/m1/source-preview.md) - fixed external-consumption evidence
