//! Public compositing capability and immutable group contracts.
use egui_prism::{
    compositing::{self, Backend, Error, Group, Item},
    egui::{self, epaint::ClippedShape},
};
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct Recording(Mutex<Vec<Group>>);
impl Backend for Recording {
    fn callback(&self, _: &egui::Context, group: Group) -> Result<egui::PaintCallback, Error> {
        let rect = group.bounds();
        self.0.lock().unwrap().push(group);
        Ok(egui::PaintCallback {
            rect,
            callback: Arc::new(()),
        })
    }
}
fn bounds() -> egui::Rect {
    egui::Rect::from_min_size(egui::pos2(4.0, 8.0), egui::vec2(80.0, 40.0))
}
fn group(opacity: f32) -> Group {
    Group::new(
        bounds(),
        opacity,
        vec![Item::Shape(ClippedShape::new(
            bounds(),
            egui::Shape::rect_filled(bounds(), 0, egui::Color32::WHITE),
        ))],
    )
    .unwrap()
}
#[test]
fn installation_is_explicit_unique_and_released() {
    let context = egui::Context::default();
    let painter = context.debug_painter();
    assert_eq!(
        compositing::paint(&painter, group(1.0)),
        Err(Error::MissingHost)
    );
    let backend = Arc::new(Recording::default());
    let registration = compositing::install(&context, backend.clone()).unwrap();
    assert!(matches!(
        compositing::install(&context, backend.clone()),
        Err(Error::AlreadyInstalled)
    ));
    compositing::paint(&painter, group(0.0)).unwrap();
    assert_eq!(backend.0.lock().unwrap().len(), 1);
    drop(registration);
    assert_eq!(
        compositing::paint(&painter, group(1.0)),
        Err(Error::MissingHost)
    );
    let _replacement = compositing::install(&context, backend).unwrap();
}
#[test]
fn inherited_opacity_is_applied_to_the_group_not_its_contents() {
    let context = egui::Context::default();
    let backend = Arc::new(Recording::default());
    let _registration = compositing::install(&context, backend.clone()).unwrap();
    let mut painter = context.debug_painter();
    painter.set_opacity(0.5);
    let parent = Group::new(bounds(), 0.5, vec![Item::Group(group(0.75))]).unwrap();
    compositing::paint(&painter, parent).unwrap();
    let captured = backend.0.lock().unwrap();
    assert_eq!(captured[0].opacity(), 0.25);
    let Item::Group(child) = &captured[0].items()[0] else {
        panic!()
    };
    assert_eq!(child.opacity(), 0.75);
    let Item::Shape(shape) = &child.items()[0] else {
        panic!()
    };
    let egui::Shape::Rect(rect) = &shape.shape else {
        panic!()
    };
    assert_eq!(rect.fill, egui::Color32::WHITE);
}
#[test]
fn invalid_geometry_opacity_meshes_and_nested_callbacks_are_rejected() {
    for opacity in [f32::NAN, f32::INFINITY, -0.01, 1.01] {
        assert!(Group::new(bounds(), opacity, vec![]).is_err());
    }
    for rect in [
        egui::Rect::NOTHING,
        egui::Rect::EVERYTHING,
        egui::Rect::ZERO,
    ] {
        assert!(Group::new(rect, 1.0, vec![]).is_err());
    }
    let callback = egui::Shape::Callback(egui::PaintCallback {
        rect: bounds(),
        callback: Arc::new(()),
    });
    let callback = egui::Shape::Vec(vec![egui::Shape::Vec(vec![callback])]);
    assert!(
        Group::new(
            bounds(),
            1.0,
            vec![Item::Shape(ClippedShape::new(bounds(), callback))]
        )
        .is_err()
    );
    let mesh = egui::Mesh {
        indices: vec![0, 1, 2],
        ..Default::default()
    };
    assert!(
        Group::new(
            bounds(),
            1.0,
            vec![Item::Shape(ClippedShape::new(bounds(), mesh))]
        )
        .is_err()
    );
}
#[test]
fn backend_failures_propagate_without_emitting_a_fallback() {
    struct Failed;
    impl Backend for Failed {
        fn callback(&self, _: &egui::Context, _: Group) -> Result<egui::PaintCallback, Error> {
            Err(Error::Renderer("lost context".into()))
        }
    }
    let context = egui::Context::default();
    let _installation = compositing::install(&context, Arc::new(Failed)).unwrap();
    let mut output = context.run_ui(egui::RawInput::default(), |ui| {
        assert_eq!(
            compositing::paint(&ui.ctx().debug_painter(), group(0.5)),
            Err(Error::Renderer("lost context".into()))
        );
    });
    assert!(output.shapes.is_empty());
    output.textures_delta.clear();
}
