# [M] Upgrade the pinned Quest gate

## Goal

Pin Quest to 4b07dd37466d6fe6af5c9de3d6f1efd7f693aba7 and validate its
ordinary and exclusive host gate on macOS without disrupting existing tasks.

## Plan

Update the exact Quest source revision, its lock entry and tool provenance.
Retain the locked Nixpkgs, Bun and Rust versions and the existing review harness.
This upstream update adds exclusive host admission and process start identifiers;
its other planning changes are not implemented runtime features.

Validate ordinary admission, host exclusivity, cancellation and PID/start-time
identity in isolated gate state. Verify the macOS process query with the actual
execution permissions. Existing jobs retain the prior tool until their review
rounds finish; do not claim mutual exclusion between old and new gate versions.
Upstream macOS lifecycle cancellation is not a passing result.

Run repository checks and independent review on the candidate, then leave the
version switch to the maintainer. This does not change application dependencies,
review providers, scheduling policy or publication behavior.
