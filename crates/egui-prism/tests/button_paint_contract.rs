//! Targeted regressions for Button clipping and raster loader requests.
use egui::load::{SizeHint, SizedTexture, TextureLoadResult, TextureLoader, TexturePoll};
use egui_prism::{
    button::{Button, IconButton},
    egui,
    theme::Theme,
};
use std::sync::{Arc, Mutex};

#[test]
fn accepts_unbounded_ui_clip() {
    egui::__run_test_ui(|ui| {
        ui.set_clip_rect(egui::Rect::EVERYTHING);
        let output = Button::new(&Theme::light(), "Action").show(ui);
        let group = output.drawing.unwrap();
        assert!(group.bounds().is_finite());
        for item in group.items() {
            let egui_prism::compositing::Item::Shape(shape) = item else {
                panic!("expected a primitive")
            };
            assert!(
                shape
                    .clip_rect
                    .contains_rect(shape.shape.visual_bounding_rect())
            );
        }
    });
}

#[derive(Default)]
struct RecordingLoader(Mutex<Vec<SizeHint>>);
impl TextureLoader for RecordingLoader {
    fn id(&self) -> &str {
        "button-raster-hint-test"
    }
    fn load(
        &self,
        _: &egui::Context,
        _: &str,
        _: egui::TextureOptions,
        hint: SizeHint,
    ) -> TextureLoadResult {
        self.0.lock().unwrap().push(hint);
        Ok(TexturePoll::Ready {
            texture: SizedTexture::new(egui::TextureId::User(7), egui::vec2(16.0, 16.0)),
        })
    }
    fn forget(&self, _: &str) {}
    fn forget_all(&self) {}
    fn byte_size(&self) -> usize {
        0
    }
}

#[test]
fn icon_requests_the_fixed_raster_size_even_with_a_smaller_image_fit() {
    let loader = Arc::new(RecordingLoader::default());
    let context = egui::Context::default();
    context.add_texture_loader(loader.clone());
    context.set_pixels_per_point(1.5);
    context
        .run_ui(Default::default(), |ui| {
            let image = egui::Image::new("bytes://button-test.svg")
                .fit_to_exact_size(egui::vec2(3.0, 5.0))
                .max_size(egui::vec2(4.0, 6.0));
            image.paint_at(
                ui,
                egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(16.0)),
            );
            let _output = IconButton::new(&Theme::light(), image, "Action").show(ui);
        })
        .drop_without_applying_deltas();
    let expected = SizeHint::Size {
        width: 24,
        height: 24,
        maintain_aspect_ratio: false,
    };
    assert_eq!(*loader.0.lock().unwrap(), [expected, expected]);
}
