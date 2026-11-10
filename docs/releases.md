# Release readiness

The project is in development and is not yet release-ready. A release is useful
when a documented scope works reliably for an external consumer; component count
or merged PR count alone does not establish that.

## Component scope

M1 targets Button, IconButton, TextInput, Checkbox, Switch, Slider, Select and
Dialog. These eight controls cover actions, settings, parameter editing and basic
forms. Button and IconButton share one implementation Quest. Each component owns
its API, caller-owned values, stable IDs, pinned shadcn appearance, states,
interaction, examples and relevant tests. The native Gallery validates their
combined use on macOS, Windows and Linux.

All eight controls now have merged component implementations and documented
evidence. Shared parity closure, integrated native Gallery acceptance, the native
accessibility bridge,
the complete platform evidence matrix, the standalone font test contract and a
clean external consumer remain open preparation work.
Adding the rest of the shadcn catalog is not a first-release prerequisite. Native
desktop remains the target; browser delivery and Pages have separate deferred
Quests. The pinned shadcn browser fixture remains a comparison source.

## Channels and versions

| Outcome | What it provides | Readiness boundary |
| --- | --- | --- |
| GitHub source preview | A tagged source snapshot, build instructions and reviewed release notes; validated Gallery binaries may be attached | Accepted advertised functionality, a clean external source consumer, complete notices and reproducible assets |
| Regular 0.1 release | The completed M1 scope with a clear API, installation path and platform support matrix | Eight accepted controls, integrated native Gallery, matching platform evidence and migration instructions |
| Rust registry packages | Core and compositor packages consumed through Cargo | A published dependency route, package contents and external-consumer checks; separate from a source release |
| 1.0 | A public API with a compatibility commitment | Stable ownership, host integration and migration policy, supported by representative consumer use |

An alpha such as 0.1.0-alpha.1 is a possible first preview, not a chosen version or
permission to publish. Pre-release status permits an explicitly evolving API; it
does not excuse known broken advertised behavior. M1 preparation retains all eight
controls and three desktop platforms. Any narrower initial support statement
must be proposed explicitly with its evidence and approved before publication.
[Semantic Versioning](https://semver.org/spec/v2.0.0.html) distinguishes initial
development, pre-release identifiers and the public API commitment at 1.0.

[GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases)
provide versioned source and optional downloadable assets. Existing debug CI
artifacts are development evidence, not validated release packages. Source-only
delivery need not invent an installer or signing infrastructure. When binaries
are offered, document architectures, runtime requirements and signing status,
and verify their extracted/downloaded payloads on the promised targets.

[GitHub Packages](https://docs.github.com/en/packages/learn-github-packages/introduction-to-github-packages)
does not currently list Cargo among its supported registries. An empty Packages
section is not a release defect. Rust registry distribution is planned separately
for an appropriate Cargo registry such as crates.io.

## Consumer and dependency gate

The controlled egui input integration requires a consistent dependency source
across the application, Prism and the official host crates. Cargo reads
[dependency patches](https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html)
from the consumer workspace root, not from a dependency's manifest. A successful
in-workspace build does not prove that an external cargo add works.

For source distribution, test an extracted fixed snapshot with a consumer outside
the workspace and the documented root configuration. For registry distribution,
the published package graph and documentation build must resolve the needed APIs
without unpublished workspace paths. Verify package metadata, versioned package
relationships, included fonts and notices, the compiler baseline and package
contents using the [Cargo publishing workflow](https://doc.rust-lang.org/cargo/reference/publishing.html).
Changes to public dependency identity or a maintained public fork need a concrete
design decision before implementation. A source preview does not require that
registry decision to be resolved first.

## Evidence and approval

A candidate records its exact commit, proposed version, dependency provenance,
asset names and hashes, release notes and support matrix. Applicable repository
checks, independent review, GitHub code/security review and CI cover that commit.
An implementation change invalidates evidence for behavior it changes; refresh
the relevant checks before presenting the candidate again.

Keep build/CI, controlled GPU tests, actual GUI interaction, IME/clipboard and
assistive technology evidence distinct. Record the OS, architecture, display
protocol where relevant, source identity and observed cases. A missing platform
observation remains a gap; a successful build cannot fill it. Native-gallery owns
the integrated platform acceptance, and source-preview preparation verifies
packaging and installation against that evidence.

The maintainer reviews the completed version, exact commit, channel, assets and
support statement before publication. Source tags/releases, registry uploads
and first Pages deployment are separate decisions. Preparation creates no public
release or package. After an approved publication, verify the actual downloads
and consumer commands before reporting completion.

## Milestones and task ownership

| Milestone | Outcome |
| --- | --- |
| M0: Engineering, theme, and acceptance foundations | Reproducible tools, themes and the pinned design/acceptance reference |
| M1: Core components and native gallery | Eight controls and integrated native Gallery; source-candidate and first-publication planning retain separate evidence and approval |
| M2: Complex components and platform completeness | Independently planned complex components, registry preparation and separately gated deferred platform delivery |

The repository Quest tree describes public, reusable implementation and release
preparation work. Detailed execution schedules, consumer-specific analysis and
operation records remain local. GitHub milestones contain stage rules only,
with no detailed work or acceptance checklist.
No delivery date is inferred from a milestone name. Component corrections stay
in their current Quests; release preparation and external publication are separate
because they have different completion evidence and approval boundaries.
