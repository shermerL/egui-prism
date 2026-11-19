# [S] Expose and verify the Dialog header in the macOS accessibility tree

## Goal

A macOS accessibility client can identify an open Prism Dialog by its title,
read its optional description and reach its actions. Closing the Dialog removes
its header from the system accessibility tree.

## Plan

[Issue #29](https://github.com/shermerL/egui-prism/issues/29) records the original
observation on macOS 26.6.2 with Apple M3 and Metal. In Gallery build
`5401239`, a system AX client could press Close, Apply and Done but could not
find the painted title or description. The unmerged [candidate from PR #46](https://github.com/shermerL/egui-prism/commit/19e0a298bc0ae8bcd35f0f881236af79440de1c2)
publishes the heading and optional description on the component's modal node.
All 23 Dialog tests passed on that candidate. The new regression checks the
heading identity, name and description relations, optional text and removal on
close. The Gallery ran on Metal. System verification remains pending because
the AX client lacked accessibility permission.

1. When work resumes, compare the candidate with the current Dialog implementation.
   Build the revision to verify in the pinned development shell with
   `cargo run --locked -p egui-prism-gallery`.
2. With a trusted macOS accessibility client, enter **Dialogs** and open
   **Dialog A**. Inspect the dialog's accessible name, its heading and the
   description “Actions belong to the application.” Verify that Close, Apply
   and Done are reachable as separate buttons.
3. Press Close and verify that the dialog's heading disappears. Record the
   revision, OS, client, and observed names, roles and dismissal in
   [the Gallery observation record](/docs/gallery.md). If a field or action is
   missing, keep the Quest open with the actual observation and repair the component.
4. Complete the Quest once the native check passes and the verified fix is
   integrated into main. The [native bridge acceptance Quest](/quest/m1/native-accessibility.md)
   tracks VoiceOver navigation, focus restoration and the other platforms.

## Waiting

- Maintainer: explicitly resume the frozen project before implementation or validation.
- Validation owner: inspect the retained candidate with a trusted macOS accessibility client.
