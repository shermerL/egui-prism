# [S] Reduce duplicate Actions runner demand

## Goal

Reduce avoidable GitHub Actions runner requests and Cargo downloads without reducing the native platform matrix, test coverage or read-only PR security boundary.

## Plan

The Check workflow previously reran on Ready transitions and had no PR concurrency cancellation. Recent runs include five repeated PR heads in a forty-run sample. PR42 also encountered macOS arm64 hosted-runner capacity failures before any job step executed; scheduling improvements do not guarantee capacity or turn those results into PASS.

Keep PR opened, synchronize and reopened triggers, including Drafts, and main pushes. Do not retrigger on ready_for_review. Supersede earlier runs only within the same PR; keep different PRs and every main push independent. Retain all current native commands and the three desktop platforms. Use macos-26 ARM64 as the pinned macOS CI baseline, dropping macOS 15/14 coverage rather than maintaining an older-version or Intel fallback. Windows and Linux labels stay unchanged. This does not declare a release minimum OS or establish GUI/AT acceptance. Cache only public Cargo dependency downloads, separated by OS/architecture with lockfile/toolchain keys; never cache compiled targets, executables or credentials. No document-only skipping, paid runner changes or self-hosted setup is included.

Implementation is present in this candidate. Before completion, validate workflow syntax, scheduling cases and unchanged checks/security boundaries; obtain eligible independent and GitHub reviews; observe exact-head hosted execution and cache miss/hit behavior without claiming unobserved savings. Remove this Quest only when the candidate is qualified for completion. The scheduling deficiency dates to the bootstrap workflow; it is not caused by the later component PRs.

## Waiting

- Reviewer: review the exact committed candidate independently of its writer and complete the applicable code/security review gates.
- GitHub Actions: execute the final candidate on all three hosted platforms and establish download-cache miss/hit behavior. If macOS cannot acquire a hosted runner, retain the capacity failure rather than substituting another platform or reporting PASS.
