use super::*;
fn blocking() -> EventFilter {
    EventFilter {
        escape: true,
        ..Default::default()
    }
}
fn begin(m: &mut Memory, viewport: ViewportId, live: &[ViewportId]) {
    m.begin_pass(
        &RawInput {
            viewport_id: viewport,
            ..Default::default()
        },
        &live.iter().copied().collect(),
    );
}
fn register(m: &mut Memory, id: Id) {
    m.register_focus_policy(id, LayerId::background(), blocking());
}
fn used(id: Id) -> IdMap<Rect> {
    core::iter::once((id, Rect::from_min_size(Pos2::ZERO, Vec2::splat(10.)))).collect()
}
#[test]
fn policy_expiration_clears_cached_and_active_filter() {
    let mut m = Memory::default();
    let a = Id::unique("a");
    begin(&mut m, ViewportId::ROOT, &[ViewportId::ROOT]);
    register(&mut m, a);
    m.request_focus(a);
    m.end_pass(&used(a));
    begin(&mut m, ViewportId::ROOT, &[ViewportId::ROOT]);
    m.end_pass(&used(a));
    assert!(
        m.focus()
            .expect("begin creates viewport focus state")
            .policies
            .is_empty()
    );
    assert!(
        !m.focus()
            .expect("begin creates viewport focus state")
            .focused_widget
            .expect("the test focused this widget")
            .filter
            .escape
    );
    assert_eq!(m.focused(), Some(a));
}
#[test]
fn policy_expiration_respects_explicit_current_filter() {
    let mut m = Memory::default();
    let a = Id::unique("a");
    begin(&mut m, ViewportId::ROOT, &[ViewportId::ROOT]);
    register(&mut m, a);
    m.request_focus(a);
    m.end_pass(&used(a));
    begin(&mut m, ViewportId::ROOT, &[ViewportId::ROOT]);
    m.set_focus_lock_filter(a, blocking());
    m.end_pass(&used(a));
    assert!(
        m.focus()
            .expect("begin creates viewport focus state")
            .policies
            .is_empty()
    );
    assert!(
        m.focus()
            .expect("begin creates viewport focus state")
            .focused_widget
            .expect("the test focused this widget")
            .filter
            .escape
    );
}
#[test]
fn ignored_widget_policy_is_removed_without_focus_reclaim() {
    let mut m = Memory::default();
    let a = Id::unique("a");
    begin(&mut m, ViewportId::ROOT, &[ViewportId::ROOT]);
    register(&mut m, a);
    m.request_focus(a);
    m.ignore_focus(a);
    assert!(
        m.focus()
            .expect("begin creates viewport focus state")
            .policies
            .is_empty()
    );
    assert_eq!(m.focused(), None);
}
#[test]
fn unused_registrations_are_pruned() {
    let mut m = Memory::default();
    begin(&mut m, ViewportId::ROOT, &[ViewportId::ROOT]);
    for n in 0..16 {
        register(&mut m, Id::unique(n));
    }
    m.end_pass(&used(Id::unique(0)));
    assert_eq!(
        m.focus()
            .expect("begin creates viewport focus state")
            .policies
            .len(),
        1
    );
}
#[test]
fn closed_viewport_policy_and_focus_are_removed() {
    let mut m = Memory::default();
    let root = ViewportId::ROOT;
    let child = ViewportId::from_hash_of("child");
    let a = Id::unique("same-id");
    begin(&mut m, root, &[root, child]);
    register(&mut m, a);
    m.end_pass(&used(a));
    begin(&mut m, child, &[root, child]);
    register(&mut m, a);
    m.request_focus(a);
    m.end_pass(&used(a));
    assert_eq!(m.focus.len(), 2);
    begin(&mut m, root, &[root]);
    assert_eq!(m.focus.len(), 1);
    assert!(!m.focus.contains_key(&child));
    assert!(
        m.focus()
            .expect("begin creates viewport focus state")
            .policies
            .contains_key(&a)
    );
}
