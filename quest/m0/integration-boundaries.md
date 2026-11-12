# [M] Assess core extensions and consumer integration boundaries

## Goal

Each controlled egui extension has a source-backed necessity assessment, and consumers can distinguish compile-time source requirements, Context-wide input migration and renderer integration costs without weakening the existing behavior contracts.

## Plan

Start from vendor/README.md, the controlled patch, docs/input-compatibility.md, the compositor integration guide and existing behavior regressions. Map each extension family to its observable requirement and a minimal failure case in unmodified egui. Reuse current tests rather than auditing every component again. A passing patched test alone does not prove that no public upstream API can satisfy the requirement.

Evaluate official egui extension points against the same causal, focus, capture, modal and lifecycle requirements. Classify each family as retain, narrow, replaceable or unresolved, with evidence and maintenance implications. Separate a different behavior contract from an equivalent implementation. Do not assume that restoring ordinary Widget syntax preserves same-batch event ordering or permits replaying arbitrary application closures.

Describe which work can live inside a component, an explicitly owned integration scope or the application frame. Inventory the actual root Cargo patch, egui type graph, registration/event/paint protocol, legacy-widget/plugin limits, compositor setup and supported host configurations. State plainly that opting into ordered execution is not the same as making the current crate compatible with unmodified egui.

Maintain the library-owned support contract defined in /docs/releases.md, linking detailed input and compositor requirements instead of duplicating them. Separate normative requirements, tested configurations, missing evidence and unsupported configurations. Account for exact source/features/type graph, declared versus tested compiler requirements, Time triggers, frame resources and platform evidence. Q1 demonstrates consumption, source-preview freezes a release candidate, consumer-fit selects an adoption route, and applications implement and accept their own products; none replaces the library as contract owner.

For each proposed new or expanded extension, require a minimal failure case for a real consumer or committed behavior, comparison with equivalent public APIs, regression coverage, migration cost and an upstream-equivalence exit condition. Patch size alone does not establish maintenance cost. Ordinary execution mode still uses the controlled source and does not prove an older application survives a dependency upgrade.

Use the fixed shadcn-egui comparison at commit 2e9b53058657fa26f70e27a8d0fa8f63ab1761fc as a reference for public extension points and encapsulation, not as a replacement visual or interaction specification. Preserve Prism's pinned shadcn/ui contract.

Acceptance is a reviewable necessity/compatibility matrix, targeted evidence for disputed boundaries, updated consumer documentation and concrete options for any remaining API decision. Keep unresolved experiments distinct from confirmed limits. This assessment does not remove patches, change public APIs, introduce compatibility modes or a hidden UI framework, publish forks, or migrate a production application. Propose implementation only after its benefit and equivalent behavior are established.

## Related

- [Consumer examples](/quest/m1/consumer-examples.md) - independent external usage evidence; neither task waits for the other's final report
- [Compositing baseline](/quest/m0/compositing-baseline.md) - rendering costs and correctness evidence, not input-policy authority
- [Consumer fit](/quest/m2/consumer-fit.md) - application-specific adoption assessment
