# [M] Empty state targeted test and native acceptance coverage

## Goal

Close the specifically missing automated, pinned-reference, native interaction and accessibility evidence for [PR #35](https://github.com/shermerL/egui-prism/pull/35), with reproducible fixtures and revision-bound evidence. This does not waive the original component Quest, authorize merging, or claim that a test plan is a passing result.

## Plan

Use the [fixed parity contract](/docs/shadcn-parity.md). The implementation and its known correctness gaps stay owned by /quest/m2/native-components/empty.md; coordinate fixes there rather than duplicate production changes in the acceptance PR. Add tests and evidence on the actual candidate, retaining a before-fix failing case for each defect. A separately reviewable test/evidence PR may wait for the fix without claiming acceptance.

The [shared acceptance protocol](/docs/component-acceptance.md) defines source identity, environment, oracle, evidence rows, integration invalidation and exit gates. Every result must name the tested head and distinguish tests, rendered browser/native observations, and AT. Use the unmodified pinned reference and matched font/theme/scale. Do not replace a required observation with a successful CI job.

1. viewport 767/768/769 logical pt × container 140/320/宽屏，固定宽容器横跨断点时 root padding应24/48，由viewport而非container决定；实际150% zoom重新记录viewport，不能只做 CSS transform。light/dark、icon有无、空文本/省略section、full/long copy，header/content最大384pt、icon40/24pt与自然高度。
2. 文本比较必须包含 `office office office`、NBSP、长词、CJK/标点，以及原生会超过浏览器 balancing 行数限制的长段。每个案例记录 line count/每行文本/宽度，零 whitespace-only行、零截掉的合法内容。修复 deterministic balancing 与 CSS 的实测差异，不能把2304几何探针当 browser parity。
3. 真窗口演示 empty→loading→failed→empty，重复20次；click/keyboard后状态只由caller改变，消失button立即退出input/AT树，旧焦点/ID不激活隐藏动作，无额外网络、timer或业务状态。disabled动作0次。Page切换与focus loss/reentry再执行一轮。
4. 原生 AT 中 empty copy保持 passive，既无 fabricated alert/busy/live语义，也不抢焦点；caller明确的loading/error呈现分别检查，不能要求Empty自带播报。parent opacity显式group和direct painting边界要按文档分别验，不声称捕获任意GPU callback。
5. CSS balancing/Unicode、stale action、错误断点为合并前阻塞；不同模型独立性未证仍记录 review gate。

Before completion, include the automatable supported-case assertions in CI, provide native/reference captures and measured comparisons, record AT/input observations, link the applicable exact-head CI and independent review, and map every original requirement to PASS or a specifically retained blocking owner. FAIL, NOT RUN and BLOCKED rows prevent this Quest being called complete. No silent scope reduction or after-merge deferral of known parity defects. Only an explicit maintainer scope decision can permit staged landing; record the deferred requirements and owners without marking them PASS or deleting incomplete Quests.

Refresh evidence after affected implementation, reference, host or integration changes. Preserve original implementation attribution and test/evidence attribution separately. Written by dot (AI).

## Waiting

- Validation executor: provide an accessible native desktop with the pinned browser reference and relevant screen reader; the current headless environment cannot supply GUI or speech observations. Capture OS/backend/window-system, browser/AT versions, fonts, scale, source SHA and raw artifacts.
- Maintainer: authorize any Codex or GitHub automated-review task separately if required. No task launch or review request is implied by this Quest; the assistant must never merge a PR.

## Related

- [Original component](/quest/m2/native-components/empty.md) - owns implementation correctness and retains its unresolved acceptance obligations
- [Native Gallery](/quest/m1/native-gallery.md) - owns broader cross-platform host and cross-component acceptance, not a substitute for these component checks
- [Native accessibility](/quest/m1/native-accessibility.md) - owns the OS bridge needed for real assistive-technology observations
