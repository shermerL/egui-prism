# [M] Upgrade the pinned Quest workflow

## Goal

Use a reviewed newer Quest revision with incremental review and directory-based
epic membership, while preserving the library's runtime dependencies and the
existing project development tools.

## Plan

- Pin upstream revision `0c1da19a74f77e4f662cf9536fb789aca686b3bc` and its locked
  dependencies. Keep the project's current nixpkgs independent of Quest's inputs.
- Migrate root, milestone, and epic child lists without changing real task
  dependencies or milestone scope. Compare ready and waiting work before and
  after migration.
- Verify the unmodified CLI in a native host context with process-query access.
  Document the macOS sandbox limitation without carrying a downstream gate patch.
- Verify private review records, incremental follow-up reviews, and reuse of a
  clean result for an unchanged head. Preserve independent review and concise
  public summaries.
- Validate the affected tooling and policy paths. Reuse component evidence when
  no component or project toolchain changes.
- Leave merge daemons, automatic pin updates, global hooks, and publication out
  of this upgrade.
