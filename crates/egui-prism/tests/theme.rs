//! Public theme contrast and host-configuration checks.

use egui_prism::{
    egui,
    theme::{ColorPair, Theme},
};

fn contrast(pair: ColorPair) -> f64 {
    fn luminance(color: egui::Color32) -> f64 {
        let channel = |value: u8| {
            let value = f64::from(value) / 255.0;
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * channel(color.r()) + 0.7152 * channel(color.g()) + 0.0722 * channel(color.b())
    }
    let foreground = luminance(pair.foreground);
    let background = luminance(pair.background);
    (foreground.max(background) + 0.05) / (foreground.min(background) + 0.05)
}

#[test]
fn default_pairs_keep_ordinary_text_readable() {
    for theme in [Theme::light(), Theme::dark()] {
        let colors = theme.colors;
        for (role, pair) in [
            ("page", colors.background),
            ("surface", colors.surface),
            ("muted", colors.muted),
            ("primary", colors.primary),
            ("selection", colors.selection),
            ("destructive", colors.destructive),
            ("disabled", colors.disabled),
            ("focus", colors.focus),
        ] {
            assert_eq!(pair.background.a(), 255, "{role} background must be opaque");
            assert_eq!(pair.foreground.a(), 255, "{role} foreground must be opaque");
            assert!(contrast(pair) >= 4.5, "{role}: {}", contrast(pair));
        }
    }
}

#[test]
fn switching_themes_preserves_host_fonts_and_interaction_settings() {
    let mut style = egui::Style::default();
    let family = egui::FontFamily::Name("application-font".into());
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(19.0, family.clone()),
    );
    let custom = egui::TextStyle::Name("application-caption".into());
    let custom_font = egui::FontId::new(13.0, family.clone());
    style
        .text_styles
        .insert(custom.clone(), custom_font.clone());
    style.animation_time = 0.75;
    let interaction = style.interaction.clone();
    for theme in [Theme::light(), Theme::dark(), Theme::light()] {
        theme.apply(&mut style);
        assert_eq!(style.text_styles[&egui::TextStyle::Body].family, family);
        assert_eq!(style.text_styles[&custom], custom_font);
        assert_eq!(style.interaction, interaction);
        assert_eq!(style.animation_time, 0.75);
        let applied = style.clone();
        theme.apply(&mut style);
        assert_eq!(style, applied);
    }
}

#[test]
fn selection_and_focus_keep_their_foregrounds_without_global_override() {
    for theme in [Theme::light(), Theme::dark()] {
        let mut style = egui::Style::default();
        style.visuals.override_text_color = Some(egui::Color32::RED);
        theme.apply(&mut style);
        assert_eq!(style.visuals.override_text_color, None);
        assert_eq!(
            style.visuals.selection.bg_fill,
            theme.colors.selection.background
        );
        assert_eq!(
            style.visuals.selection.stroke.color,
            theme.colors.selection.foreground
        );
        let active = style.visuals.widgets.active;
        assert!(
            contrast(ColorPair {
                background: active.bg_fill,
                foreground: active.fg_stroke.color
            }) >= 4.5
        );
        assert_eq!(active.bg_stroke.color, theme.colors.focus.foreground);
    }
}
