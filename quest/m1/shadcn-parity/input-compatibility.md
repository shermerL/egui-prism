# [L] Input compatibility for strict shadcn interaction

## Goal

Approved M1 components can process focus, ordered input and exact hit/capture
semantics without stealing host focus, losing events or widening their hit areas.
The same fixed egui source is used throughout each consumer's dependency graph.

## Plan

M1 owns the shared input capability required by strict component parity. Maintain
the smallest controlled patch set against egui 0.37.0 for focus policy, event
ordering, authoritative hit routing and capture. The existing egui focus filter
and frame-wide pointer snapshot cannot express all observed reference behavior;
this is an upstream integration boundary, not a new component design.

Base changes on the pinned official egui source and actual shadcn reference
behavior. Preserve upstream copyright and licenses; record upstream identity,
each patch's purpose and touched boundary, and reproducible application or source
verification. Keep renderer and platform integration on their official packages.
Do not create a public fork repository, post upstream or publish packages here.

Respect event order when focus changes within a batch. Use the authoritative
widget/layer geometry and capture owner for each pointer event. Preserve clipping,
transforms, modal isolation, disabled safety and overlapping controls. Capture
release must respect an explicit host focus surrender. Keep input plugins, text
composition, keyboard state and multi-viewport ownership coherent. Clean up
registration and capture on disable, disappearance, viewport removal or restart.

Do not replay coordinates outside authoritative routing, restore focus without
ownership evidence, discard raw events, broaden bounding boxes into clickable
corners, or depend on an extra UI pass. Regressions must cover first-focus input,
same-batch Tab/Arrow/Escape transitions, multiple pointer actions with different
positions, host surrender, one-pass and multi-pass execution, modal/overlapping
controls and IME interruption. Report failures as open gaps, not tradeoffs that
erase an existing passing behavior.

Document the minimal component-facing API and necessary M1 migrations. The
consumer application root must select this same egui source for the core and
official integration crates; a dependency-local Cargo patch is insufficient.
Verify a standalone consumer, one-source dependency resolution and type identity.
Keep root manifest and lockfile changes in this Quest; component Quests adopt the
reviewed capability and retain their own exact-head interaction acceptance.

Use controlled regressions for the framework and record actual macOS, Windows
and Linux input/IME observations separately when available. Synthetic composition
and compilation do not establish platform IME acceptance. The native Gallery and
component Quests own end-to-end platform acceptance; browser delivery is deferred.
Complete the shared implementation, checks, migration guidance, independent
review, GitHub code/security review and CI before dependent component adoption.
