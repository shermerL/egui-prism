# Controlled egui input patch

`egui/` is the complete official **egui 0.37.0** crate distribution with the
reviewable changes in `egui.patch`. It is not a renderer fork or a published
package. Upstream authorship is retained; the license files omitted from the
crate archive are added from the official `0.37.0` tag.

- Distribution: <https://static.crates.io/crates/egui/egui-0.37.0.crate>
- Registry record: <https://index.crates.io/eg/ui/egui> (version `0.37.0`)
- Archive SHA-256: `bec9144f49b7a28b5f395d3147e325b4fe3e77ba686eac11d1bdf0a7da3f375f`
- Licensing: upstream MIT OR Apache-2.0, copied unchanged from
  [LICENSE-MIT](https://github.com/emilk/egui/blob/0.37.0/LICENSE-MIT) and
  [LICENSE-APACHE](https://github.com/emilk/egui/blob/0.37.0/LICENSE-APACHE) into
  `egui/`. Binary distributions also retain the egui notice in
  the repository's `THIRD_PARTY_NOTICES.md`.
- License SHA-256: MIT `95ca92f5f8ea5231f1580b3a2a799e8260af3114b900e1def5355a7f44bcf60c`;
  Apache `8173d5c29b4f956d532781d2b86e4e30f83e6b7878dce18c919451d6ba707c90`.

The registry checksum and all 137 original archive files were checked before
patching. The archive's VCS metadata is retained as supplied, but is not used as
a separately verified GitHub commit identity. The distribution checksum is the
source pin.

## Inventory

| Boundary | Files | Purpose |
| --- | --- | --- |
| Distribution metadata | `Cargo.toml` | Independent workspace and `publish = false`; version and official dependency requirements unchanged |
| License texts | `LICENSE-MIT`, `LICENSE-APACHE` | Add the unchanged official license files omitted from the crate archive |
| Input lifecycle and public entry points | `src/context.rs`, `src/context/input_dispatch.rs`, `src/lib.rs` | Opt-in registration/event/paint phases, explicit raw/time triggers, current widget-tree routing, egui capture including Escape drag termination, modal-blocked policy retention, pass-local part/rounding metadata, event and composition ownership |
| Continuous scrolling | `src/input_state/mod.rs`, `src/containers/scroll_area.rs` | Separate once-per-frame carry/physics from new event impulses; private per-instance consumption tokens, layout-only discard continuations and retained native wheel/physics math |
| Focus state | `src/memory/mod.rs`, `src/memory/focus_policy_tests.rs` | Current-pass policy registration, ordered focus changes, interruption generations, current layer ordering for routing and Area containment, and closed-viewport focus pruning |
| Authoritative geometry | `src/hit_test.rs`, `src/widget_rect.rs` | Shape predicates before native hit selection and removal of closed layer geometry |
| Widget updates | `src/response.rs`, `src/ui.rs` | Response interaction and Ui final geometry use an explicit private update path, preserving existing text sources and appended senses without renewing revoked sources |
| Modal placement | `src/containers/area.rs`, `src/containers/modal.rs` | Explicit layout-only measurement/registration and current modal barrier after stock Area placement |
| Built-in selection | `src/text_selection/label_text_selection.rs`, `src/text_selection/text_cursor_state.rs` | Event-time Label copy/cancel, directional keyboard ranges and shared stock range/press calculation; pass-local text references revoked on widget re-registration and renewed only by selectable labels |
| Stock editor migration | `src/widgets/text_edit/builder.rs` | Register its existing navigation policy before ordered first-focus input and preserve host focus on capture release |

The patch uses the existing `this_pass.widgets`, native hit selection, egui
pointer capture and directional focus search. There is no second retained widget
tree, renderer change, component coordinate replay or enlarged composite rectangle.
The architecture comparison consulted Iced's build/update/draw API and Dear
ImGui's event processing discussion; no code from those projects is copied here.

## Reconstruct and verify

```sh
nix develop --command quest gate python3 scripts/check_egui_source.py
```

The verifier checks the archive checksum, safely reconstructs its regular files,
applies `egui.patch`, and compares every resulting file byte-for-byte to `egui/`.
It uses a cached crate when available, otherwise downloads the checksum-pinned
archive. `--archive /path/to/egui-0.37.0.crate` selects an existing archive.
Unexpected files, missing files, changed licenses or a stale patch fail the check.

Run unit tests without placing generated output in the vendor directory:

```sh
nix develop --command quest gate cargo test --manifest-path vendor/egui/Cargo.toml \
  --locked --lib --features serde --target-dir target/vendor-egui
```

The pinned Rust/Clippy reports existing diagnostics in the unmodified crate
(type complexity, test unwraps and two unfulfilled lint expectations). The check
entry point therefore runs **the same strict Clippy command on both reconstructed
upstream and patched source** and rejects every added diagnostic, matching its
code, message, source file and source text. It prints both exit codes and retains
the logs locally. This is a regression comparison, not a claim that upstream
strict Clippy passes; no lint is disabled and no expected-diagnostic list is
silently refreshed from the candidate.

Keep the patch synchronized with every source change. Regenerate a unified diff
with paths relative to an extracted official crate (`a/…`) and `vendor/egui`
(`b/…`), review the inventory, then run the reconstruction check. Do not update
expected source bytes merely to silence a mismatch.

See [the host migration and evidence boundaries](../docs/input-compatibility.md).
The framework regressions and source/type checks are automated evidence, not
native OS IME, accessibility, renderer or component parity acceptance.
