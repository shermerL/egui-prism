# [M] Inline Alert targeted test and native acceptance coverage

## Goal

Close the specifically missing automated, pinned-reference, native interaction and accessibility evidence for [PR #33](https://github.com/shermerL/egui-prism/pull/33), with reproducible fixtures and revision-bound evidence. This does not waive the original component Quest, authorize merging, or claim that a test plan is a passing result.

## Plan

Use the [fixed parity contract](/docs/shadcn-parity.md). The implementation and its known correctness gaps stay owned by /quest/m2/native-components/alert.md; coordinate fixes there rather than duplicate production changes in the acceptance PR. Add tests and evidence on the actual candidate, retaining a before-fix failing case for each defect. A separately reviewable test/evidence PR may wait for the fix without claiming acceptance.

The [shared acceptance protocol](/docs/component-acceptance.md) defines source identity, environment, oracle, evidence rows, integration invalidation and exit gates. Every result must name the tested head and distinguish tests, rendered browser/native observations, and AT. Use the unmodified pinned reference and matched font/theme/scale. Do not replace a required observation with a successful CI job.

1. 补 reference-driven 断行矩阵：448/320/160pt 和小于 padding+icon 宽度；`alpha-beta/gamma`、URL/长无空格词、NBSP、中文标点/中英混排、带重音字符。先记录 browser min-content/行数/词边界及原生实测，再修复 hyphen/punctuation/CJK 支持范围；异常窄容器的“强制 clip”与 CSS overflow 不一致也必须记录、解决或经明确范围决策，不能因为已有文档就视为通过。
2. default/destructive，icon/title/description 有无组合，长 title ellipsis、完整 accessible label，light/dark与 opacity。与现有九项测试做映射，仅新增未覆盖断言。
3. 在原生 AT 打开后动态插入 notice、只变 description、只变 title、连续相同帧、移除再重建、两个独立 ID。预期每个真实内容改变有完整 title+body 通知，相同稳定帧无重复通知，焦点不被夺走，描述不漏读或重复。比较同环境 browser role=alert，记录 AT 合并/打断机制，不以精确毫秒要求异步语音。异常去查系统桥接/宿主，不改组件成自带播报队列。
4. 与旁边 caller Button 交互：真实 click/Enter/Space各一次只触发一次；disabled零次；notice 不自己 retry/dismiss。CSS缺口及遗漏通知为合并前阻塞。桥接 unavailable 记依赖既有 native-accessibility，不伪造通过。

Before completion, include the automatable supported-case assertions in CI, provide native/reference captures and measured comparisons, record AT/input observations, link the applicable exact-head CI and independent review, and map every original requirement to PASS or a specifically retained blocking owner. FAIL, NOT RUN and BLOCKED rows prevent this Quest being called complete. No silent scope reduction or after-merge deferral of known parity defects. Only an explicit maintainer scope decision can permit staged landing; record the deferred requirements and owners without marking them PASS or deleting incomplete Quests.

Refresh evidence after affected implementation, reference, host or integration changes. Preserve original implementation attribution and test/evidence attribution separately. Written by dot (AI).

## Waiting

- Validation executor: provide an accessible native desktop with the pinned browser reference and relevant screen reader; the current headless environment cannot supply GUI or speech observations. Capture OS/backend/window-system, browser/AT versions, fonts, scale, source SHA and raw artifacts.
- Maintainer: authorize any Codex or GitHub automated-review task separately if required. No task launch or review request is implied by this Quest; the assistant must never merge a PR.

## Related

- [Original component](/quest/m2/native-components/alert.md) - owns implementation correctness and retains its unresolved acceptance obligations
- [Native Gallery](/quest/m1/native-gallery.md) - owns broader cross-platform host and cross-component acceptance, not a substitute for these component checks
- [Native accessibility](/quest/m1/native-accessibility.md) - owns the OS bridge needed for real assistive-technology observations
