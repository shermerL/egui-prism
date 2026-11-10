# [S] Publish the approved Rust packages

## Goal

The approved package set resolves from the selected Cargo registry and builds in a clean external consumer.

## Plan

Publish only approved package names, versions, dependency order and exact prepared candidates. Verify downloaded registry contents, licenses, documentation and consumer resolution. Record any partial upload accurately; do not silently replace public identity or upload unrelated packages. Source release and Pages publication are separate outcomes.

## Required

- [Registry readiness](/quest/m2/registry-readiness.md) - reproducible complete package graph

## Waiting

- Maintainer: approve the exact registry, package names/versions and checked candidates before upload. Automation can prepare and verify them, but cannot select the public publication commitment.
