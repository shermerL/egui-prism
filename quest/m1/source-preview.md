# [M] Prepare a reproducible source preview

## Goal

A clean external consumer can build and use the accepted M1 library from a fixed
source snapshot, and the maintainer can inspect a complete first-release candidate.

## Plan

Keep all eight M1 controls and native macOS, Windows and Linux as the preparation
target. Reuse native-gallery acceptance; do not duplicate component implementation
or claim a build proves GUI, IME or assistive technology behavior. Follow
/docs/releases.md. A narrower published support promise needs a concrete proposal
and maintainer decision, not an implicit waiver of an unfinished check.

Verify the source archive after extracting it outside the repository, without
workspace-only paths, private tooling or unpublished generated files. Document
the exact compiler, dependencies, supported targets and build commands. Prove
that the external application's root configuration selects one controlled egui
source for Prism, the official native host and the compositor. Include required
patch sources, notices, fonts and licenses. Build the canonical consumer example
against the extracted candidate rather than maintaining another installation
recipe. Keep its documented dependency and host configuration consistent with
the actual archive.

Prepare release notes, the proposed version and exact commit, archive contents,
checksums and the platform evidence matrix. Record known limitations and API
migration requirements. Use read-only CI to prepare candidates; publishing
credentials do not belong in PR jobs. Debug CI artifacts are not release assets.
Include runnable Gallery binaries only for explicit targets whose downloaded or
equivalent extracted payloads have been validated, with signing status stated.

Completion means the candidate is reproducible and ready for a concrete approval
request. Do not create a release tag, publish a GitHub Release or registry package,
deploy Pages, or create a public dependency fork as part of preparation.

## Required

- [Consumer examples](/quest/m1/consumer-examples.md) - canonical external integration to validate against the source candidate
- [Build and validate the native gallery](/quest/m1/native-gallery.md)
