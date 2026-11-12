# Pinned shadcn reference

This development fixture renders actual unmodified new-york-v4 component source
from the commit in `manifest.json`. It uses the same pinned CSS theme/base rules,
Geist package, and locked Radix primitives. It is a comparison tool, not the Rust
Gallery and not a deployable replacement for the library.

From the repository's Nix shell:

```sh
npm ci --ignore-scripts --no-audit --no-fund --prefix reference
npm run dev --prefix reference
```

The server binds only to 127.0.0.1 on port 8084. `npm run build --prefix reference`
checks production bundling. `prepare.mjs` extracts the exact theme, light/dark and
base blocks from the vendored global CSS and copies the package's variable font.
`python3 scripts/check_reference.py` checks source/font hashes. Updating the source
pin requires reviewing the manifest and the parity contract together.

Geist Regular/Medium/SemiBold and Geist Mono Regular TTF assets in the Rust crate come unmodified from the
same package. License notices are retained in LICENSE.shadcn and the crate's
fonts/OFL.txt, as well as THIRD_PARTY_NOTICES.md.

Observed in the browser: light/dark rendering, checkbox Enter leaves state
unchanged, Space toggles, Slider Home selects minimum, and Shift+Right advances
ten steps. This is reference evidence, not Rust component acceptance.

The `/?sliders` fixture isolates Slider comparisons with 448, 200, 20 and 16 CSS-pixel
rails, matching ranges and steps, light/dark themes, disabled states and 100/150%
scales. Its explicit 16-pixel minimum matches the Rust intrinsic thumb footprint.
Open `/?compositing` for real CSS group-opacity samples over multiple backgrounds,
opaque/translucent images, clipping, nested opacity, 100%/150% scale, and all pinned
Button roles. This fixture does not alter the pinned source components.

The `/?selects` fixture isolates item-aligned Select comparisons: independent
values and change counts, default/small triggers, disabled and invalid states,
long lists with disabled items, empty/all-disabled lists, missing selected
values, outside-action blocking, light/dark themes and tinted surfaces. Use
actual browser zoom for scaled comparisons. This fixture does not use CSS zoom,
which changes the coordinate mapping of Radix item-aligned positioning.

The `/?dialogs` fixture compares controlled A/B modal instances, neutral
dismissal, caller actions, long scrolling bodies, instance reversal, light/dark
themes and tinted backgrounds. The caller constrains body height to fit the
viewport; the pinned Dialog source is unchanged.

The `/?inputs` fixture compares caller-controlled text, placeholder, selection,
invalid and disabled states, reordering and narrow columns. Compare viewport widths
on either side of 768 pixels for the input typography breakpoint. Unicode inspection
separates committed text from platform font coverage during native IME checks.

The `/?checkboxes` fixture isolates the boolean Checkbox with controlled values
and counters, invalid and disabled states, reordered instances, long labels,
three compositing backgrounds, light/dark themes, and 320px content. It renders
the unmodified pinned component; use actual browser zoom for scale comparisons.

The `/?badges` fixture renders all six static Badge variants from the pinned
source, long text in a horizontal scroll area, and group opacity over a tinted
surface. Light/dark and 100%/150% controls match the native Badges page.
The Link variant remains a noninteractive span in both fixtures.

The `/?alerts` fixture uses the unchanged pinned Alert source for default and
destructive notices, optional icons/titles/descriptions, 448/160/320-pixel widths,
clipping and 60% parent opacity. Its descriptions are direct plain text children.
Light/dark and 100%/150% controls match the native Alerts page.

The `/?tooltips` fixture compares explicitly identified pointer and keyboard
triggers, immediate and delayed opening, hoverable plain-text content, disabled
triggers, viewport collisions and long bounded text. It uses the unmodified
pinned Tooltip source and Radix Tooltip 1.2.8; the shadcn provider defaults to
zero delay and the content defaults to zero side offset.
