# Engineering

Use the pinned development shell described in CONTRIBUTING.md. Keep the core
library dependent on egui, with native winit/egui-winit integration and renderer choices in the gallery.

## Ownership, data flow, and lifecycle

Make resource creation, ownership, borrowing, transfer, and release clear in
interfaces and call sites, along with inputs, transformations, and outputs.
Use shared references for read-only access; require mutable borrows or ownership
only when needed. Do not add getters mechanically.

Where asynchronous work or resources are involved, identify cleanup on success,
failure, cancellation, reentry, and restart. Prevent stale tasks or callbacks from
affecting a new instance.

## State and layers

Persist only ownership, observable state, and cleanup tokens needed across calls
or phases. Before adding a field, identify its later reads or cleanup purpose;
avoid duplicating derivable state. Keep construction and frame-local data local,
and pass transient Ui and drawing parameters through calls.

Focus, popup, and selection state may persist with stable IDs and clear reset
rules. Give caches a purpose and invalidation policy. Holding an egui::Context
is valid when the owner needs it; frame-local data rules do not forbid that.

Place responsibilities where the necessary context lives. Callers own business
policy, application values, networking, media sessions, and platform permissions.
The layer owning the frame context controls synchronization and execution order;
widgets and drawing helpers must not introduce hidden lifecycles.

## Interfaces and implementation

Expose only capabilities callers need; keep implementation types and state
private. Do not add configurable parameters whose values are identical for all
current valid calls. Prefer direct code; retain helpers and wrappers when they
provide reuse, an independent responsibility, or a real phase boundary. A single
caller alone is not grounds for deletion.

Within the approved change, remove replaced paths, unused abstractions, and
redundant state. This does not authorize deleting unrelated files, Git branches,
or user changes.

Do not follow an asserted programmer invariant with a duplicate conditional that
hides its violation through an early return or fallback. Retain external input
validation and recoverable error handling.

Preserve accurate, useful user explanations and move or update comments with
refactored code. Explain removal of obsolete or unhelpful comments. Retain
third-party copyright and license notices. Use the project formatter, with one
normal final newline and no extra blank lines at the end of files.

## Workflow and evidence

Use quest/ as the task source. Preserve unrelated changes and keep local notes,
logs, and review transcripts in ignored directories. Review public changes for
private paths, data, and credentials before staging explicit file paths.

Component changes include their gallery examples, API documentation, and relevant
behavior checks. Report automated checks, observed GUI behavior, and untested
platforms separately. Read REVIEW.md before reviewing a change.

Quests: before any work, run `quest context` and follow it, unless this session already holds its rules.
