# Contributing

The workspace contains the egui-only library in `crates/egui-prism` and the
native/WebAssembly host in `gallery`. The host displays semantic themes, Prism buttons and text inputs, and labeled egui harness
controls. Other components are subsequent quests.
The [quest tree](quest/README.md) is the task source.

## Development tools

Use Nix with flakes enabled on Apple Silicon macOS or Linux:

```sh
nix develop
quest init
quest context
```

The locked shell supplies Quest, rustup, Trunk, wasm-bindgen, Python, and lint
utilities. Rustup installs the exact toolchain and wasm target declared in
`rust-toolchain.toml` on first use, without changing the global default Rust
toolchain. See [tool provenance](docs/toolchain.md) for versions and limits.

Nix reads tracked files. When testing newly created, untracked flake inputs,
use `nix develop path:.` until those files are staged.

## Checks

Run the complete local and CI entry point through the resource gate:

```sh
nix develop --command quest gate python3 scripts/check.py
```

It checks Quest structure, Nix and Rust formatting, workflow lint, the standalone
core library, native and wasm Clippy, tests and doctests, documentation, native
builds, and a release wasm bundle. Finally it checks generated assets and license
notices below `/egui-prism/`. Cargo and Trunk use `Cargo.lock` without updating it.
To check only the native path while iterating:

```sh
nix develop --command quest gate python3 scripts/check.py --native-only
```

The library has compiling documentation examples and public component behavior tests.
Add meaningful regressions with each component quest. A green
build is not evidence of real input, IME, accessibility, or another platform's GUI.

## Run the gallery

Native:

```sh
nix develop --command cargo run --locked -p egui-prism-gallery
```

Browser, after the full check builds the bundle:

```sh
nix develop --command python3 -m http.server 8080 --directory dist
```

Open `http://localhost:8080/egui-prism/`. The browser requires WebGL 2. The host
selects glow explicitly, with no WebGPU fallback. The bundle is under
`dist/egui-prism/`, including license notices. Local serving does not deploy Pages.

## Changes and review

Read `quest guide`, plan one independently completable outcome per quest, and
claim approved work with `quest start`. Include implementation, runnable examples,
documented ownership, and relevant checks in the same component PR. Keep transient
UI state isolated by stable IDs and preserve third-party notices.

Append commits and merge base updates into pushed branches. Run checks and obtain
an independent review of the exact committed head before marking a PR ready.
A ready PR is a candidate for maintainer review, not an instruction to merge.
The maintainer selects which reviewed changes may land; releases and the first
Pages deployment are separate decisions.

PR CI has read-only permissions, skips drafts, and uploads native/wasm artifacts.
It contains no deployment job. Keep private notes and raw review logs out of
public changes.

## PR descriptions

Use the [PR template](.github/PULL_REQUEST_TEMPLATE.md). Keep the description
short, in English, and organized around the final change. Scale the detail to
the change; a small correction does not need a work log or a file inventory.
Update the title and body when the scope changes.

- **Problem:** state the observable problem and why it needs fixing.
- **Approach:** explain the resulting behavior and the important implementation choice.
- **Impact:** list public API and behavior changes, with migration steps for breaking
  changes. Mark a breaking title with `!`. Include renderer or platform requirements
  when they change; omit unrelated protocol or wire-format boilerplate.
- **Alternatives:** explain a meaningful tradeoff, or state that none was needed.
- **Validation:** give the relevant check results and distinguish actual native/browser
  observations from automated tests. Identify the tested revision when it differs
  from the current head, and state the remaining acceptance gaps.
- **Review:** summarize the latest independent review and its reviewed head, plus the
  GitHub code and security review status. Link to the reviews where available and
  state unresolved findings. Keep reviewer names short; do not include model variants,
  effort settings, scores, internal session identifiers, or raw review transcripts.
- **Follow-ups:** link concrete remaining work, or state that none is needed.

Add **Decisions** only for consequential choices that help the maintainer assess the
change. Describe the selected choice and its reason without copying a conversation.
Keep **Waiting** when a draft has unresolved work or an external dependency; name
the actor, condition and verification. Use `Maintainer:` only for a decision or
action that an agent cannot take, not for routine implementation or verification.

Preserve required Quest relationships and current review evidence when shortening
tool-generated text. Do not add `Models`, `Size` or `Implementer` blocks. Attribution
must follow the applicable contribution authorization; the template does not grant it.
Request GitHub code and security reviews for the candidate head and address their
feedback alongside the independent review. Concise descriptions do not relax these
checks or turn an earlier revision's review into acceptance of a later one.
