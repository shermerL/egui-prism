# Component acceptance protocol

Written by dot (AI).

This is an acceptance specification, not evidence of completed validation.

### 证据层分开

A. 单元/布局/metadata/合成输入；B. GPU framebuffer/compositor；C. 实际浏览器参考；D. 原生窗口、真实输入和渲染；E. 原生辅助技术；F. 多组件集成/跨平台。每层分别标 `PASS / FAIL / NOT RUN / BLOCKED / N/A`。N/A 必须写合同依据，不能用“没环境”替代。CI、截图、metadata 各自不能代替其他层。

### 执行环境与统一样本

- 记录 git head、base、树、参考 manifest/hash、OS/version、GPU/backend、窗口系统、浏览器/version、AT/version、字体文件/字重、viewport/container 的逻辑与物理尺寸、DPR、OS scale、应用 zoom。固定 shadcn `8a3536678a528cc012b9e62484290e79d0be410f`，真实未修改的参考组件，Geist 字体。中文缺字应明确 FAIL/字体阻塞，不能以字符串存在冒充渲染正常。
- 两端可访问且真实渲染：在同一验收桌面构建启动 Gallery 与 reference。先验证实际 URL，再比较。禁止用自行画的 HTML mock 代替固定参考；禁止把软件 Vulkan 无窗口测试称为 Linux 桌面通过。
- 每个关键样本覆盖 light/dark × 100%/150%。100% 是 1 CSS px 对应 1 egui logical point；150% 记录实际 DPR/缩放，不以 CSS zoom 伪装 viewport media-query 行为。不同字体 rasterization 不追求虚构的全图零差异。
- 每个样本留 native/reference 原图、同尺寸并排/差异图、测量表和操作记录。几何使用逻辑点：合同中的 padding/gap/border/radius 与实现计算值必须一致；截屏边界容许的单物理像素栅格误差必须逐项解释，不允许以阈值豁免错误行数、缺字、截断、错位或明显阴影差异。文本行数、词边界、完整内容必须匹配支持范围的参考；视觉差异未解释且未处理即 FAIL。
- 通用代码入口：`nix develop --command quest gate python3 scripts/check.py`；平台受限可拆分 `--cargo-only` 与 policy，但分别报告。定向命令 `cargo test --locked -p egui-prism --test COMPONENT` 和 `cargo run --locked -p egui-prism --example COMPONENT`。示例使用 recording compositor，不是 GUI。
- 新增失败样本先复现，再修复并进 CI；慢像素/多平台任务可按仓库约定安排，不能仅留本地未运行脚本。不得增加新依赖/host/vendor 边界来偷换本次范围，必要时单独决策。

### 合并与完成边界

**合并前必须**：已知支持范围内 CSS/行为偏差解决；新回归加入 CI；实际 pinned browser 与至少明确记录的平台 native 对照完成；组件相关语义/交互实机检查完成；冲突解决后的精确 head full checks + affected review；所有验收行有证据或明确阻塞。普通“代码可以合并”不等于 Quest 完成。

**可保留在现有 M1 的工作**：尚未完成的其他 OS、Linux X11/Wayland 组合、通用 host 生命周期/IME/跨组件场景，仅当其确实不属于当前组件修复且有明确 M1 路径、负责人和复验触发条件。这不是自动豁免：若用户/维护者决定阶段性落地，必须公开剩余范围，不删除未完成 Quest、不宣称全平台/完整 parity。默认仍保持 draft。严格原合同并未授予代理自行降低验收标准。

**合并后才可执行的验证**：由维护者实际合并后，在 main 的最终合并 SHA 做安装/运行 smoke，确认 Gallery 页、导出、reference route/hash 和证据链接未丢，运行受影响测试。此项是集成确认，不替代合并前必需测试。代理永不自行合并、开 auto-merge 或 merge queue。

### 防遗漏及 head 失效机制

每行验收记录：`ID | 原 Quest 条款 | 类型 | 样本/步骤 | 期望 | 实际 | 状态 | head+base | 环境 | 原始证据链接 | 执行者 | 复核者 | 阻塞/归属`。

PR 描述 Waiting、Quest Waiting 与组件 evidence 文档必须能互相追溯。关闭前逐行核对，不能以“写了跟进任务”标 PASS。每次代码/fixture/base 合入变化先生成影响列表，重跑相关样本；未受影响证据可复用但写明旧 head、树/路径未变依据。新增提交后旧 review/CI 不自动算新 head 通过。本地 merge main 解决冲突，不 rebase/force push；不得抢改最近一小时其他人更新的 PR 分支。

四条独立分支共享 lib exports、Gallery navigation/input、reference routes/manifest、docs evidence index；#33/#35 涉及 Button shared helper，#34/#35 涉及 Gallery input。依次合入最终候选时确认四组件、既有 Tooltip/Badge 全部可到达，保持各自 source hashes，重跑受影响控件和整个检查入口。零遗漏 exports/routes、零重复注册/激活、零新增 lint/test 失败；未应用的其他 PR 不可要求当前分支提供其组件。

合并/审查权限独立于验收：记录 GitHub code/security review 缺口，不擅自启动耗额度任务。取得独立 reviewer 非实现者证据；无法证明不同模型则写 BLOCKED，不推断。
