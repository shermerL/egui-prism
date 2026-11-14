# Alert wrapping measurement harness

Written by dot (AI). This is partial acceptance tooling, not a CSS parity result.

## Scope and reproduction

The shared corpus is `reference/fixtures/alert-wrapping.tsv`: ordinary prose,
hyphens/slashes, a URL, an unbroken word, NBSP, mixed CJK and accented Latin.
Both probes use description-only Alerts without icons or titles. No pinned
component source, font, dependency or production behavior is changed.

Native controlled probe:

```sh
cargo test --locked -p egui-prism --test alert_wrapping -- --nocapture
```

The normal workspace test command runs this test in CI. It checks outer width
and preservation of the complete layout input plus non-space row characters
and NBSP, not CSS line counts or every inter-word space. The 56 samples
cover 448/320/160/24 logical points at 1x/1.5x. Capture the `ALERT_CASE` and
`ALERT_ROW` TSV records. Text fields are UTF-8 hexadecimal, so spaces and NBSP
remain distinguishable; decode with Python `bytes.fromhex(value).decode()`.
CASE fields: ID, outer width, ppp, height, paint clip width, row count, input hex.
ROW fields: ID, width, ppp, row index, layout width, text hex. A CASE with
`NO_TEXT_PAINT` instead records absence of a Text shape, not an empty line.
Widths below the 34-point padding/border footprint currently omit text paint;
the probe reports this without requiring that behavior to remain after a fix.
Neither galley widths nor preserved strings establish visible glyph coverage.

Browser probe:

```sh
npm ci --ignore-scripts --no-audit --no-fund --prefix reference
npm run dev --prefix reference
```

Open `http://127.0.0.1:8084/?alert-wrapping`, use the actual browser's zoom
controls, then select **Measure loaded reference**. Save the displayed JSON
alongside the repository commit/tree, OS/browser version, zoom, fonts and source
hash verification. It waits for Geist and measures original text nodes with DOM
Ranges, not inserted per-character spans that would change grid layout.
The output includes raw per-character Range rectangles, inferred line strings,
widths, element/grid dimensions, overflow,
typography, viewport and DPR. Missing Range positions are explicitly listed.
Line grouping by Range top can be ambiguous with mixed fallback metrics; inspect
the raw rectangles before accepting any inferred line counts.
The UI deliberately labels output `MEASURED_NOT_ACCEPTED`, not PASS.

Compare native and browser at matching width/scale: line count and non-collapsed
word boundaries first; assess geometry separately. Preserve raw spaces/NBSP.
Browser layout Range widths and native glyph layout widths are not ink bounds.
Inspect the rendered glyphs separately. CJK remains font-blocked until matching
fallback coverage is demonstrated, even if text strings survive both probes.
This harness does not cover title/icon combinations, destructive style, live
announcements, input or native windows.

## Evidence recorded 2026-11-12

Candidate based on `962f8875bb5e342ddf9f6cc33ffeae3461c35ff0`, with the probe
introduced alongside this document, Linux and Rust 1.96.0:

- Controlled native corpus: PASS (56 samples); source text survives at supported
  content widths. At 24 points all seven cases report NO_TEXT_PAINT at both scales.
- Browser fixture production bundle: PASS. Browser execution and matched
  comparison: BLOCKED / NOT RUN, not inferred from the build.
- Local Chromium 156.0.8078.79 failed before page execution with
  `socket() failed: Operation not permitted`; the connected cloud browser
  could not reach the running loopback fixture (`ERR_CONNECTION_REFUSED`).
  No sandbox setting was disabled and no fixture was publicly deployed.
- At 160 points and 1x, native observed row counts were: plain 5, hyphen 4,
  URL 1, long word 1, NBSP 3, CJK 2, accent 3. These are observations, **not**
  golden CSS expectations. The source's space-only min-content calculation and
  clipping remain unresolved acceptance concerns.

The original Alert Quest and acceptance Quest remain incomplete. A validation
executor with a browser able to reach this fixture must collect the missing
measurements; native GUI/AT and matched-font comparison remain separate gates.
Changes to the implementation, fixture, fonts, reference or base invalidate
affected evidence. Exact-head CI and review status are linked from PR #33.

## Measured browser comparison, 2026-11-14

Measured source: `47546ae22eead7452dc5ab264ac5fa03fc8aa301`, on macOS
26.6.2 ARM64, Chrome 157.0.8103.51 headless. A fresh isolated browser profile
used default 100% zoom, DPR1, visual viewport scale1 and a 1280x813 CSS-pixel
viewport. The reduced browser user-agent reports Intel/macOS10_15_7; that is
not the host architecture or OS. No sandbox setting was disabled.

Served the existing production bundle on loopback and activated the existing
**Measure loaded reference** button after font loading. The fixture measured
28 actual pinned React Alerts through its original text-node Ranges. No spans,
CSS substitutes, page-scale emulation or component changes were inserted.
All18 reference asset hashes passed. Loaded pinned Geist assets were the
browser WOFF2 (`3626eba547294501c8a4d662c7d9ea3b49734409a4c5a1ccf12fa86429b1687e`)
and native Regular TTF (`2c460a7a71063e53491f343d44b382267bc0cf496e31db64865f962af3152e68`).
Native samples were freshly exported by the command above using an isolated
Cargo target and the pinned development shell. Only the native1x samples are
paired here; native1.5x was exported but browser150% zoom was NOT RUN.

| Corpus case | Outer width requested | Browser rows | Native rows | Observation |
| --- | --- | --- | --- | --- |
| URL | 320 | 2 | 1 | Browser breaks after `gamma?`; native retains one 360.84375pt row inside a286pt clip. |
| URL | 160 | 2 | 1 | Browser grid text track expands to193.203125px and breaks after `alpha-`; native keeps the complete URL row inside a126pt clip. |
| Hyphen | 160 | 3 | 4 | Browser keeps one `alpha-beta/gamma` per row. Native splits the middle copy after `alpha-`. |
| All seven cases | 24 | varies | no text paint | Browser outer width is34px, constrained by its padding/border footprint. Native outer width stays24pt and omits Text shapes. |

At width160, the browser hyphen row measures123.703125px excluding the
collapsed trailing space. Native reports126.5pt for its first row including
that space, while its final unspaced copy measures123.71875pt. Preserve raw
row strings and widths: this points to an additional whitespace/wrapping issue,
not proof that changing min-content or adding a line-break dependency alone
repairs the behavior.

The other15 non-CJK samples at widths448/320/160 have matching row strings
when only ASCII spaces at line boundaries are stripped. This is a limited
row comparison, not acceptance: raw spaces/NBSP remain in the exports, geometry
is assessed separately, and ink coverage/native windows/AT were not inspected.
All four CJK rows remain FONT BLOCKED because matching fallback coverage has
not been demonstrated, even where row counts happen to agree. Browser and
native raw exports and the derived comparison were retained locally; do not
reinterpret the successful native test or CI as a successful oracle comparison.

Result: browser execution is now MEASURED for this revision and environment,
but the paired comparison FAILS the documented strict text behavior. The
historical Linux browser failure above remains historical, not the current
reason this candidate cannot pass. The acceptance Quest still owns runnable
comparison automation/CI integration, font coverage and remaining platform
observations; known implementation differences need actual repairs.
