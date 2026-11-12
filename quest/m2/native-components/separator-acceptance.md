# [S] Separator targeted test and native acceptance coverage

## Goal

Close the specifically missing automated, pinned-reference, native interaction and accessibility evidence for [PR #32](https://github.com/shermerL/egui-prism/pull/32), with reproducible fixtures and revision-bound evidence. This does not waive the original component Quest, authorize merging, or claim that a test plan is a passing result.

## Plan

Use the [fixed parity contract](/docs/shadcn-parity.md). The implementation and its known correctness gaps stay owned by /quest/m2/native-components/separator.md; coordinate fixes there rather than duplicate production changes in the acceptance PR. Add tests and evidence on the actual candidate, retaining a before-fix failing case for each defect. A separately reviewable test/evidence PR may wait for the fix without claiming acceptance.

The [shared acceptance protocol](/docs/component-acceptance.md) defines source identity, environment, oracle, evidence rows, integration invalidation and exit gates. Every result must name the tested head and distinguish tests, rendered browser/native observations, and AT. Use the unmodified pinned reference and matched font/theme/scale. Do not replace a required observation with a successful CI job.

1. 保留现有 7 个回归。native `Separators` 与 `?separators`：两方向 × decorative/semantic；普通和 justified parent；正常/极窄/零 extent；部分裁切；60% parent opacity tinted surface；100%/150%。水平/竖直线 1pt，extent 等于 caller 提供范围，hover/accessible bounds 不扩大，AA 不意外截断，无额外 margin。
2. 对比 framebuffer 的 fractional-scale 细线和裁切边缘，补当前只有几何断言的渲染证据。装饰线不入可交互 tab 序列；语义线暴露正确 role/orientation、无 activate/drag/focus。原生 AT 验证暴露和阅读行为，允许平台读法不同但不能把装饰线读成控件。
3. 合并前阻塞：匹配视觉和语义证据、`79b860a` 或更新 head 的有效独立集成 review。无已证新实现缺陷，不要预设修复代码。已通过的 macOS受控测试不重新描述成 GUI。

Before completion, include the automatable supported-case assertions in CI, provide native/reference captures and measured comparisons, record AT/input observations, link the applicable exact-head CI and independent review, and map every original requirement to PASS or a specifically retained blocking owner. FAIL, NOT RUN and BLOCKED rows prevent this Quest being called complete. No silent scope reduction or after-merge deferral of known parity defects. Only an explicit maintainer scope decision can permit staged landing; record the deferred requirements and owners without marking them PASS or deleting incomplete Quests.

Refresh evidence after affected implementation, reference, host or integration changes. Preserve original implementation attribution and test/evidence attribution separately. Written by dot (AI).

## Waiting

- Validation executor: provide an accessible native desktop with the pinned browser reference and relevant screen reader; the current headless environment cannot supply GUI or speech observations. Capture OS/backend/window-system, browser/AT versions, fonts, scale, source SHA and raw artifacts.
- Maintainer: authorize any Codex or GitHub automated-review task separately if required. No task launch or review request is implied by this Quest; the assistant must never merge a PR.

## Related

- [Original component](/quest/m2/native-components/separator.md) - owns implementation correctness and retains its unresolved acceptance obligations
- [Native Gallery](/quest/m1/native-gallery.md) - owns broader cross-platform host and cross-component acceptance, not a substitute for these component checks
- [Native accessibility](/quest/m1/native-accessibility.md) - owns the OS bridge needed for real assistive-technology observations
