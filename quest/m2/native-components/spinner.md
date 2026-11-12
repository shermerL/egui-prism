# [S] Spinner

## Goal

A caller-controlled busy indicator matches the pinned shadcn geometry and motion without implying measurable progress.

## Plan

Follow [the fixed parity contract](/docs/shadcn-parity.md) and `registry/new-york-v4/ui/spinner.tsx` at shadcn commit `8a3536678a528cc012b9e62484290e79d0be410f`. Start from the existing transition timing and theme conventions; preserve the accepted egui-only core and host-owned input, frame and GPU responsibilities.

- Keep busy state and accessible status text at the caller. Own only frame-local drawing and any minimal stable timing required by animation.
- Schedule repaint only while the indicator is visible and active, respecting an available reduced-motion policy. Do not create a timer thread or interpret background work.
- Use controlled time to test motion and idle repaint behavior. Include the same indicator in an inline and button-adjacent Gallery example; do not fabricate percentage progress.

Complete implementation, the compiling public API example, component documentation, native Gallery case and meaningful regressions in one PR. Compare a mature candidate with the actual pinned reference at matching scale and theme. Report controlled tests, native observations and untested platforms separately. Review only the changed component and affected shared paths; reuse unchanged evidence. No host/vendor architecture change, new dependency identity or publication is included.

### Acceptance handoff (implementation candidate, not complete)

Automated coverage lives in `crates/egui-prism/tests/spinner.rs`; the collector
example and Gallery/reference fixtures exercise the public interface. Retain this
Quest until the comparison required above is actually observed, not merely built.

- Identify the exact PR head, pinned shadcn commit, Lucide 1.21.0, OS, renderer,
  browser/version, display scale and reduced-motion setting in every report.
- Run native Gallery Spinners and reference `/?spinners` side by side at matching
  100% and 150% scale, light/dark theme. Capture static reduced-motion frames and
  timed animated clips. Compare 16-point footprint, rounded arc/stroke, color,
  clipping and blue-background opacity sample. Record measurable discrepancies;
  do not substitute mesh-coordinate assertions for raster comparison.
- Observe at least three full cycles: clockwise linear rotation, one second per
  cycle without jumps during normal frames. Hide/reinsert and toggle reduced
  motion; static/hidden indicators must stop requesting animation frames after
  host settling. Separate other host repaint sources in traces. Scroll out/in,
  switch Gallery cases, and check no accumulated timer or stale per-ID phase.
- Verify inline and disabled-button-adjacent layouts at both scales; spinner is
  neither a button nor a progress bar. Keyboard navigation must skip it and the
  disabled button. No fake percentage or activation event may appear.
- With a real native screen reader (record adapter/version), change the caller's
  status text and insert/remove the indicator. Confirm useful polite status
  announcements without per-frame repetition, numeric progress or focus theft.
  Compare browser status behavior using its actual accessibility bridge. A
  serialized AccessKit node alone does not pass this check.
- Attach screenshots/video, frame timing or repaint observations, accessibility
  transcript and exact build IDs to the PR. Mark each platform checked, failed
  or not run separately. Record defects and reproducible steps in this same
  Quest rather than duplicating tasks or marking unobserved checks passed.

**Merge gate:** the mature-candidate comparison remains required. Headless tests
and CI do not waive it; no native/browser/AT acceptance is claimed by this handoff.
