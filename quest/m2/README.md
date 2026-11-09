# M2: Complex components and platform completeness

## Goal

Extend the library with evidence-driven complex components and improve native
input, accessibility and release readiness. Reconsider browser delivery only
after native macOS, Windows and Linux requirements are satisfied.

## Plan

This milestone is a scope basket, not an approved implementation task. Its browser
tasks preserve deferred preparation and publication work, not a support promise.
Plan independently useful quests when concrete requirements are available: bounded
ScrollArea/List behavior, Tabs, Popover, Tooltip, composition layouts, and package
publication readiness. Do not invent a full Web widget catalog or general overlay
framework in advance. Quantify virtualization requirements before choosing one.

Record macOS, Windows, Linux, browser, IME, DPI, and assistive technology evidence
separately. Publishing a crate requires package contents, dependency/license
checks, usage examples, and a compatibility/migration policy.

## Waiting

- Maintainer: choose concrete M2 outcomes and API scope after the core gallery provides evidence; recommend planning from observed gaps rather than implementing the whole basket. The tradeoff requires product judgment, so automation can collect evidence but cannot select the scope.

## Required

- [Browser gallery decision and preparation](/quest/m2/browser-gallery.md)
- [First Pages deployment](/quest/m2/pages-deployment.md)
