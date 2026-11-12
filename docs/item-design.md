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

## Internal preparation is viable

A second, unpublished prototype extracted the existing Button measurement block
into a private `layout(&Ui)` helper, reused by ordinary registration. It did not
add a public API or change the host contract. A controlled test covered all four
Button sizes, primary/outline/link roles and IconButton: preparation preserves
the next automatic ID, registers no hit response, and returns the same size as
subsequent registration. That test and all 21 existing Button tests passed.

Therefore absence of a public sizing method is **not** a demonstrated blocker
and does not by itself require a maintainer decision. Prefer crate-private
preparation for Item's concrete Button/IconButton slots; do not expose a new
public preparation protocol merely for this internal consumer. Preserve one
measurement implementation and the existing ButtonOutput event/paint lifecycle.
Arbitrary caller widgets would require a wider declared composition scope; the
basic Item need not promise arbitrary effectful UI closures.

## Remaining implementation work

Establish deterministic width/wrap and balanced two-line description behavior
with full accessible names and explicit supported slots. The constrained-width
probe demonstrates why a fixed right-hand action width is insufficient; it is
not a browser oracle or proof of CSS flex/min-content parity. Keep these actual
layout requirements distinct from specialist browser/native/AT testing, which
the maintainer has deferred. That deferral is not permission to call an
incomplete text or input implementation complete.
