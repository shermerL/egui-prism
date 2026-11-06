# Contributing

This repository is being bootstrapped. There is no Rust crate or gallery yet.
The initial baseline installs Quest; M0 will add the workspace and its checks.

## Development tools

Install Nix with flakes enabled, then enter the pinned project shell:

```sh
nix develop
quest --version
quest init
quest context
quest check
```

Before the first commit, untracked flake files are invisible to Git-backed Nix
flakes. Use `nix develop path:.` while inspecting an uncommitted bootstrap.
The shell supplies Quest, without enabling hooks, runners, or deployment.
See [tool provenance](docs/toolchain.md) for the exact pin and platform limits.

## Changes

Use the official Quest lifecycle: read `quest guide`, plan one independently
completable outcome per quest, then claim approved work with `quest start`.
An initial empty main baseline must exist before claiming work. Planning changes
use their own PR; milestone numbers describe this library only.

A component PR includes implementation, runnable examples, documented ownership,
and meaningful interaction checks. Keep transient UI state isolated by stable
IDs. Preserve third-party notices when reusing code or assets.

Use additional commits and merge base updates into pushed branches. Before
marking a PR ready, run the repository checks and obtain the independent review
described by Quest for the exact committed head. A review does not merge a PR.

PR CI uses read-only permissions. Deployment and release are separate workflows;
this bootstrap contains neither. Keep private notes and raw review logs out of
public changes.

## Current checks

```sh
quest check
```

M0 repository-foundation will introduce locked Rust formatting, lint, test,
documentation, native build, and wasm build commands. Do not report them as
passing until that workspace exists and the commands have run.
