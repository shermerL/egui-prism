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
and preservation of complete laid-out text, not CSS line counts. The 56 samples
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
The output includes raw line strings, widths, element/grid dimensions, overflow,
typography, viewport and DPR. Missing Range positions are explicitly listed.
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
affected evidence. Exact-head CI and independent review are linked from PR #33.
