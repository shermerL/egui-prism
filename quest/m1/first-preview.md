# [S] Publish the first approved preview

## Goal

The approved source preview is publicly available at an immutable version with
verified downloads, release notes and an accurate support statement.

## Plan

Publish only the version, commit, channel and asset set approved for the prepared
candidate. Recheck the exact commit's review, CI, dependency and acceptance
evidence before creating its tag and GitHub pre-release. Do not rebuild different
bytes silently or reuse an approval after a material candidate change.

Download the published assets, verify checksums and repeat the documented clean
consumer smoke check. Record the public URLs and outcomes. If publication partly
fails, report exactly which objects exist, preserve their identity and ask before
replacing or removing public artifacts. Do not mark partial delivery complete.

This outcome does not publish to crates.io or deploy Pages. Preparation remains
in the source-preview Quest; this Quest owns the external publication and its
download verification.

## Waiting

- Maintainer: approve the prepared version, exact commit, GitHub pre-release channel, assets and support matrix. Recommend reviewing the reproducible candidate first; tests cannot make the public support and publication decision. Existing implementation and merge permission does not cover release publication.

## Required

- [Prepare a reproducible source preview](/quest/m1/source-preview.md)
