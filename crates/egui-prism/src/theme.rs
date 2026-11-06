//! Semantic colors and layout tokens shared by components and their host.

use egui::{Color32, CornerRadius, FontId, Margin, Stroke, Style, TextStyle, Vec2, vec2};

/// A foreground and its intended opaque background.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorPair {
    /// The surface behind the content.
    pub background: Color32,
    /// The text or icon drawn on the surface.
    pub foreground: Color32,
}

/// Semantic color pairs that applications may customize for their brand.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Palette {
    /// The page and its ordinary text.
    pub background: ColorPair,
    /// Containers, inputs, and resting controls.
    pub surface: ColorPair,
    /// Subdued content and hover surfaces.
    pub muted: ColorPair,
    /// Supporting actions and their content.
    pub secondary: ColorPair,
    /// Hovered surfaces and their content.
    pub accent: ColorPair,
    /// The primary action and its content.
    pub primary: ColorPair,
    /// Selected text and selected controls.
    pub selection: ColorPair,
    /// Destructive actions and their content.
    pub destructive: ColorPair,
    /// Explicit disabled component surfaces, distinct from egui's opacity effect.
    pub disabled: ColorPair,
    /// Focused or pressed controls and their content.
    pub focus: ColorPair,
    /// Decorative separators and resting outlines.
    pub border: Color32,
    /// Input outlines and translucent dark input fills.
    pub input: Color32,
    /// The neutral focus ring before component-specific opacity is applied.
    pub ring: Color32,
}

/// Shared logical-point measurements with separate page and container ownership.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Metrics {
    /// Outer margin applied once by the page host.
    pub page_margin: Margin,
    /// Inner margin applied once by each container.
    pub container_margin: Margin,
    /// Space between sibling widgets.
    pub item_spacing: Vec2,
    /// Padding inside a control's background.
    pub control_padding: Vec2,
    /// Default button/input height; other components own their reference geometry.
    pub control_height: f32,
    /// Corner radius shared by controls and containers.
    pub corner_radius: CornerRadius,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            page_margin: Margin::same(16),
            container_margin: Margin::same(12),
            item_spacing: vec2(8.0, 8.0),
            control_padding: vec2(16.0, 8.0),
            control_height: 36.0,
            corner_radius: CornerRadius::same(8),
        }
    }
}

/// Text sizes in logical points, leaving font families and font data to the host.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Typography {
    /// Ordinary text, button text, and monospace text size.
    pub body: f32,
    /// Supporting text size.
    pub small: f32,
    /// Section heading size.
    pub heading: f32,
}

impl Default for Typography {
    fn default() -> Self {
        Self {
            body: 14.0,
            small: 12.0,
            heading: 24.0,
        }
    }
}

/// An editable theme applied to an existing egui style without owning application state.
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    /// Semantic colors shared by the host and its components.
    pub colors: Palette,
    /// Layout measurements shared by the host and its components.
    pub metrics: Metrics,
    /// Text sizes that preserve the host's font families.
    pub typography: Typography,
    dark: bool,
}

const fn pair(background: u32, foreground: u32) -> ColorPair {
    const fn color(rgb: u32) -> Color32 {
        Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
    }
    ColorPair {
        background: color(background),
        foreground: color(foreground),
    }
}

/// Install the reference Geist fonts once during host setup, preserving custom families.
pub fn install_fonts(context: &egui::Context) {
    for (name, bytes, family) in [
        (
            "prism-geist-mono",
            include_bytes!("../fonts/GeistMono-Regular.ttf").as_slice(),
            egui::FontFamily::Monospace,
        ),
        (
            "prism-geist-regular",
            include_bytes!("../fonts/Geist-Regular.ttf").as_slice(),
            egui::FontFamily::Proportional,
        ),
        (
            "prism-geist-medium",
            include_bytes!("../fonts/Geist-Medium.ttf").as_slice(),
            egui::FontFamily::Name("prism-geist-medium".into()),
        ),
        (
            "prism-geist-semibold",
            include_bytes!("../fonts/Geist-SemiBold.ttf").as_slice(),
            egui::FontFamily::Name("prism-geist-semibold".into()),
        ),
    ] {
        context.add_font(egui::FontInsert::new(
            name,
            egui::FontData::from_static(bytes),
            vec![egui::InsertFontFamily {
                family,
                priority: egui::FontPriority::Highest,
            }],
        ));
    }
    context.all_styles_mut(|style| {
        for (text_style, family) in [
            (TextStyle::Button, "prism-geist-medium"),
            (TextStyle::Heading, "prism-geist-semibold"),
        ] {
            if let Some(font) = style.text_styles.get_mut(&text_style)
                && font.family == egui::FontFamily::Proportional
            {
                font.family = egui::FontFamily::Name(family.into());
            }
        }
    });
}

impl Theme {
    /// Create the built-in light theme.
    pub fn light() -> Self {
        Self {
            colors: Palette {
                background: pair(0xffffff, 0x000000),
                surface: pair(0xffffff, 0x000000),
                muted: pair(0xf5f5f5, 0x737373),
                secondary: pair(0xf5f5f5, 0x171717),
                accent: pair(0xf5f5f5, 0x171717),
                primary: pair(0x000000, 0xfafafa),
                selection: pair(0x000000, 0xffffff),
                destructive: pair(0xe7000b, 0xffffff),
                disabled: pair(0xf5f5f5, 0x737373),
                focus: pair(0xf5f5f5, 0x171717),
                border: Color32::from_gray(229),
                input: Color32::from_gray(229),
                ring: Color32::from_gray(161),
            },
            metrics: Metrics::default(),
            typography: Typography::default(),
            dark: false,
        }
    }

    /// Create the built-in dark theme.
    pub fn dark() -> Self {
        Self {
            colors: Palette {
                background: pair(0x0a0a0a, 0xfafafa),
                surface: pair(0x171717, 0xfafafa),
                muted: pair(0x262626, 0xa1a1a1),
                secondary: pair(0x262626, 0xfafafa),
                accent: pair(0x404040, 0xfafafa),
                primary: pair(0xe5e5e5, 0x171717),
                selection: pair(0xe5e5e5, 0x171717),
                destructive: pair(0xff6467, 0xffffff),
                disabled: pair(0x262626, 0xa1a1a1),
                focus: pair(0x262626, 0xfafafa),
                border: Color32::from_white_alpha(26),
                input: Color32::from_white_alpha(38),
                ring: Color32::from_gray(115),
            },
            metrics: Metrics::default(),
            typography: Typography::default(),
            dark: true,
        }
    }

    /// Apply theme-owned style fields while preserving font families and interaction settings.
    ///
    /// Global text-color overrides are cleared so selection keeps its paired foreground.
    /// Page margins remain the host's responsibility; egui disabled UI retains its opacity.
    ///
    /// ```
    /// use egui_prism::{egui, theme::Theme};
    /// let mut style = egui::Style::default();
    /// let mut theme = Theme::light();
    /// theme.typography.body = 16.0;
    /// theme.apply(&mut style);
    /// ```
    pub fn apply(&self, style: &mut Style) {
        let colors = self.colors;
        let metrics = self.metrics;
        let visuals = &mut style.visuals;
        visuals.dark_mode = self.dark;
        visuals.override_text_color = None;
        visuals.weak_text_color = Some(colors.muted.foreground);
        visuals.panel_fill = colors.background.background;
        visuals.window_fill = colors.surface.background;
        visuals.window_stroke = Stroke::new(1.0, colors.border);
        visuals.window_corner_radius = metrics.corner_radius;
        visuals.menu_corner_radius = metrics.corner_radius;
        visuals.faint_bg_color = colors.muted.background;
        visuals.extreme_bg_color = colors.surface.background;
        visuals.text_edit_bg_color = Some(colors.surface.background);
        visuals.code_bg_color = colors.muted.background;
        visuals.hyperlink_color = colors.focus.foreground;
        visuals.error_fg_color = colors.destructive.background;
        visuals.selection.bg_fill = colors.selection.background;
        visuals.selection.stroke = Stroke::new(1.0, colors.selection.foreground);
        visuals.text_cursor.stroke = Stroke::new(1.5, colors.surface.foreground);
        visuals.ime_composition.active_underline_stroke = Stroke::new(1.5, colors.focus.foreground);
        visuals.ime_composition.inactive_underline_stroke =
            Stroke::new(1.0, colors.muted.foreground);
        for (widget, pair, outline) in [
            (
                &mut visuals.widgets.noninteractive,
                colors.background,
                colors.border,
            ),
            (&mut visuals.widgets.inactive, colors.surface, colors.border),
            (&mut visuals.widgets.hovered, colors.muted, colors.border),
            (
                &mut visuals.widgets.active,
                colors.focus,
                colors.focus.foreground,
            ),
            (
                &mut visuals.widgets.open,
                colors.muted,
                colors.focus.foreground,
            ),
        ] {
            widget.bg_fill = pair.background;
            widget.weak_bg_fill = pair.background;
            widget.fg_stroke = Stroke::new(1.0, pair.foreground);
            widget.bg_stroke = Stroke::new(1.0, outline);
            widget.corner_radius = metrics.corner_radius;
            widget.expansion = 0.0;
        }
        style.spacing.item_spacing = metrics.item_spacing;
        style.spacing.button_padding = metrics.control_padding;
        style.spacing.interact_size.y = metrics.control_height;
        style.spacing.window_margin = metrics.container_margin;
        style.spacing.menu_margin = metrics.container_margin;
        for (text, size) in [
            (TextStyle::Body, self.typography.body),
            (TextStyle::Button, self.typography.body),
            (TextStyle::Monospace, self.typography.body),
            (TextStyle::Small, self.typography.small),
            (TextStyle::Heading, self.typography.heading),
        ] {
            if let Some(FontId { size: current, .. }) = style.text_styles.get_mut(&text) {
                *current = size;
            }
        }
    }
}
