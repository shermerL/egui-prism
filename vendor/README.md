# Controlled egui input patch

`egui/` is the complete official **egui 0.37.0** crate distribution with the
reviewable changes in `egui.patch`. It is not a renderer fork or a published
package. Both upstream license files and upstream authorship are retained.

- Distribution: <https://static.crates.io/crates/egui/egui-0.37.0.crate>
- Registry record: <https://index.crates.io/eg/ui/egui> (version `0.37.0`)
- Archive SHA-256: `bec9144f49b7a28b5f395d3147e325b4fe3e77ba686eac11d1bdf0a7da3f375f`
- Licensing: upstream MIT OR Apache-2.0, retained in `egui/LICENSE-MIT` and
  `egui/LICENSE-APACHE`. Binary distributions also retain the egui notice in
  the repository's `THIRD_PARTY_NOTICES.md`.

The registry checksum and all 137 original archive files were checked before
patching. The archive's VCS metadata is retained as supplied, but is not used as
a separately verified GitHub commit identity. The distribution checksum is the
source pin.

## Inventory

| Boundary | Files | Purpose |
| --- | --- | --- |
| Distribution metadata | `Cargo.toml` | Independent workspace and `publish = false`; version and official dependency requirements unchanged |
| Input lifecycle and public entry points | `src/context.rs`, `src/context/input_dispatch.rs`, `src/lib.rs` | Opt-in registration/event/paint phases, current widget-tree routing, egui capture, pass-local part/rounding metadata, event and composition ownership |
| Focus state | `src/memory/mod.rs`, `src/memory/focus_policy_tests.rs` | Current-pass policy registration, ordered focus changes, interruption generations, current layer ordering and closed-viewport focus pruning |
| Authoritative geometry | `src/hit_test.rs`, `src/widget_rect.rs` | Shape predicates before native hit selection and removal of closed layer geometry |
| First-show modal placement | `src/containers/area.rs`, `src/containers/modal.rs` | Explicit layout-only measurement/registration using stock Area/Modal placement |
| Stock editor migration | `src/widgets/text_edit/builder.rs` | Register its existing navigation policy before ordered first-focus input |

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
