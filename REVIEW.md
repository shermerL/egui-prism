# Review

Review the exact committed head against its base. Use the independent reviewer
and finding-resolution process in `quest context`; record unavailable review
capabilities honestly.

Use a reviewer model that wrote none of the changes. Pass `--exclude-model` for
every implementing model; the upstream default excludes only the latest writer.

Apply the ownership, state, layering, API, and implementation conventions in
AGENTS.md to the changed code; flag violations at the affected call sites.

Check these boundaries when relevant:

- Core dependencies stay independent of windowing, renderers, networking, and media.
- Callers own values and lifecycles; UI IDs isolate multiple instances.
- Disabled widgets emit no action; repeated frames do not repeat an action.
- Keyboard focus, popup dismissal, and modal isolation have behavior evidence.
- Drawing, hit targets, and value mapping agree across sizes and scale factors.
- Theme foreground/background pairs remain readable in every exposed state.
- Tests and examples exercise the public API, and documentation states its limits.
- CI is read-only; public files contain no private data or local handoff material.

Automated tests, screenshots, native observation, browser observation, IME, and
assistive technology checks establish different evidence. Name what ran and on
which platform. Do not infer another platform's acceptance from a successful build.
