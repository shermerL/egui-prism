//! Controlled-time geometry, scheduling, semantics and explicit submission.
use egui_prism::{
    compositing::{self, Group, Item},
    egui,
    spinner::{Spinner, SpinnerOutput},
    theme::Theme,
};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

fn frame(
    ctx: &egui::Context,
    time: f64,
    animate: bool,
    visible: bool,
    opacity: f32,
) -> (SpinnerOutput, egui::FullOutput) {
    let mut result = None;
    let full = ctx.run_ui(
        egui::RawInput {
            time: Some(time),
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(320.0, 200.0),
            )),
            ..Default::default()
        },
        |ui| {
            ui.set_opacity(opacity);
            if !visible {
                ui.set_clip_rect(egui::Rect::from_min_size(
                    egui::pos2(1000.0, 1000.0),
                    egui::vec2(10.0, 10.0),
                ));
            }
            result = Some(
                Spinner::new(&Theme::light(), "spinner", "Loading projects")
                    .animate(animate)
                    .show(ui),
            );
        },
    );
    (result.unwrap(), full)
}
fn points(output: &SpinnerOutput) -> Vec<egui::Pos2> {
    let Item::Shape(shape) = &output.drawing.as_ref().unwrap().items()[0] else {
        panic!("arc")
    };
    let egui::Shape::Mesh(mesh) = &shape.shape else {
        panic!("mesh")
    };
    (0..=64)
        .map(|i| {
            mesh.vertices[i * 4 + 1]
                .pos
                .lerp(mesh.vertices[i * 4 + 2].pos, 0.5)
        })
        .collect()
}

fn close(actual: egui::Vec2, expected: egui::Vec2) {
    assert!(
        (actual - expected).length() < 0.002,
        "{actual:?} != {expected:?}"
    );
}

#[test]
fn one_second_clockwise_motion_matches_svg_endpoints_at_controlled_times() {
    let ctx = egui::Context::default();
    for (time, expected) in [
        (10.0, egui::vec2(6.0, 0.0)),
        (10.25, egui::vec2(0.0, 6.0)),
        (10.5, egui::vec2(-6.0, 0.0)),
        (10.75, egui::vec2(0.0, -6.0)),
        (11.0, egui::vec2(6.0, 0.0)),
    ] {
        let (output, full) = frame(&ctx, time, true, true, 1.0);
        close(points(&output)[0] - output.response.rect.center(), expected);
        assert_eq!(output.response.rect.size(), egui::vec2(16.0, 16.0));
        if time == 10.0 {
            close(
                points(&output)[64] - output.response.rect.min,
                egui::vec2(14.781, 3.44) * (2.0 / 3.0),
            );
        }
        assert_eq!(
            full.viewport_output[&egui::ViewportId::ROOT].repaint_delay,
            Duration::ZERO
        );
        full.drop_without_applying_deltas();
    }
}

#[test]
fn inactive_clipped_and_zero_opacity_stop_scheduling_after_context_warmup() {
    for (animate, visible, opacity) in [(false, true, 1.0), (true, false, 1.0), (true, true, 0.0)] {
        let ctx = egui::Context::default();
        for i in 0..8 {
            let (output, full) = frame(&ctx, i as f64 * 0.1, animate, visible, opacity);
            if !visible {
                assert!(output.drawing.is_none());
            }
            if !animate {
                close(
                    points(&output)[0] - output.response.rect.center(),
                    egui::vec2(6.0, 0.0),
                );
            }
            if i > 3 {
                assert_ne!(
                    full.viewport_output[&egui::ViewportId::ROOT].repaint_delay,
                    Duration::ZERO
                );
            }
            full.drop_without_applying_deltas();
        }
    }
}

#[test]
fn omitted_frames_and_reenabling_motion_restart_but_clipping_does_not() {
    let ctx = egui::Context::default();
    frame(&ctx, 0.0, true, true, 1.0)
        .1
        .drop_without_applying_deltas();
    frame(&ctx, 0.1, true, false, 1.0)
        .1
        .drop_without_applying_deltas();
    let (output, full) = frame(&ctx, 0.25, true, true, 1.0);
    close(
        points(&output)[0] - output.response.rect.center(),
        egui::vec2(0.0, 6.0),
    );
    full.drop_without_applying_deltas();
    ctx.run_ui(
        egui::RawInput {
            time: Some(0.4),
            ..Default::default()
        },
        |_| {},
    )
    .drop_without_applying_deltas();
    let (output, full) = frame(&ctx, 0.5, true, true, 1.0);
    close(
        points(&output)[0] - output.response.rect.center(),
        egui::vec2(6.0, 0.0),
    );
    full.drop_without_applying_deltas();
    frame(&ctx, 0.6, false, true, 1.0)
        .1
        .drop_without_applying_deltas();
    let (output, full) = frame(&ctx, 0.75, true, true, 1.0);
    close(
        points(&output)[0] - output.response.rect.center(),
        egui::vec2(6.0, 0.0),
    );
    full.drop_without_applying_deltas();
    for time in [0.8, 0.9, 1.0, 1.1, 1.2, 1.3] {
        let full = ctx.run_ui(
            egui::RawInput {
                time: Some(time),
                ..Default::default()
            },
            |_| {},
        );
        if time >= 1.1 {
            assert_ne!(
                full.viewport_output[&egui::ViewportId::ROOT].repaint_delay,
                Duration::ZERO
            );
        }
        full.drop_without_applying_deltas();
    }
}

#[test]
fn instances_have_independent_epochs_and_discard_does_not_restart_motion() {
    let ctx = egui::Context::default();
    for time in [0.0, 0.25, 0.5] {
        let full = ctx.run_ui(
            egui::RawInput {
                time: Some(time),
                ..Default::default()
            },
            |ui| {
                let a = Spinner::new(&Theme::light(), "a", "A").show(ui);
                let rotation = egui::emath::Rot2::from_angle(time as f32 * std::f32::consts::TAU);
                close(
                    points(&a)[0] - a.response.rect.center(),
                    rotation * egui::vec2(6.0, 0.0),
                );
                if time >= 0.25 {
                    let b = Spinner::new(&Theme::light(), "b", "B").show(ui);
                    let rotation =
                        egui::emath::Rot2::from_angle((time - 0.25) as f32 * std::f32::consts::TAU);
                    close(
                        points(&b)[0] - b.response.rect.center(),
                        rotation * egui::vec2(6.0, 0.0),
                    );
                }
                if time == 0.5 {
                    ctx.request_discard("exercise same-frame relayout");
                }
            },
        );
        full.drop_without_applying_deltas();
    }
}

#[test]
fn pinned_size_colors_caps_and_clips_survive_scale_and_justified_layout() {
    for theme in [Theme::light(), Theme::dark()] {
        for scale in [1.0, 1.5] {
            let ctx = egui::Context::default();
            ctx.set_pixels_per_point(scale);
            ctx.run_ui(Default::default(), |ui| {
                ui.with_layout(
                    egui::Layout::top_down_justified(egui::Align::Center),
                    |ui| {
                        ui.set_width(200.0);
                        let clip = egui::Rect::from_min_size(
                            ui.cursor().min + egui::vec2(0.0, 4.0),
                            egui::vec2(200.0, 8.0),
                        );
                        ui.set_clip_rect(clip);
                        let output = Spinner::new(&theme, "s", "Loading").animate(false).show(ui);
                        assert_eq!(output.response.rect.size(), egui::vec2(16.0, 16.0));
                        assert!(!output.response.sense.is_focusable());
                        let group = output.drawing.unwrap();
                        assert!(clip.contains_rect(group.bounds()));
                        assert_eq!(group.opacity(), 1.0);
                        for item in group.items() {
                            let Item::Shape(shape) = item else {
                                panic!("shape")
                            };
                            assert!(clip.contains_rect(shape.clip_rect));
                            let egui::Shape::Mesh(mesh) = &shape.shape else {
                                panic!("mesh");
                            };
                            assert!(mesh.is_valid());
                            assert_eq!(mesh.vertices.len(), 330);
                            assert_eq!(mesh.indices.len(), 1440);
                            assert!(
                                mesh.vertices
                                    .iter()
                                    .any(|v| v.color == theme.colors.background.foreground)
                            );
                            assert!(
                                mesh.vertices
                                    .iter()
                                    .all(|v| v.color == theme.colors.background.foreground
                                        || v.color == egui::Color32::TRANSPARENT)
                            );
                        }
                    },
                );
            })
            .drop_without_applying_deltas();
        }
    }
}

#[test]
fn accessible_status_is_caller_named_polite_atomic_and_never_progress_or_focus() {
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    for time in [0.0, 0.25] {
        let (output, full) = frame(&ctx, time, true, true, 1.0);
        let update = full.platform_output.accesskit_update.as_ref().unwrap();
        let (_, node) = update
            .nodes
            .iter()
            .find(|(id, _)| *id == output.response.id.accesskit_id())
            .unwrap();
        assert_eq!(node.role(), egui::Role::Status);
        assert_eq!(node.label(), Some("Loading projects"));
        assert_eq!(node.live(), Some(egui::accesskit::Live::Polite));
        assert!(node.is_live_atomic());
        assert!(node.numeric_value().is_none());
        assert!(!node.supports_action(egui::accesskit::Action::Focus));
        assert!(!node.supports_action(egui::accesskit::Action::Click));
        full.drop_without_applying_deltas();
    }
}

#[derive(Default)]
struct Collector(Mutex<Vec<Group>>);
impl compositing::Backend for Collector {
    fn callback(
        &self,
        _: &egui::Context,
        group: Group,
    ) -> Result<egui::PaintCallback, compositing::Error> {
        let rect = group.bounds();
        self.0.lock().unwrap().push(group);
        Ok(egui::PaintCallback {
            rect,
            callback: Arc::new(()),
        })
    }
}
#[test]
fn drawing_requires_host_and_parent_opacity_is_applied_once() {
    let ctx = egui::Context::default();
    ctx.run_ui(Default::default(), |ui| {
        assert_eq!(
            Spinner::new(&Theme::light(), "s", "Loading")
                .show(ui)
                .paint(ui.painter())
                .unwrap_err(),
            compositing::Error::MissingHost
        );
    })
    .drop_without_applying_deltas();
    let collector = Arc::new(Collector::default());
    let _installed = compositing::install(&ctx, collector.clone()).unwrap();
    ctx.run_ui(Default::default(), |ui| {
        ui.multiply_opacity(0.6);
        Spinner::new(&Theme::light(), "s", "Loading")
            .show(ui)
            .paint(ui.painter())
            .unwrap();
    })
    .drop_without_applying_deltas();
    let groups = collector.0.lock().unwrap();
    assert_eq!(groups.len(), 1);
    assert!((groups[0].opacity() - 0.6).abs() < 0.001);
}
