# egui-prism

[English](README.md) | [简体中文](README.zh-CN.md)

A Rust component library implementing shadcn/ui designs with egui.

[egui](https://github.com/emilk/egui) is an immediate-mode GUI library written in
Rust for native and web applications. [shadcn/ui](https://ui.shadcn.com/docs)
defines the component appearance, geometry, states, and interaction contract.
egui-prism implements that contract in Rust, with native macOS, Windows and Linux as the primary targets.

The library workspace and native engineering gallery remain in development.
The workspace provides semantic light/dark themes. All eight core controls,
Button, IconButton, TextInput, Checkbox, Switch, Slider, Select, and Dialog,
are delivered at component level. The gallery distinguishes Prism components
from standard egui harness controls; other widgets are planned.
There is no published crate or live component gallery yet.

Browser deployment is deferred and does not block native component acceptance.

Development will use [Quest](https://github.com/moq-dev/quest): M0 establishes the engineering and design foundations, M1 delivers core components and the first gallery, and M2 extends components and platform validation.

## License

Original project code is licensed under MIT. Third-party code and assets retain their respective notices and licenses; see [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

## Try the engineering gallery

With Nix and flakes enabled:

```sh
nix develop --command cargo run --locked -p egui-prism-gallery
```

For native builds and complete checks, see [CONTRIBUTING.md](CONTRIBUTING.md).
The [acceptance gallery](docs/gallery.md) documents the interaction, layout, scale,
and screenshot cases.
The core library depends on egui; the gallery owns its winit window and explicit wgpu rendering
loop, with egui-winit handling OS input, IME and clipboard. Applications can use the library with their own host.

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md) for the pinned Quest setup and current
checks, and [toolchain.md](docs/toolchain.md) for provenance and platform limits.
The versioned [quest tree](quest/README.md) is the source of planned work.

See [semantic themes](docs/themes.md) for the theme API, color pairs, spacing ownership, and validation limits.

See [buttons](docs/buttons.md) for roles, sizes, icons, and caller-owned actions.
See [text input](docs/text-input.md) for editing, validation, IDs, and IME evidence.

See [checkboxes](docs/checkboxes.md) for caller-owned boolean editing.
See [switches](docs/switches.md) for controlled boolean changes and interaction.

See [sliders](docs/sliders.md) for range, step, geometry, and caller-owned values.
See [selects](docs/selects.md) for controlled option values, frame-local registration,
ordered popup input, and the native Gallery cases.
See [dialogs](docs/dialogs.md) for controlled modal state, neutral dismissal,
scoped content and explicit drawing groups.

## shadcn parity

Every component targets strict parity with the pinned shadcn source in appearance,
geometry, state styling and interaction. egui is the rendering/input substrate.
Component delivery does not establish acceptance on every target platform.
Automated tests and CI do not establish visual acceptance;
native platform, IME and assistive-technology gaps remain explicit. See the
[reference contract and current evidence index](docs/shadcn-parity.md)
and the [native Gallery acceptance work](quest/m1/native-gallery.md).

The opt-in [ordered input integration](docs/input-compatibility.md) requires an
application-root Cargo patch. Its [source inventory](vendor/README.md) and
standalone consumer checks keep the official platform/renderer types unified.

The optional [`egui-prism-wgpu`](docs/compositing.md) host adapter provides isolated
group opacity with the official wgpu renderer. The core remains egui-only; hosts own
renderer initialization, texture lifetimes and explicit shutdown. The Gallery
includes a dedicated final-framebuffer conformance fixture.
