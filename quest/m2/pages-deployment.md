# [S] Publish the first approved Pages gallery

## Goal

The approved gallery artifact is publicly reachable at the repository Project
Pages path, with its source identity, resources, and core interactions verified.

## Plan

This existing publication task is deferred from M1 with browser delivery.
Deployment is separate from building it so publication waits for review of a
concrete artifact without blocking authorized preparation.
Use the reviewed workflow and approved exact commit, serving below /egui-prism/.
Do not change the user root site, add a custom domain, or publish a crate.

After approval, run the protected deployment path and verify the live URL, refresh,
asset responses, wasm MIME, theme controls, Button/Input behavior, source identity,
and loading errors in a real browser. Record the deployed commit and observed
results. Remove this quest and its references in its completion PR; do not treat
a successful artifact build as a deployment result. If the deployment fails,
report the failure and fix the relevant configuration before claiming completion.

## Waiting

- Maintainer: approve the first Pages deployment after reviewing the exact gallery commit, build artifact, and browser evidence; recommend the repository Project Pages path rather than a user root site or custom domain. Publication needs a human decision; automation can prepare and verify the artifact but cannot grant that approval.

## Required

- [Build and validate the browser gallery](/quest/m2/browser-gallery.md)
