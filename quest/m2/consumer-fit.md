# [M] Assess a native consumer integration route

## Goal

A representative native consumer has a concrete compatibility assessment and a bounded first-adoption proposal before any production migration.

## Plan

Use the [source-backed integration boundary assessment](/docs/integration-boundaries.md) for current compatibility responsibilities and explicitly unresolved alternatives.

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

Select a candidate against the library-owned support contract in /docs/releases.md.
Deliver a service replacement table for windows/Context/viewports, platform
output, storage and identity, IME/clipboard/accessibility, repaint, surface/device
errors, renderer access, final submission and shutdown. Prefer evaluating a
bounded official native host, but do not infer its adoption cost from Gallery.
Compare a controlled eframe hook only with evidence that it can express the
required shapes, after-upload preparation, borrowing, errors and cleanup.
Choose one production route or document a specific blocker; do not maintain two
production hosts or let implementation rediscover an unresolved architecture.

Use the canonical recipe and minimal isolated probes for the decisive unknowns.
Keep experiments outside consumer source and everyday storage; this assessment
does not edit the consumer. It need not prove the entire unimplemented migration
successful. Identify shared build roots, dependency/API effects and the evidence
required when implementation actually changes them. Shared upgrades require
impact-matched compatibility evidence before landing, not a promise to repair
unmigrated platforms later. Reorder platform work if isolation cannot preserve
existing production behavior.

Define the transition contract: keep the old production entry while an isolated
candidate proves the new host; integrate complete Context input and resources
there; switch each platform's default launch/package entry only after its product
acceptance. A binary or feature alone does not isolate root patches, lockfiles
or shared path sources. Specify independent build/data/artifact identities and
any effect on production. Patched ordinary mode is an optional intermediate state
requiring evidence, not upstream compatibility or a runtime rollback promise.
Do not enable ordered input in production before migrating its full Context.
Retire temporary facilities per platform after acceptance and regression capture,
without removing APIs still needed by other platforms.

Before actual platform migration, require a retrievable, launchable old optimized
package with rebuild provenance, settings samples, recovery steps and comparable
behavior/performance baselines. Before the candidate writes everyday state,
exercise read/save/exit and recovery using disposable old-state copies. Before
default switching, repeat old-to-final-candidate-to-old with actual packages.
If old readers cannot consume the new format, demonstrate snapshot restoration
or conversion, not deletion of all configuration. Define workloads, measurement
methods, budgets and pause/rollback criteria before judging candidate results;
record compiler/dependency differences rather than attributing every change to
Prism. Keep package preparation distinct from final platform acceptance.

Specify resource owners and phase boundaries for host, coordinator, image
producer, compositor and UI, including renderer borrowing, final submission,
source reuse and failure cleanup. Keep native callbacks/background coordination
and Time triggers progressing without raw input; execute each valid intent once
and never replay arbitrary business closures during layout or paint. Using Prism
Dialog brings Issue 29's repair and applicable verification into the adopted
path, even if the full platform AT matrix is owned elsewhere.

Acceptance is the exact candidate, source-backed compatibility matrix, bounded
probe evidence, chosen host route, real pilot scope, transition/start conditions
and an explicit adoption decision request. Implementation and product acceptance
remain application-owned; their task carrier follows that repository's rules.
Keep application-specific inventories and evidence out of public library plans
and external model materials. This quest neither edits the consumer nor claims
product/device acceptance, replaces its design system or defines its business
policy. Independent architecture or confirmed defect work is re-scoped on its
merits, not hidden inside a fixed task-count budget.

## Required

- [Consumer examples](/quest/m1/consumer-examples.md) - fixed external-consumption evidence available before final source-preview preparation

## Related

- [Source preview](/quest/m1/source-preview.md) - candidate preparation remains separate from this assessment
