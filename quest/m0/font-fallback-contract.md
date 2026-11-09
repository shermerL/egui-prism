# [S] Make standalone font fallback tests explicit

## Goal

Standalone library tests and workspace tests verify the same host-owned font contract without accidental dependency feature unification.

## Plan

Start at crates/egui-prism/tests/theme.rs, crates/egui-prism/src/theme.rs, Cargo.toml and gallery/Cargo.toml. Issue 20 already records the standalone failure and passing workspace control; inspect that evidence before deciding whether a new baseline run adds information. Supply an explicit, license-covered fallback fixture or explicit test-only feature. Keep runtime fallback coverage owned by the host; do not enable a new runtime dependency just to satisfy the test.

Acceptance: the named font_setup_retains_host_families_and_installs_each_weight_once case passes with both cargo test --locked -p egui-prism --test theme and the workspace invocation, with the intended feature graph recorded. Preserve host families, weight installation idempotence and notices. Document that bundled Geist does not promise all-language glyph coverage. This fixes the test contract, not native CJK/IME acceptance.

## Closes

- [#20](https://github.com/shermerL/egui-prism/issues/20) - explicit standalone fallback test contract
