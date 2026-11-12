# [M] Card targeted test and native acceptance coverage

## Goal

Close the specifically missing automated, pinned-reference, native interaction and accessibility evidence for [PR #34](https://github.com/shermerL/egui-prism/pull/34), with reproducible fixtures and revision-bound evidence. This does not waive the original component Quest, authorize merging, or claim that a test plan is a passing result.

## Plan

Use the [fixed parity contract](/docs/shadcn-parity.md). The implementation and its known correctness gaps stay owned by /quest/m2/native-components/card.md; coordinate fixes there rather than duplicate production changes in the acceptance PR. Add tests and evidence on the actual candidate, retaining a before-fix failing case for each defect. A separately reviewable test/evidence PR may wait for the fix without claiming acceptance.

The [shared acceptance protocol](/docs/component-acceptance.md) defines source identity, environment, oracle, evidence rows, integration invalidation and exit gates. Every result must name the tested head and distinguish tests, rendered browser/native observations, and AT. Use the unmodified pinned reference and matched font/theme/scale. Do not replace a required observation with a successful CI job.

1. 用同一长词/Unicode corpus 验证 header wrapping；空/短/长 section，省略 description，overflow content，短/混合/小数 footer height、accent glyph。测 header/content/footer坐标、24pt padding/gap、14pt radius，overflow 不扩后续 section，closure只运行一次。保留 15 项回归，新增 CSS oracle 对照而非自我比较。
2. 重点 GPU+真实视觉：双层 shadow-sm 在 light/dark、浅/深/tinted背景和60% group opacity；parent clip 和 rounded border内排除；与 Button shadow-xs并排确认共用 helper 未回归。两层几何存在不等于 blur 像素接近参考。
3. 原生 input：nested Save Button 与 editor，Tab/Shift-Tab、Enter/Space/click、disabled、重复渲染、page切换回来、focus loss/reentry。每次有效输入一次动作，禁用零次，editor值不因 card绘制/切页丢失，hit bounds与视觉一致；无 shell click/focus 冒充 child。AT 不新增交互容器，child label/order可用。
4. 标准 egui editor不是 Prism TextInput，不要求将其外观误验为 shadcn输入。CardAction slot不是已实现公共能力，不把其无关扩展混进本验收。header实现缺口、shadow明显差异、输入错位为合并前阻塞。

Before completion, include the automatable supported-case assertions in CI, provide native/reference captures and measured comparisons, record AT/input observations, link the applicable exact-head CI and independent review, and map every original requirement to PASS or a specifically retained blocking owner. FAIL, NOT RUN and BLOCKED rows prevent this Quest being called complete. No silent scope reduction or after-merge deferral of known parity defects. Only an explicit maintainer scope decision can permit staged landing; record the deferred requirements and owners without marking them PASS or deleting incomplete Quests.

Refresh evidence after affected implementation, reference, host or integration changes. Preserve original implementation attribution and test/evidence attribution separately. Written by dot (AI).

## Waiting

- Validation executor: provide an accessible native desktop with the pinned browser reference and relevant screen reader; the current headless environment cannot supply GUI or speech observations. Capture OS/backend/window-system, browser/AT versions, fonts, scale, source SHA and raw artifacts.
- Maintainer: authorize any Codex or GitHub automated-review task separately if required. No task launch or review request is implied by this Quest; the assistant must never merge a PR.

## Related

- [Original component](/quest/m2/native-components/card.md) - owns implementation correctness and retains its unresolved acceptance obligations
- [Native Gallery](/quest/m1/native-gallery.md) - owns broader cross-platform host and cross-component acceptance, not a substitute for these component checks
- [Native accessibility](/quest/m1/native-accessibility.md) - owns the OS bridge needed for real assistive-technology observations
