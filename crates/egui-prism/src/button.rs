//! Reference-shaped buttons that leave actions and resources with the caller.

use crate::{
    compositing::{self, Error, Group, Item},
    egui,
    theme::Theme,
};
use egui::{Color32, Image, Response, Stroke, Ui, WidgetText};

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
    /// A text action with an underline on hover.
    Link,
}

/// The fixed sizes defined by the pinned shadcn Button source.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonSize {
    /// A 24-point control with 12-point reference text and icons.
    ExtraSmall,
    /// A 32-point control.
    Small,
    /// A 36-point control.
    #[default]
    Medium,
    /// A 40-point control.
    Large,
}

/// A text action with an optional leading image and a caller-owned theme.
///
/// Use a disabled UI scope to disable activation, retaining its parent painter before
/// disabling. Give reorderable
/// instances stable UI scopes, and handle `Response::clicked()` in the caller.
///
/// ```
/// use egui_prism::{button::{Button, ButtonRole}, egui, theme::Theme};
/// let theme = Theme::light();
/// egui::__run_test_ui(|ui| {
///     let output = Button::new(&theme, "Save").role(ButtonRole::Primary).show(ui);
///     let response = output.response;
///     // Submit output.drawing through the installed compositor in a real host.
///     if response.clicked() { /* application action */ }
/// });
/// ```
#[must_use = "call show and submit the returned drawing"]
pub struct Button<'theme, 'a> {
    theme: &'theme Theme,
    text: Option<WidgetText>,
    image: Option<Image<'a>>,
    role: ButtonRole,
    size: ButtonSize,
    invalid: bool,
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
            invalid: false,
        }
    }

    /// Choose the action's semantic role.
    pub fn role(mut self, role: ButtonRole) -> Self {
        self.role = role;
        self
    }

    /// Choose the reference height, horizontal padding and icon size.
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// Mark the action invalid using the reference border and focus-ring state.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Put a caller-supplied image before the label without loading an icon font.
    pub fn icon(mut self, image: impl Into<Image<'a>>) -> Self {
        self.image = Some(image.into());
        self
    }

    fn draw(self, ui: &mut Ui, icon_name: Option<&str>, handle_input: bool) -> ButtonOutput {
        let (height, padding, icon_padding, gap, icon_size) = match self.size {
            ButtonSize::ExtraSmall => (24.0, 8.0, 6.0, 4.0, 12.0),
            ButtonSize::Small => (32.0, 12.0, 10.0, 6.0, 16.0),
            ButtonSize::Medium => (36.0, 16.0, 12.0, 8.0, 16.0),
            ButtonSize::Large => (40.0, 24.0, 16.0, 8.0, 16.0),
        };
        let label = icon_name
            .unwrap_or_else(|| self.text.as_ref().map_or("", WidgetText::text))
            .to_owned();
        let text_style = ui
            .style()
            .override_text_style
            .clone()
            .unwrap_or(egui::TextStyle::Button);
        let mut font = text_style.resolve(ui.style());
        if self.size == ButtonSize::ExtraSmall {
            font.size -= 2.0;
        }
        let galley = self.text.map(|text| {
            let mut style = std::borrow::Cow::Borrowed(ui.style().as_ref());
            if style.visuals.override_text_color.is_some() {
                style.to_mut().visuals.override_text_color = None;
            }
            text.into_galley_impl(
                ui.ctx(),
                &style,
                egui::text::TextWrapping::from_wrap_mode_and_width(
                    egui::TextWrapMode::Extend,
                    f32::INFINITY,
                ),
                font.into(),
                ui.text_valign(),
            )
        });
        let text_width = galley.as_ref().map_or(0.0, |text| text.size().x);
        let has_icon = self.image.is_some();
        let content_width = text_width
            + if has_icon { icon_size } else { 0.0 }
            + if has_icon && galley.is_some() {
                gap
            } else {
                0.0
            };
        let border_width = if self.role == ButtonRole::Outline {
            1.0
        } else {
            0.0
        };
        let width = if icon_name.is_some() {
            height
        } else {
            content_width + 2.0 * if has_icon { icon_padding } else { padding } + 2.0 * border_width
        };
        let size = egui::vec2(width, height);
        let (id, allocated_rect) = ui.allocate_space(size);
        let rect = ui.layout().align_size_within_rect(size, allocated_rect);
        let mut response = ui.interact(rect, id, egui::Sense::click());
        response.set_intrinsic_size(size);
        let state_id = response.id.with("prism-button");
        let mut state = ui
            .data(|data| data.get_temp::<ButtonState>(state_id))
            .unwrap_or_default();
        let frame = ui.ctx().cumulative_frame_nr();
        if state.last_frame.is_some_and(|last| frame > last + 1) {
            state.space_armed = false;
        }
        state.last_frame = Some(frame);
        if response.enabled() {
            ui.memory_mut(|memory| {
                memory.register_focus_policy(
                    response.id,
                    response.layer_id,
                    egui::EventFilter {
                        horizontal_arrows: true,
                        vertical_arrows: true,
                        escape: true,
                        ..Default::default()
                    },
                )
            });
        }
        if handle_input {
            // Ordinary egui does not focus a click-only widget on pointer press.
            // Ordered dispatch owns this decision before handing over the event.
            if response.enabled()
                && response.is_pointer_button_down_on()
                && ui.input(|input| input.pointer.primary_pressed())
            {
                response.request_focus();
            }
            update_input(&mut response, &mut state);
        } else if !response.has_focus() || !response.enabled() {
            state.space_armed = false;
            state.focus_visible = false;
            state.focused = false;
        } else if !state.focused {
            // Navigation or accessibility may acquire focus without a targeted event.
            state.space_armed = false;
            state.focus_visible = true;
            state.focused = true;
        }
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::Button, response.enabled(), &label)
        });
        if self.invalid {
            ui.ctx().accesskit_node_builder(response.id, |node| {
                node.set_invalid(egui::accesskit::Invalid::True)
            });
        }
        let hovered = response.hovered() && response.enabled();
        let colors = self.theme.colors;
        let dark = self.theme.dark;
        let (fill, foreground) = match self.role {
            ButtonRole::Primary => (
                colors
                    .primary
                    .background
                    .gamma_multiply(if hovered { 0.9 } else { 1.0 }),
                colors.primary.foreground,
            ),
            ButtonRole::Secondary => (
                colors
                    .secondary
                    .background
                    .gamma_multiply(if hovered { 0.8 } else { 1.0 }),
                colors.secondary.foreground,
            ),
            ButtonRole::Outline if dark => (
                colors.input.gamma_multiply(if hovered { 0.5 } else { 0.3 }),
                colors.background.foreground,
            ),
            ButtonRole::Outline => (
                if hovered {
                    colors.accent.background
                } else {
                    colors.background.background
                },
                if hovered {
                    colors.accent.foreground
                } else {
                    colors.background.foreground
                },
            ),
            ButtonRole::Ghost => (
                if hovered {
                    colors
                        .accent
                        .background
                        .gamma_multiply(if dark { 0.5 } else { 1.0 })
                } else {
                    Color32::TRANSPARENT
                },
                if hovered {
                    colors.accent.foreground
                } else {
                    colors.background.foreground
                },
            ),
            ButtonRole::Destructive => (
                colors.destructive.background.gamma_multiply(if dark {
                    0.6
                } else if hovered {
                    0.9
                } else {
                    1.0
                }),
                Color32::WHITE,
            ),
            ButtonRole::Link => (Color32::TRANSPARENT, colors.primary.background),
        };
        let border = if self.invalid {
            colors.destructive.background
        } else if state.focus_visible {
            colors.ring
        } else if dark {
            colors.input
        } else {
            colors.border
        };
        let ring = if self.invalid || self.role == ButtonRole::Destructive {
            colors
                .destructive
                .background
                .gamma_multiply(if dark { 0.4 } else { 0.2 })
        } else {
            colors.ring.gamma_multiply(0.5)
        };
        let target = ButtonPaint {
            fill,
            foreground,
            border,
            ring,
            ring_width: if state.focus_visible { 3.0 } else { 0.0 },
            opacity: if response.enabled() { 1.0 } else { 0.5 },
        };
        let now = ui.input(|input| input.time);
        let paint = state.animate(target, now);
        if paint != target {
            ui.ctx().request_repaint();
        }
        ui.data_mut(|data| data.insert_temp(state_id, state));
        let drawing = if ui.is_rect_visible(rect.expand(4.0)) {
            let radius = self.theme.metrics.corner_radius;
            let mut shapes = Vec::new();
            if self.role == ButtonRole::Outline {
                shapes.push(egui::Shape::from(
                    egui::Shadow {
                        offset: [0, 1],
                        blur: 2,
                        spread: 0,
                        color: Color32::from_black_alpha(13),
                    }
                    .as_shape(rect, radius),
                ));
            }
            if paint.ring_width > 0.0 {
                shapes.push(egui::Shape::rect_stroke(
                    rect,
                    radius,
                    Stroke::new(paint.ring_width, paint.ring),
                    egui::StrokeKind::Outside,
                ));
            }
            shapes.push(egui::Shape::from(egui::epaint::RectShape::new(
                rect,
                radius,
                paint.fill,
                Stroke::new(border_width, paint.border),
                egui::StrokeKind::Inside,
            )));
            let mut x = rect.center().x - content_width / 2.0;
            if let Some(mut image) = self.image {
                if image.image_options().tint == Color32::WHITE {
                    image = image.tint(paint.foreground);
                }
                let image_rect = egui::Rect::from_min_size(
                    egui::pos2(x, rect.center().y - icon_size / 2.0),
                    egui::Vec2::splat(icon_size),
                );
                shapes.extend(image_shapes(ui, &image, image_rect));
                x += icon_size + gap;
            }
            if let Some(galley) = galley {
                let position = egui::pos2(x, rect.center().y - galley.size().y / 2.0)
                    - galley.rect.min.to_vec2();
                shapes.push(egui::Shape::galley(
                    position,
                    galley.clone(),
                    paint.foreground,
                ));
                if self.role == ButtonRole::Link && hovered {
                    for row in &galley.rows {
                        if let Some(glyph) = row.glyphs.first() {
                            let y = position.y + row.pos.y + glyph.pos.y + 4.0;
                            shapes.push(egui::Shape::line_segment(
                                [egui::pos2(x, y), egui::pos2(x + row.size.x, y)],
                                Stroke::new(1.0, paint.foreground),
                            ));
                        }
                    }
                }
            }
            let bounds = shapes.iter().fold(rect, |bounds, shape| {
                bounds.union(shape.visual_bounding_rect())
            });
            let items = shapes
                .into_iter()
                .map(|shape| {
                    Item::Shape(egui::epaint::ClippedShape::new(
                        if ui.clip_rect().is_finite() {
                            ui.clip_rect()
                        } else {
                            ui.clip_rect().intersect(bounds)
                        },
                        shape,
                    ))
                })
                .collect();
            Some(Group::new(bounds, paint.opacity, items).expect("valid button drawing"))
        } else {
            None
        };
        ButtonOutput {
            response,
            drawing,
            label,
        }
    }

    /// Lay out the button and return its response and unfaded drawing for this frame.
    pub fn show(self, ui: &mut Ui) -> ButtonOutput {
        self.draw(ui, None, true)
    }

    /// Register geometry and focus policy, or build the final drawing, without handling input.
    ///
    /// In an ordered pass, retain this output for `ButtonOutput::handle`, then
    /// call `register` again at the final layout and submit that final drawing.
    pub fn register(self, ui: &mut Ui) -> ButtonOutput {
        self.draw(ui, None, false)
    }
}

/// A button response and drawing submitted by the caller in the current frame.
#[must_use = "submit the drawing with paint or include it in a parent Group"]
pub struct ButtonOutput {
    /// The interaction response, consumed by the caller's action logic.
    pub response: Response,
    /// The unfaded group, absent when the button is not visible.
    pub drawing: Option<Group>,
    label: String,
}
impl ButtonOutput {
    /// Handle one routed event using the current authoritative response.
    ///
    /// Call only for this button's target. After a focus transfer, also call
    /// `cancel_unfocused` on the old focused button before routing another event.
    /// The returned response belongs only to this event; regenerate the final
    /// drawing with `register` after the host drains the event stream.
    pub fn handle(&self, event: &egui::RoutedInput) -> Response {
        assert_eq!(event.target, Some(self.response.id), "button event target");
        let mut response = self
            .response
            .ctx
            .read_response(self.response.id)
            .expect("registered button response");
        let state_id = response.id.with("prism-button");
        let mut state = response
            .ctx
            .data(|data| data.get_temp::<ButtonState>(state_id))
            .unwrap_or_default();
        update_input(&mut response, &mut state);
        response
            .ctx
            .data_mut(|data| data.insert_temp(state_id, state));
        if response.clicked() {
            response.widget_info(|| {
                egui::WidgetInfo::labeled(egui::Role::Button, response.enabled(), &self.label)
            });
        }
        response
    }

    /// Cancel a pending Space activation after focus or enablement is lost.
    pub fn cancel_unfocused(&self) {
        let context = &self.response.ctx;
        if !context.memory(|memory| memory.has_focus(self.response.id))
            || !context
                .read_response(self.response.id)
                .is_some_and(|response| response.enabled())
        {
            context.data_mut(|data| {
                if let Some(mut state) =
                    data.get_temp::<ButtonState>(self.response.id.with("prism-button"))
                {
                    state.space_armed = false;
                    state.focused = false;
                    state.focus_visible = false;
                    data.insert_temp(self.response.id.with("prism-button"), state);
                }
            });
        }
    }

    /// Submit with a same-layer painter captured before entering a disabled scope.
    ///
    /// The painter supplies actual parent opacity; egui's disabled fade must not
    /// already be included. Keep referenced textures alive through host submission.
    /// For overlapping siblings, put their drawings in a parent `Group` instead.
    pub fn paint(self, painter: &egui::Painter) -> Result<Response, Error> {
        assert_eq!(
            &self.response.ctx,
            painter.ctx(),
            "button painter must use the same context"
        );
        assert_eq!(
            self.response.layer_id,
            painter.layer_id(),
            "button painter must use the same layer"
        );
        if let Some(group) = self.drawing {
            compositing::paint(painter, group)?;
        }
        Ok(self.response)
    }
}

// Build image primitives without passing through a painter that may already be faded.
pub(crate) fn image_shapes(ui: &Ui, image: &Image<'_>, rect: egui::Rect) -> Vec<egui::Shape> {
    use egui::emath::GuiRounding as _;
    let rect = rect.round_to_pixels(ui.pixels_per_point());
    let image = image
        .clone()
        .fit_to_exact_size(rect.size())
        .max_size(rect.size())
        .maintain_aspect_ratio(false);
    match image.load_for_size(ui.ctx(), rect.size()) {
        Ok(egui::load::TexturePoll::Ready { texture }) => {
            let options = image.image_options();
            let mut shapes = Vec::new();
            if options.bg_fill != Color32::TRANSPARENT {
                shapes.push(egui::Shape::rect_filled(
                    rect,
                    options.corner_radius,
                    options.bg_fill,
                ));
            }
            if let Some((rotation, origin)) = options.rotation {
                assert_eq!(
                    options.corner_radius,
                    egui::CornerRadius::ZERO,
                    "rotated images cannot also have rounded corners"
                );
                let mut mesh = egui::Mesh::with_texture(texture.id);
                mesh.add_rect_with_uv(rect, options.uv, options.tint);
                mesh.rotate(rotation, rect.min + origin * rect.size());
                shapes.push(egui::Shape::mesh(mesh));
            } else {
                shapes.push(egui::Shape::from(
                    egui::epaint::RectShape::filled(rect, options.corner_radius, options.tint)
                        .with_texture(texture.id, options.uv),
                ));
            }
            shapes
        }
        Ok(egui::load::TexturePoll::Pending { .. }) => {
            // The host owns asynchronous loaders and requests repaint when an image arrives.
            Vec::new()
        }
        Err(_) => {
            let galley = ui.painter().layout_no_wrap(
                "⚠".into(),
                egui::TextStyle::Body.resolve(ui.style()),
                ui.visuals().error_fg_color,
            );
            vec![egui::Shape::galley(
                rect.center() - galley.size() * 0.5,
                galley,
                ui.visuals().error_fg_color,
            )]
        }
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
///     let _output = IconButton::new(&theme, image, "Add item").show(ui);
/// });
/// ```
#[must_use = "call show and submit the returned drawing"]
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
                invalid: false,
            },
            name,
        }
    }

    /// Choose the action's semantic role.
    pub fn role(mut self, role: ButtonRole) -> Self {
        self.button.role = role;
        self
    }

    /// Mark the icon action invalid using the reference border and focus-ring state.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.button.invalid = invalid;
        self
    }

    /// Choose the fixed reference square size.
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.button.size = size;
        self
    }
}

impl IconButton<'_, '_> {
    /// Register an image action without handling input; submit only the final drawing.
    pub fn register(self, ui: &mut Ui) -> ButtonOutput {
        let mut output = self.button.draw(ui, Some(&self.name), false);
        output.response = output.response.on_hover_text(self.name);
        output
    }

    /// Lay out the icon action and return its response and drawing for this frame.
    pub fn show(self, ui: &mut Ui) -> ButtonOutput {
        let mut output = self.button.draw(ui, Some(&self.name), true);
        output.response = output.response.on_hover_text(self.name);
        output
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ButtonPaint {
    fill: Color32,
    foreground: Color32,
    border: Color32,
    ring: Color32,
    ring_width: f32,
    opacity: f32,
}

impl ButtonPaint {
    fn mix(self, other: Self, t: f32) -> Self {
        let color = |a: Color32, b: Color32| {
            let a = a.to_array();
            let b = b.to_array();
            let c: [u8; 4] = std::array::from_fn(|i| {
                egui::lerp(f32::from(a[i])..=f32::from(b[i]), t).round() as u8
            });
            Color32::from_rgba_premultiplied(c[0], c[1], c[2], c[3])
        };
        Self {
            fill: color(self.fill, other.fill),
            foreground: color(self.foreground, other.foreground),
            border: color(self.border, other.border),
            ring: color(self.ring, other.ring),
            ring_width: egui::lerp(self.ring_width..=other.ring_width, t),
            opacity: egui::lerp(self.opacity..=other.opacity, t),
        }
    }
}

fn update_input(response: &mut Response, state: &mut ButtonState) {
    let context = response.ctx.clone();
    let pointer_pressed = response.is_pointer_button_down_on()
        && context.input(|input| input.pointer.primary_pressed());
    let focused = response.has_focus() && response.enabled();
    if pointer_pressed || (focused && !state.focused) {
        state.space_armed = false;
    }
    if focused {
        let (clicked, any_key) = context.input(|input| {
            let mut clicked =
                input.has_accesskit_action_request(response.id, egui::accesskit::Action::Click);
            let mut any_key = false;
            for event in &input.events {
                if let egui::Event::Key {
                    key,
                    pressed,
                    repeat,
                    ..
                } = event
                {
                    any_key |= *pressed;
                    match key {
                        egui::Key::Space if *pressed && !repeat => state.space_armed = true,
                        egui::Key::Space if !pressed => {
                            clicked |= state.space_armed;
                            state.space_armed = false;
                        }
                        egui::Key::Enter if *pressed => clicked = true,
                        _ => {}
                    }
                }
            }
            (clicked, any_key)
        });
        response
            .flags
            .set(egui::response::Flags::FAKE_PRIMARY_CLICKED, clicked);
        if !state.focused || any_key {
            state.focus_visible = true;
        }
        if pointer_pressed {
            state.focus_visible = false;
        }
    } else {
        state.space_armed = false;
        state.focus_visible = false;
    }
    state.focused = focused;
}

#[derive(Clone, Default)]
struct ButtonState {
    last_frame: Option<u64>,
    space_armed: bool,
    focused: bool,
    focus_visible: bool,
    transition: Option<(ButtonPaint, ButtonPaint, f64)>,
}

impl ButtonState {
    fn animate(&mut self, target: ButtonPaint, now: f64) -> ButtonPaint {
        let Some((from, to, start)) = self.transition else {
            self.transition = Some((target, target, now));
            return target;
        };
        let elapsed = ((now - start) / 0.15).clamp(0.0, 1.0) as f32;
        let current = if elapsed >= 1.0 {
            to
        } else {
            from.mix(to, css_ease(elapsed))
        };
        if to != target {
            self.transition = Some((current, target, now));
        }
        current
    }
}

// Tailwind's default timing function: cubic-bezier(0.4, 0, 0.2, 1).
fn css_ease(x: f32) -> f32 {
    let mut low = 0.0;
    let mut high = 1.0;
    for _ in 0..16 {
        let t = (low + high) / 2.0;
        let bx = 1.2 * (1.0 - t) * (1.0 - t) * t + 0.6 * (1.0 - t) * t * t + t * t * t;
        if bx < x {
            low = t;
        } else {
            high = t;
        }
    }
    let t = (low + high) / 2.0;
    3.0 * (1.0 - t) * t * t + t * t * t
}
