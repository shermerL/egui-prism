# Toolchain

## Quest

`flake.nix` selects the official Quest CLI from the source mirror
[shermerL/quest at 19a2438cd80cf968dddfbf664570a2c8b9a254bf](https://github.com/shermerL/quest/tree/19a2438cd80cf968dddfbf664570a2c8b9a254bf).
`flake.lock` also fixes its Nix dependencies and content hashes. This revision
matches the upstream source inspected for adoption. The CLI reports `0.1.0`;
that number alone does not identify its behavior.

The project shell selects `quest.packages.<system>.default`, not the upstream
runner or upstream development shell. Its Bun runtime is supplied by the locked
Nix package (1.4.2). The upstream flake also records a separate `quest-pinned`
input for its own merge-driver development; that older input is not this
project's selected CLI.

The flake exposes Apple Silicon macOS, x86_64 Linux, and ARM64 Linux. Only Apple
Silicon macOS has been exercised during bootstrap. Intel macOS and native Windows
need a separately verified CLI installation route; the older v0.1.0 release does
not establish parity with this source revision. This tooling limitation is not
a claim about eventual widget runtime support.

`quest init` installs the upstream skill stubs, creates the task root, adds the
agent reference and `.gitattributes` entry, and configures the clone-local merge
driver. Run it inside the shell so `quest` is also available for later merges.
It does not enable global hooks or start an unattended service.

Upgrade by changing the explicit source SHA, regenerating the lock, reviewing
upstream behavior changes, and rerunning initialization and checks. Do not
silently follow the mirror's main branch or substitute an older cached binary.

## Planned Rust baseline

M0 will pin Rust 1.96.0, egui and eframe 0.37.0, and the gallery's wasm build
tooling after building the native and browser targets. The official egui 0.37.0
manifest requires Rust 1.95 or later. These are engineering candidates, not a
published compatibility promise. No consumer application is upgraded by this
repository's choice.

The core will depend on egui. The gallery will use eframe with an explicitly
selected renderer. Start by evaluating the glow/WebGL path for the first shared
native/wasm slice; do not add a silent WebGPU fallback. Lock transitive Cargo
dependencies for reproducible gallery and CI builds.

Sources: [egui 0.37.0](https://docs.rs/crate/egui/0.37.0),
[Quest setup](https://github.com/moq-dev/quest/blob/19a2438cd80cf968dddfbf664570a2c8b9a254bf/SETUP.md),
[Quest package](https://github.com/moq-dev/quest/blob/19a2438cd80cf968dddfbf664570a2c8b9a254bf/nix/package.nix).
