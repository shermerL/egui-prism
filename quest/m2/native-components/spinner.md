# [S] Spinner

## Goal

A caller-controlled busy indicator matches the pinned shadcn geometry and motion without implying measurable progress.

## Plan

Follow [the fixed parity contract](/docs/shadcn-parity.md) and `registry/new-york-v4/ui/spinner.tsx` at shadcn commit `8a3536678a528cc012b9e62484290e79d0be410f`. Start from the existing transition timing and theme conventions; preserve the accepted egui-only core and host-owned input, frame and GPU responsibilities.

- Keep busy state and accessible status text at the caller. Own only frame-local drawing and any minimal stable timing required by animation.
- Schedule repaint only while the indicator is visible and active, respecting an available reduced-motion policy. Do not create a timer thread or interpret background work.
- Use controlled time to test motion and idle repaint behavior. Include the same indicator in an inline and button-adjacent Gallery example; do not fabricate percentage progress.

Complete implementation, the compiling public API example, component documentation, native Gallery case and meaningful regressions in one PR. The implementation stage uses controlled tests and source inspection. Real browser/native comparison and assistive-technology acceptance are deferred to the separately owned specialist test work under the maintainer's explicit scope decision. Report controlled tests, native observations and untested platforms separately. Review only the changed component and affected shared paths; reuse unchanged evidence. No host/vendor architecture change, new dependency identity or publication is included.

### Deferred specialist acceptance handoff

Automated coverage lives in `crates/egui-prism/tests/spinner.rs`; the collector
example and Gallery/reference fixtures exercise the public interface. The checklist
below remains unperformed specialist work; retain it until transferred to its
follow-up test Quest, without treating it as an implementation-stage merge gate.

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

**Scope decision:** the maintainer explicitly assigned this implementation and
basic tests here and deferred real desktop/browser/AT checks to other work. Those
checks are not hidden implementation-stage merge requirements. This is a scope
deferral, not a passing result: no native/browser/AT acceptance is claimed. Normal
CI, code review and known implementation defects remain applicable merge gates.
