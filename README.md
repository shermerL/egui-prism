# egui-prism

A shadcn-inspired component library for egui.

[egui](https://github.com/emilk/egui) is an immediate-mode GUI library written in
Rust for native and web applications. [shadcn/ui](https://ui.shadcn.com/docs)
provides the design reference for composable components and carefully chosen
default styles. egui-prism aims to bring reusable widgets, semantic themes, and
consistent interactions to egui, with one Rust implementation shared by native
and WebAssembly galleries.

This independent project is in its planning and bootstrap stage. There is no published crate or live component gallery yet.

The planned GitHub Pages site will display the actual components, their states, and Rust examples.

Development will use [Quest](https://github.com/moq-dev/quest): M0 establishes the engineering and design foundations, M1 delivers core components and the first gallery, and M2 extends components and platform validation.

## License

Original project code is licensed under MIT. Third-party code and assets retain their respective notices and licenses; see [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md) for the pinned Quest setup and current
checks, and [toolchain.md](docs/toolchain.md) for provenance and platform limits.
The versioned [quest tree](quest/README.md) is the source of planned work.
