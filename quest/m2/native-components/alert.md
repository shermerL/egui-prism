# [S] Inline Alert

## Goal

An inline notice presents a title and description with pinned shadcn sizing, overflow and clear accessibility semantics.

## Plan

Follow [the fixed parity contract](/docs/shadcn-parity.md) and `registry/new-york-v4/ui/alert.tsx` at shadcn commit `8a3536678a528cc012b9e62484290e79d0be410f`. Start from theme tokens and existing text/control composition; preserve host-owned input, frame and GPU responsibilities. Internal Alert text layout may use `unicode-linebreak`0.1.5, without adding an egui/epaint vendor patch or changing Card.

- The caller selects semantic presentation and provides text. Rendering the notice must not infer network success, retry operations or dismiss application errors.
- Match the pinned layout, icon slot, variant colors, wrapping and description typography. Compose any caller action using existing Button rather than owning its policy.
- Cover long text, constrained width, light/dark and the supported icon/description combinations. Include an example and relevant layout/metadata checks; this is not a timed toast manager.

Complete implementation, the compiling public API example, component documentation, native Gallery case and meaningful regressions in one PR. Compare a mature candidate with the actual pinned reference at matching scale and theme. Report controlled tests, native observations and untested platforms separately. Review only the changed component and affected shared paths; reuse unchanged evidence. No host/vendor architecture change or publication is included. The border box has the source's34-point minimum; description overflow reaches the parent UI clip rather than an Alert-owned content clip. Keep builder/output types unchanged. Unicode opportunities are not themselves complete CSS layout: preserve source-bound browser comparisons, whitespace/min-content rules and explicit unverified script/font cases.

## Waiting

- Native validation: compare the Alerts Gallery page with the actual pinned `?alerts` reference at matching size, font, light/dark theme and 100%/150% scale. Record real platform/backend and screen-reader observations separately from controlled metadata/layout tests. The headless implementation environment cannot establish these observations.

## Related

- [Targeted acceptance](/quest/m2/native-components/alert-acceptance.md) - trace missing tests and native evidence without waiving this component quest.
