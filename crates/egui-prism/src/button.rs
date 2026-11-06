//! Stateless themed buttons that leave actions and resources with the caller.

use crate::{egui, theme::Theme};
use egui::{Image, Response, Stroke, Ui, Vec2, Widget, WidgetText};

/// The visual importance of an action, chosen by its caller.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonRole {
    /// The main action in a group.
    #[default]
    Primary,
    /// A supporting action on a muted surface.
    Secondary,
    /// A surface-colored action with a resting outline.
    Outline,
    /// An action whose background appears during interaction.
    Ghost,
    /// An action that destroys or removes caller-owned data.
    Destructive,
}

/// Control density relative to the theme's baseline metrics.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonSize {
    /// Compact padding and a lower minimum height.
    Small,
    /// The theme's baseline padding and minimum height.
    #[default]
    Medium,
    /// More padding and a higher minimum height.
    Large,
}

/// A text action with an optional leading image and a caller-owned theme.
///
/// Use `ui.add_enabled(false, button)` to disable activation. Give reorderable
/// instances stable UI scopes, and handle `Response::clicked()` in the caller.
///
/// ```
/// use egui_prism::{button::{Button, ButtonRole}, egui, theme::Theme};
/// let theme = Theme::light();
/// egui::__run_test_ui(|ui| {
///     let response = ui.add(Button::new(&theme, "Save").role(ButtonRole::Primary));
///     if response.clicked() { /* application action */ }
/// });
/// ```
#[must_use = "Add the button to a Ui with ui.add(button)"]
pub struct Button<'theme, 'a> {
    theme: &'theme Theme,
    text: Option<WidgetText>,
    image: Option<Image<'a>>,
    role: ButtonRole,
    size: ButtonSize,
}

impl<'theme, 'a> Button<'theme, 'a> {
    /// Create a primary text button using the borrowed theme.
    pub fn new(theme: &'theme Theme, text: impl Into<WidgetText>) -> Self {
        Self {
            theme,
            text: Some(text.into()),
            image: None,
            role: ButtonRole::Primary,
            size: ButtonSize::Medium,
        }
    }

    /// Choose the action's semantic role.
    pub fn role(mut self, role: ButtonRole) -> Self {
        self.role = role;
        self
    }

    /// Choose padding and minimum height relative to the theme.
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// Put a caller-supplied image before the label without loading an icon font.
    pub fn icon(mut self, image: impl Into<Image<'a>>) -> Self {
        self.image = Some(image.into());
        self
    }

    fn show(self, ui: &mut Ui, square: bool) -> Response {
        // Borrow the host layout and IDs directly; a nested Ui would alter both.
        let previous_style = ui.style().clone();
        let style = ui.style_mut();
        let colors = self.theme.colors;
        let metrics = self.theme.metrics;
        let scale = match self.size {
            ButtonSize::Small => 0.875,
            ButtonSize::Medium => 1.0,
            ButtonSize::Large => 1.25,
        };
        let height = metrics.control_height * scale;
        style.spacing.button_padding = if square {
            Vec2::splat(metrics.control_padding.y * scale)
        } else {
            metrics.control_padding * scale
        };
        style.spacing.interact_size.y = height;
        style.visuals.override_text_color = None;
        let resting = match self.role {
            ButtonRole::Primary => colors.primary,
            ButtonRole::Secondary => colors.muted,
            ButtonRole::Outline | ButtonRole::Ghost => colors.surface,
            ButtonRole::Destructive => colors.destructive,
        };
        for (visuals, pair, outline) in [
            (&mut style.visuals.widgets.inactive, resting, colors.border),
            (
                &mut style.visuals.widgets.hovered,
                resting,
                resting.foreground,
            ),
            (
                &mut style.visuals.widgets.active,
                colors.focus,
                colors.focus.foreground,
            ),
            (
                &mut style.visuals.widgets.noninteractive,
                colors.disabled,
                colors.border,
            ),
        ] {
            visuals.bg_fill = pair.background;
            visuals.weak_bg_fill = pair.background;
            visuals.fg_stroke = Stroke::new(1.0, pair.foreground);
            visuals.bg_stroke = Stroke::new(1.0, outline);
            visuals.corner_radius = metrics.corner_radius;
            visuals.expansion = 0.0;
        }
        if matches!(
            self.role,
            ButtonRole::Primary | ButtonRole::Secondary | ButtonRole::Destructive
        ) {
            style.visuals.widgets.inactive.bg_stroke = Stroke::NONE;
        }
        if self.role == ButtonRole::Ghost {
            style.visuals.widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
            style.visuals.widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
            style.visuals.widgets.inactive.bg_stroke = Stroke::NONE;
            style.visuals.widgets.hovered.bg_fill = colors.muted.background;
            style.visuals.widgets.hovered.weak_bg_fill = colors.muted.background;
            style.visuals.widgets.hovered.fg_stroke.color = colors.muted.foreground;
        }
        let tint_follows_text = self
            .image
            .as_ref()
            .is_some_and(|image| image.image_options().tint == egui::Color32::WHITE);
        let text = self
            .text
            .map(|text| text.fallback_text_style(egui::TextStyle::Button));
        let button = egui::Button::opt_image_and_text(self.image, text)
            .image_tint_follows_text_color(tint_follows_text)
            .wrap()
            .frame(true)
            .min_size(egui::vec2(if square { height } else { 0.0 }, height));
        let response = ui.add(button);
        ui.set_style(previous_style);
        response
    }
}

impl Widget for Button<'_, '_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui, false)
    }
}

/// An image-only action with a required accessible name and caller-owned image.
///
/// The name is exposed through egui's widget information and used as a tooltip.
/// The host owns texture lifetime, image loaders, and platform accessibility.
///
/// ```
/// use egui_prism::{button::IconButton, egui, theme::Theme};
/// let theme = Theme::light();
/// egui::__run_test_ui(|ui| {
///     // The application supplies and retains the real texture in production.
///     let image = (egui::TextureId::User(1), egui::vec2(16.0, 16.0));
///     let _response = ui.add(IconButton::new(&theme, image, "Add item"));
/// });
/// ```
#[must_use = "Add the icon button to a Ui with ui.add(button)"]
pub struct IconButton<'theme, 'a> {
    button: Button<'theme, 'a>,
    name: String,
}

impl<'theme, 'a> IconButton<'theme, 'a> {
    /// Create an icon action; an empty accessible name is a programmer error.
    pub fn new(theme: &'theme Theme, image: impl Into<Image<'a>>, name: impl Into<String>) -> Self {
        let name = name.into();
        assert!(
            !name.trim().is_empty(),
            "IconButton requires a nonempty accessible name"
        );
        Self {
            button: Button {
                theme,
                text: None,
                image: Some(image.into().alt_text(name.clone())),
                role: ButtonRole::Primary,
                size: ButtonSize::Medium,
            },
            name,
        }
    }

    /// Choose the action's semantic role.
    pub fn role(mut self, role: ButtonRole) -> Self {
        self.button.role = role;
        self
    }

    /// Choose the minimum square hit area relative to the theme.
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.button.size = size;
        self
    }
}

impl Widget for IconButton<'_, '_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.button.show(ui, true).on_hover_text(self.name)
    }
}
