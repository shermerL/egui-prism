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

Open `/?compositing` for real CSS group-opacity samples over multiple backgrounds,
opaque/translucent images, clipping, nested opacity, 100%/150% scale, and all pinned
Button roles. This fixture does not alter the pinned source components.
