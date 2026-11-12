# [M] Add executable native consumer recipes

## Goal

A small native application demonstrates the supported public integration from
outside the Prism workspace, with runnable source, a tested ownership workflow
and documentation that stays consistent with that source.

## Plan

This extracts example ownership from source-preview preparation so useful
consumer evidence can land before final Gallery and release acceptance. Use the
merged public APIs and the existing tests/input-consumer dependency contract.
Keep that narrow type-compatibility fixture useful; a larger example does not
replace its focused regression. Preserve the egui-only core, official native
host and independent compositor; no new facade or host framework is required.

Before implementation, record the exact source candidate and the selected modal,
third-party widget and supported rendering configuration in the implementation
PR. Follow the library-owned support contract in /docs/releases.md; list missing
evidence explicitly. Do not combine successes from different branch heads into
acceptance of the selected candidate.

Provide one complete settings application with a modal editing flow using the
accepted controls. If it uses Prism Dialog, include repair and applicable
verification of the known title/description semantics defect in
[Issue 29](https://github.com/shermerL/egui-prism/issues/29). Ordinary egui Modal
is an allowed explicit alternative only with ordered-input behavior evidence;
it does not complete Prism Dialog acceptance. Missing platform observations
remain distinct from an unrepaired known component defect. Show caller-owned values, stable IDs, explicit drawing,
ordered native input, fonts and host/compositor initialization and cleanup.
Consume public Prism, compositor and official host APIs; do not import private
Gallery modules or test-only component setters. Explain root Cargo patch setup
and show one compatible egui type graph. State the supported compiler and host
configuration, the mandatory patched egui source, the Context-wide input protocol
and the renderer preparation point; do not imply stock eframe compatibility. Keep the example unpublished, build it
outside the repository workspace against a fixed source revision, and record
the supported commands and dependency configuration. Do not assume the parent
workspace's lockfile, patch table, generated files or private tools are present.

Exercise the same application update path with ordinary pointer, key and text
events: edit a value, redraw for an unrelated reason, edit again, then confirm
and cancel a modal edit. Assert caller-observable values, continued editability
and focus restoration. Mix Prism controls, ordinary egui and one selected
third-party widget through the documented event path. Identify any unsupported
legacy action or plugin behavior explicitly; a successful build alone does not
prove mixed input compatibility. Pin and justify the selected widget's source
and document any adapter the caller must provide. Reuse existing control
regressions; add only gaps exposed by this composition. Compile the documented native configurations in existing
platform CI and report controlled interaction evidence separately from OS IME,
screen-reader, GPU and visual acceptance owned by native-gallery.

Add a bounded synthetic current-frame GPU image producer using the existing
adapter conformance approach. Show ordinary image shapes in a Group and an
overlay consuming the current content. Identify who produces/uploads the image,
registers and retains the source, borrows the renderer for compositor preparation,
encodes and submits, then releases registrations on success, failure and restart.
Verify production precedes offscreen consumption and retained resources survive
submission. Do not introduce decoding, network sessions, device policy or a new
rendering contract; reuse unchanged GPU regressions rather than reproducing the
full conformance suite.

Give each complete recipe a stable source identity and distinguish it from
partial API snippets. Documentation should link to the canonical source; where
a runnable listing is repeated, check that its bounded source region matches.
Validate the source-to-document mapping and stale listings without generating
surrounding prose or duplicating every component page. Refresh the relevant
example checks when its API, host setup or copied listing changes.

Acceptance includes the external build instructions and result, the stateful
workflow regression, current documentation links/listings and explicit limits
on the observations made. Record the actual setup and migration steps, separating required consumer work
from reusable library integration. This delivers consumer examples, not a release
candidate or production-application migration. Do not add media, networking or
application-specific policy to establish this generic integration. If the recipe
requires a larger input patch, new host facade or unsupported render target,
stop expansion with minimal failure evidence and a concrete design choice.

Use [GPUI Kit's executable recipe practice](https://github.com/longbridge/gpui-kit/blob/8650093ccee70317fe96d48d53ead459620d38f0/examples/ai_recipes/README.md)
as an engineering reference. Adapt the ownership and evidence principles to
egui; the pinned shadcn contract remains the visual and interaction authority.

## Related

- [Integration boundaries](/quest/m0/integration-boundaries.md) - evidence can inform the assessment without waiting for its completion
- [Native gallery](/quest/m1/native-gallery.md) - integrated platform acceptance; share applicable workflows and evidence without making it a prerequisite for this example
