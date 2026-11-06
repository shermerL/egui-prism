//! Public theme contrast and host-configuration checks.

use egui_prism::{
    egui,
    theme::{ColorPair, Theme},
};

#[test]
fn neutral_tokens_match_pinned_css_and_preserve_dark_alpha() {
    let css = include_str!("../../../reference/source/globals.css");
    for (theme, selector) in [(Theme::light(), ":root {"), (Theme::dark(), ".dark {")] {
        let block = css
            .split_once(selector)
            .unwrap()
            .1
            .split('}')
            .next()
            .unwrap();
        for (name, color) in [
            ("background", theme.colors.background.background),
            ("foreground", theme.colors.background.foreground),
            ("primary", theme.colors.primary.background),
            ("primary-foreground", theme.colors.primary.foreground),
            ("muted", theme.colors.muted.background),
            ("muted-foreground", theme.colors.muted.foreground),
            ("secondary", theme.colors.secondary.background),
            ("accent", theme.colors.accent.background),
            ("ring", theme.colors.ring),
        ] {
            let prefix = format!("--{name}: oklch(");
            let value = block
                .split_once(&prefix)
                .unwrap()
                .1
                .split(')')
                .next()
                .unwrap();
            let mut channels = value.split_whitespace();
            let lightness = channels.next().unwrap();
            let l: f64 = if let Some(percent) = lightness.strip_suffix('%') {
                percent.parse::<f64>().unwrap() / 100.0
            } else {
                lightness.parse().unwrap()
            };
            assert_eq!(channels.next(), Some("0"));
            assert_eq!(channels.next(), Some("0"));
            // Neutral OKLCH has equal linear RGB channels L cubed.
            let linear = l.powi(3);
            let srgb = if linear <= 0.0031308 {
                12.92 * linear
            } else {
                1.055 * linear.powf(1.0 / 2.4) - 0.055
            };
            assert_eq!(
                color,
                egui::Color32::from_gray((srgb * 255.0).round() as u8),
                "{name}"
            );
        }
    }
    assert_eq!(
        Theme::dark().colors.border,
        egui::Color32::from_white_alpha(26)
    );
    assert_eq!(
        Theme::dark().colors.input,
        egui::Color32::from_white_alpha(38)
    );
}

#[test]
fn font_setup_retains_host_families_and_installs_each_weight_once() {
    let context = egui::Context::default();
    let custom = egui::TextStyle::Name("host-caption".into());
    context.all_styles_mut(|style| {
        style
            .text_styles
            .insert(custom.clone(), egui::FontId::monospace(15.0));
    });
    let mut definitions = egui::FontDefinitions::default();
    let original_fallbacks = definitions.families[&egui::FontFamily::Proportional].clone();
    for _ in 0..2 {
        egui_prism::theme::install_fonts(&context, definitions);
        context
            .run_ui(Default::default(), |ui| {
                ui.label("Geist");
            })
            .drop_without_applying_deltas();
        definitions = context.fonts(|fonts| fonts.definitions().clone());
    }
    context.fonts(|fonts| {
        let defs = fonts.definitions();
        let proportional = &defs.families[&egui::FontFamily::Proportional];
        assert_eq!(proportional[0], "prism-geist-regular");
        assert_eq!(
            proportional
                .iter()
                .filter(|name| *name == "prism-geist-regular")
                .count(),
            1
        );
        assert_eq!(
            defs.families[&egui::FontFamily::Monospace][0],
            "prism-geist-mono"
        );
        for name in ["prism-geist-medium", "prism-geist-semibold"] {
            let chain = &defs.families[&egui::FontFamily::Name(name.into())];
            assert_eq!(chain[0], name);
            assert!(
                original_fallbacks
                    .iter()
                    .all(|fallback| chain.contains(fallback))
            );
        }
    });
    context.fonts_mut(|fonts| {
        for name in ["prism-geist-medium", "prism-geist-semibold"] {
            assert!(fonts.has_glyph(
                &egui::FontId::new(14.0, egui::FontFamily::Name(name.into())),
                '⅖'
            ));
        }
    });
    assert_eq!(
        context.style_of(egui::Theme::Light).text_styles[&custom],
        egui::FontId::monospace(15.0)
    );
    assert_eq!(
        context.style_of(egui::Theme::Light).text_styles[&egui::TextStyle::Button].family,
        egui::FontFamily::Name("prism-geist-medium".into())
    );
}

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
            ("primary", colors.primary),
            ("selection", colors.selection),
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
            theme.colors.focus.background
        );
        assert_eq!(
            style.visuals.selection.stroke.color,
            theme.colors.focus.foreground
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

#[test]
fn composition_indicators_remain_legible_with_primary_text_selection() {
    for theme in [Theme::light(), Theme::dark()] {
        for legacy_visuals in [false, true] {
            let mut style = egui::Style::default();
            style.visuals.ime_composition.legacy_visuals = legacy_visuals;
            theme.apply(&mut style);
            let background = style.visuals.text_edit_bg_color.unwrap();
            assert!(
                contrast(ColorPair {
                    background,
                    foreground: style.visuals.selection.stroke.color
                }) >= 3.0
            );
            // Prism TextInput overrides selection and paints a separate ring.
            for stroke in [
                style.visuals.ime_composition.active_underline_stroke,
                style.visuals.ime_composition.inactive_underline_stroke,
            ] {
                assert!(
                    contrast(ColorPair {
                        background,
                        foreground: stroke.color
                    }) >= 3.0
                );
            }
            assert_eq!(style.visuals.ime_composition.legacy_visuals, legacy_visuals);
        }
    }
}

#[test]
fn ordinary_editor_keyboard_focus_has_a_visible_outline() {
    for theme in [Theme::light(), Theme::dark()] {
        let context = egui::Context::default();
        let id = egui::Id::unique("ordinary-editor");
        context.memory_mut(|memory| memory.request_focus(id));
        let mut value = String::from("Ordinary editor");
        // Widget state derives from the previous pass's interaction response.
        for _ in 0..2 {
            context
                .run_ui(Default::default(), |ui| {
                    theme.apply(ui.style_mut());
                    ui.add(egui::TextEdit::singleline(&mut value).id(id));
                })
                .drop_without_applying_deltas();
        }
        let mut output = context.run_ui(Default::default(), |ui| {
            theme.apply(ui.style_mut());
            let response = ui.add(egui::TextEdit::singleline(&mut value).id(id));
            assert!(response.has_focus());
        });
        output.textures_delta.clear();
        assert!(output.shapes.iter().any(|shape| {
            matches!(&shape.shape, egui::epaint::Shape::Rect(rect)
                if rect.stroke.width > 0.0 && rect.stroke.color == theme.colors.focus.foreground)
        }));
        output.drop_without_applying_deltas();
    }
}
