# Item composition: implementation boundary

This is a design checkpoint, not an implemented Item or a parity result.

## Fixed source and supported semantics

The source is [item.tsx at the fixed shadcn commit](https://github.com/shadcn-ui/ui/blob/8a3536678a528cc012b9e62484290e79d0be410f/apps/v4/registry/new-york-v4/ui/item.tsx).
It contains default/outline/muted variants, default/sm sizes, optional media,
content, actions, header/footer and a separate list wrapper. Its passive div does
not become a button. The `asChild` interactive case is a separate public scope.

Its root wraps flex lines; content has an automatic minimum width. Actions can
move as a group rather than always squeezing text. Description uses both
`text-balance` and `line-clamp-2`. Media with a description aligns to the top plus
2px. These are implementation requirements to resolve, not merely deferred tests.
Plain egui wrapping or silent truncation cannot be described as fixed parity.

## Minimal public-path probe

At base `c6bf5176c25ec07336b26a3839883f04bd6bf692`, with the pinned fonts and
light theme, register an Outline `Open` Button and a default
`Download original archive` Button. Their combined measured width plus the
reference's 8-point action gap is 277.75 points. With default Item padding (32
points total) and a 16-point content/action gap:

| Outer width | Inner width | Space remaining for text |
|---|---|---|
| 400 | 368 | 74.25 |
| 160 | 128 | -165.75 |
| 80 | 48 | -245.75 |

The gated diagnostic compiled and ran successfully. `Context::read_response`
already returns the registered button's rectangle and focusable sense before
any hypothetical final Item placement. Thus `Button::register` is not a pure
measurement call. This proves neither that layout is impossible nor that a
second registration is always invalid; it shows why a generic caller closure
cannot be blindly replayed as measurement and why moving only drawing after
registration would leave input geometry inconsistent.

## Recommended boundary decision

Prefer an explicitly prepared Button/IconButton layout value: prepare text/icon
geometry and intrinsic size without registering input, then consume it once when
registering in its final child UI. Reuse existing ButtonOutput event handling and
painting; do not introduce a new host/input association system. Existing normal
Button calls should reuse the same implementation internally, not duplicate
measurement constants or glyph calculations in Item.

Alternative: first restrict Item actions to a declared finite descriptor set.
That is a scope restriction, not support for arbitrary widgets, and still needs
one authoritative measurement implementation. Arbitrary effectful UI callbacks,
guessed fixed action widths and retained application models are not acceptable
shortcuts. This shared public-boundary choice requires the maintainer's decision
under the existing Item Quest before production implementation.

After that decision, establish deterministic width/wrap and two-line-description
behavior with full accessible names and explicit unsupported cases. Keep actual
layout limitations distinct from specialist browser/native/AT testing, which the
maintainer has deferred. That deferral is not permission to call an incomplete
text or input implementation complete.
