# egui-prism

A shadcn-inspired component library for egui.

[egui](https://github.com/emilk/egui) is an immediate-mode GUI library written in
Rust for native and web applications. [shadcn/ui](https://ui.shadcn.com/docs)
provides the design reference for composable components and carefully chosen
default styles. egui-prism aims to bring reusable widgets, semantic themes, and
consistent interactions to egui, with one Rust implementation shared by native
and WebAssembly galleries.

The library workspace and native/WebAssembly engineering gallery are in development. Semantic light/dark themes, Button, IconButton, TextInput, and Checkbox are available. The gallery distinguishes Prism components from standard egui harness controls; other widgets are planned. There is no published crate or live component gallery yet.

The planned GitHub Pages site will display the actual components, their states, and Rust examples.

Development will use [Quest](https://github.com/moq-dev/quest): M0 establishes the engineering and design foundations, M1 delivers core components and the first gallery, and M2 extends components and platform validation.

## License

Original project code is licensed under MIT. Third-party code and assets retain their respective notices and licenses; see [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

## Try the engineering gallery

With Nix and flakes enabled:

```sh
nix develop --command cargo run --locked -p egui-prism-gallery
```

For the browser build and complete checks, see [CONTRIBUTING.md](CONTRIBUTING.md).
The [acceptance gallery](docs/gallery.md) documents the interaction, layout, scale,
and screenshot cases.
The core library depends on egui; the gallery owns the eframe window and rendering
backend. Applications can use the library with their own host.

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md) for the pinned Quest setup and current
checks, and [toolchain.md](docs/toolchain.md) for provenance and platform limits.
The versioned [quest tree](quest/README.md) is the source of planned work.

See [semantic themes](docs/themes.md) for the theme API, color pairs, spacing ownership, and validation limits.

See [buttons](docs/buttons.md) for roles, sizes, icons, and caller-owned actions.
See [text input](docs/text-input.md) for editing, validation, IDs, and IME evidence.

See [checkboxes](docs/checkboxes.md) for caller-owned boolean editing.

## shadcn parity

Every component targets strict parity with the pinned shadcn source in appearance,
geometry, state styling and interaction. egui is the rendering/input substrate.
The current components are undergoing a full parity correction; existing test and
CI results do not establish visual acceptance. See the [reference contract and
component audit](docs/shadcn-parity.md) and [correction Quests](quest/m1/shadcn-parity/README.md).
