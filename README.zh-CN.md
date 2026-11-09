# egui-prism

[English](README.md) | [简体中文](README.zh-CN.md)

一个基于 egui、在 Rust 中实现 shadcn/ui 设计的组件库。

[egui](https://github.com/emilk/egui) 是用 Rust 编写的即时模式 GUI 库，支持原生应用和网页应用。[shadcn/ui](https://ui.shadcn.com/docs) 提供组件的外观、几何尺寸、状态样式和交互规范。egui-prism 在 Rust 中实现这些规范，主要面向 macOS、Windows 和 Linux 原生桌面应用。

组件库与原生工程展示程序（Gallery）仍在开发中。语义化浅色／深色主题，以及 Button、IconButton、TextInput、Checkbox、Switch、Slider 和 Select 已有初步实现，正在按固定版本的 shadcn 参考逐项校正。Gallery 会区分 Prism 组件和用于验收操作的标准 egui 控件；其他组件仍在规划中。目前尚未发布 crate，也没有已上线的组件展示网站。

浏览器部署暂缓，不作为原生组件验收的前置条件。

开发采用 [Quest](https://github.com/moq-dev/quest) 工作流：M0 建立工程与设计基础，M1 实现核心组件和首版 Gallery，M2 扩展组件并完善平台验证。

## 许可证

本项目原创代码采用 MIT 许可证。第三方代码和资源保留各自的版权声明及许可证，详见 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)。

## 运行工程展示程序

安装 Nix 并启用 flakes 后运行：

```sh
nix develop --command cargo run --locked -p egui-prism-gallery
```

原生构建与完整检查步骤见 [CONTRIBUTING.md](CONTRIBUTING.md)。[Gallery 验收说明](docs/gallery.md) 列出了交互、布局、缩放和截图场景。

核心库依赖 egui；Gallery 管理自己的 winit 窗口和显式 wgpu 渲染循环，由 egui-winit 对接操作系统输入、输入法和剪贴板。应用也可以在自己的宿主程序中使用组件库。

## 开发

[CONTRIBUTING.md](CONTRIBUTING.md) 说明了固定版本的 Quest 环境与当前检查流程；[工具链说明](docs/toolchain.md) 记录了依赖来源和平台限制。仓库中的 [Quest 任务树](quest/README.md) 是开发计划的正式来源。

组件与集成文档目前使用英文：

- [语义化主题](docs/themes.md)：主题 API、颜色配对、间距职责和验证范围。
- [按钮](docs/buttons.md)：用途、尺寸、图标和由调用方管理的操作。
- [文本输入](docs/text-input.md)：编辑、校验、稳定 ID 和输入法验证证据。
- [复选框](docs/checkboxes.md)：编辑由调用方持有的布尔值。
- [开关](docs/switches.md)：受控布尔值变更与交互。
- [滑块](docs/sliders.md)：范围、步长、几何尺寸和由调用方持有的数值。
- [选择框](docs/selects.md)：受控选项值、帧内注册、有序弹层输入和原生 Gallery 场景。

## 与 shadcn 保持一致

所有组件都以固定版本的 shadcn 源码为基准，严格对齐外观、几何尺寸、状态样式和交互行为；egui 提供底层绘制与输入能力。现有组件正在逐项校正，自动化测试和 CI 通过本身不代表视觉验收完成。详见[参考规范与组件核对记录](docs/shadcn-parity.md)及[校正任务](quest/m1/shadcn-parity/README.md)。

可选的[有序输入集成](docs/input-compatibility.md)要求在应用根目录的 Cargo 配置中声明补丁。[源码清单](vendor/README.md)和独立消费者检查用于保证官方平台集成与渲染器使用统一的类型。

可选的 [`egui-prism-wgpu`](docs/compositing.md) 宿主适配包配合官方 wgpu 渲染器，为组件提供独立的整体透明度合成。核心库仍只依赖 egui；宿主负责渲染器初始化、纹理生命周期和显式关闭。Gallery 包含专门的最终帧缓冲一致性验收场景。
