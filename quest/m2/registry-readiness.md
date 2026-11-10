# [L] Prepare independently consumable Rust packages

## Goal

An approved registry dependency route lets external applications resolve and
build the core and compositor with one compatible egui type graph, without
relying on the repository workspace.

## Plan

Use the verified source consumer to evaluate registry distribution separately
from the M1 source preview. Cargo applies patches at the consumer workspace root;
publishing a library does not propagate its local patch configuration. A registry
consumer and docs.rs build must resolve every required API from the published
dependency graph. Removing publish=false is not a distribution design.

Compare compatible upstream support, an explicitly maintained published patch,
and retaining source distribution. Document type identity across egui, egui-winit
and egui-wgpu, ownership and update policy, public package identity and migration
cost. Do not create a public fork, contact upstream or publish a patched crate
before the concrete route is approved.

After that decision, prepare versioned package relationships and metadata,
package-file inventories, required source/assets/licenses, documentation and
examples. Verify cargo package and publish dry-runs, package unpacking, the
intended compiler baseline, a docs.rs-equivalent build and a clean external
consumer. Identify the dependency publication order and any prerequisite versions
that are not yet available; do not call a partial dependency chain ready.

Completion delivers checked package candidates and publication instructions.
Actual registry uploads require a separate exact-candidate publication decision.
GitHub Packages currently provides no Cargo registry; do not invent a wrapper
package or container just to populate the repository's Packages section.

## Waiting

- Maintainer: approve a concrete dependency publishing route before implementation changes public package identity, creates a maintained public fork or changes the host API. Recommend continuing the validated source route until the registry alternatives and maintenance costs are reviewable; automation may gather that evidence without making the public commitment.

## Required

- [Prepare a reproducible source preview](/quest/m1/source-preview.md)
