//! Passive caller-owned item composition with independent, explicitly identified actions.
//!
//! Title, description, decorative icon and Button/IconButton slots are supported.
//! This is not an interactive row, selection model or arbitrary widget container.
//! Description balancing uses egui line breaks; exact CSS text-balancing and
//! browser min-content parity are not established. See `docs/items.md`.

use crate::{
    button::{Button, ButtonOutput, IconButton},
    compositing::{self, Error, Group, Item as DrawItem},
    theme::Theme,
};
use egui::{
    Color32, FontId, Galley, IdSalt, Image, Rect, Response, Sense, Shape, Stroke, StrokeKind,
    TextFormat, TextStyle, Ui, UiBuilder, pos2, vec2,
};
use std::sync::Arc;

/// The passive reference surface appearances.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ItemVariant {
    /// Transparent surface with a reserved transparent border.
    #[default]
    Default,
    /// Transparent surface with the semantic border.
    Outline,
    /// Half-opacity muted surface with a reserved transparent border.
    Muted,
}
/// The reference padding and inter-slot spacing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ItemSize {
    /// Sixteen-point padding and gaps.
    #[default]
    Default,
    /// Twelve-point vertical padding, sixteen horizontal and ten-point gaps.
    Small,
}

#[derive(Clone)]
struct SurfaceTransition {
    from: [Color32; 2],
    to: [Color32; 2],
    started: f64,
    frame: u64,
}
impl SurfaceTransition {
    fn colors(&self, now: f64) -> [Color32; 2] {
        let t = crate::button::css_ease(((now - self.started) / 0.1).clamp(0.0, 1.0) as f32);
        std::array::from_fn(|i| {
            let from = self.from[i].to_array();
            let to = self.to[i].to_array();
            let channels: [u8; 4] = std::array::from_fn(|j| {
                egui::lerp(f32::from(from[j])..=f32::from(to[j]), t).round() as u8
            });
            Color32::from_rgba_premultiplied(channels[0], channels[1], channels[2], channels[3])
        })
    }
}

enum ActionControl<'theme, 'image> {
    Button(Button<'theme, 'image>),
    Icon(IconButton<'theme, 'image>),
}
/// One caller-identified nested control, without application-side effects.
pub struct ItemAction<'theme, 'image> {
    id: IdSalt,
    control: ActionControl<'theme, 'image>,
    enabled: bool,
}
impl<'theme, 'image> ItemAction<'theme, 'image> {
    /// Place a text button in the trailing action group with a stable caller key.
    pub fn new(id: impl egui::AsIdSalt, button: Button<'theme, 'image>) -> Self {
        Self {
            id: IdSalt::new(id),
            control: ActionControl::Button(button),
            enabled: true,
        }
    }
    /// Place a named icon button in the trailing action group with a stable caller key.
    pub fn icon(id: impl egui::AsIdSalt, button: IconButton<'theme, 'image>) -> Self {
        Self {
            id: IdSalt::new(id),
            control: ActionControl::Icon(button),
            enabled: true,
        }
    }
    /// Disable this action without disabling its neighboring actions or status text.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    fn size(&self, ui: &Ui) -> egui::Vec2 {
        match &self.control {
            ActionControl::Button(button) => button.intrinsic_size(ui),
            ActionControl::Icon(button) => button.intrinsic_size(ui),
        }
    }
}

/// A frame-local passive composition borrowing copy and theme, with caller-owned actions.
pub struct Item<'theme, 'image> {
    theme: &'theme Theme,
    id: IdSalt,
    title: Option<&'theme str>,
    description: Option<&'theme str>,
    icon: Option<Image<'image>>,
    actions: Vec<ItemAction<'theme, 'image>>,
    variant: ItemVariant,
    size: ItemSize,
}
impl<'theme, 'image> Item<'theme, 'image> {
    /// Create a passive item with identity stable across content changes and reordering.
    pub fn new(theme: &'theme Theme, id: impl egui::AsIdSalt) -> Self {
        Self {
            theme,
            id: IdSalt::new(id),
            title: None,
            description: None,
            icon: None,
            actions: Vec::new(),
            variant: ItemVariant::Default,
            size: ItemSize::Default,
        }
    }
    /// Set the complete caller-owned title; it remains accessible when clipped.
    pub fn title(mut self, title: &'theme str) -> Self {
        self.title = Some(title);
        self
    }
    /// Set a two-line, visibly elided description with its complete accessible name.
    pub fn description(mut self, description: &'theme str) -> Self {
        self.description = Some(description);
        self
    }
    /// Add a decorative sixteen-point image in the reference's thirty-two-point icon tile.
    pub fn icon(mut self, image: impl Into<Image<'image>>) -> Self {
        self.icon = Some(image.into());
        self
    }
    /// Choose the passive surface appearance without adding focus or activation.
    pub fn variant(mut self, variant: ItemVariant) -> Self {
        self.variant = variant;
        self
    }
    /// Choose reference padding and slot spacing.
    pub fn size(mut self, size: ItemSize) -> Self {
        self.size = size;
        self
    }
    /// Append a stable independent control; the caller interprets its returned response.
    pub fn action(mut self, action: ItemAction<'theme, 'image>) -> Self {
        self.actions.push(action);
        self
    }
    /// Register final hit geometry without handling events, for host-ordered dispatch.
    pub fn register(self, ui: &mut Ui) -> ItemOutput {
        self.draw(ui, false)
    }
    /// Register and handle ordinary egui input once, returning explicit unfaded drawing.
    pub fn show(self, ui: &mut Ui) -> ItemOutput {
        self.draw(ui, true)
    }

    fn draw(self, ui: &mut Ui, handle_input: bool) -> ItemOutput {
        let width = ui.available_width();
        assert!(width.is_finite(), "Item requires finite available width");
        let width = width.max(0.0);
        let (vertical, gap) = match self.size {
            ItemSize::Default => (16.0, 16.0),
            ItemSize::Small => (12.0, 10.0),
        };
        // One-point border is reserved even for transparent variants, like CSS border-box.
        let inset = vec2(17.0, vertical + 1.0);
        let available = (width - 2.0 * inset.x).max(0.0);
        let origin = ui.available_rect_before_wrap().min;
        let id = ui.make_persistent_id(self.id);
        let title_text = self.title.map(collapse);
        let description_text = self.description.map(collapse);
        let minimum = [
            (title_text.as_deref(), true),
            (description_text.as_deref(), false),
        ]
        .into_iter()
        .filter_map(|(value, title)| value.map(|value| (value, title)))
        .flat_map(|(value, title)| value.split_inclusive(' ').map(move |word| (word, title)))
        .map(|(word, title)| {
            text(ui, word, f32::INFINITY, title, false, self.theme)
                .size()
                .x
        })
        .fold(0.0_f32, f32::max);
        let media_width = if self.icon.is_some() { 32.0 } else { 0.0 };
        let has_copy = self.title.is_some() || self.description.is_some();
        let action_sizes: Vec<_> = self.actions.iter().map(|action| action.size(ui)).collect();
        let action_width = action_sizes.iter().map(|size| size.x).sum::<f32>()
            + action_sizes.len().saturating_sub(1) as f32 * 8.0;
        let action_height = action_sizes
            .iter()
            .map(|size| size.y)
            .fold(0.0_f32, f32::max);
        let prefix = media_width
            + if self.icon.is_some() && has_copy {
                gap
            } else {
                0.0
            };
        let copy_minimum = if has_copy { minimum } else { 0.0 };
        let action_gap = if !self.actions.is_empty() && (has_copy || self.icon.is_some()) {
            gap
        } else {
            0.0
        };
        // Keep an action group intact, wrapping it before squeezing copy below its
        // space-delimited minimum. This is an explicit bounded composition, not CSS flex.
        let wrap_actions = !self.actions.is_empty()
            && (has_copy || self.icon.is_some())
            && prefix + copy_minimum + action_gap + action_width > available;
        let content_width = (available
            - prefix
            - if wrap_actions {
                0.0
            } else {
                action_gap + action_width
            })
        .max(copy_minimum);
        let title = title_text
            .as_deref()
            .map(|value| text(ui, value, content_width, true, false, self.theme));
        let description = description_text
            .as_deref()
            .map(|value| text(ui, value, content_width, false, true, self.theme));
        let copy_height = title.as_ref().map_or(0.0, |g| g.size().y)
            + description.as_ref().map_or(0.0, |g| g.size().y)
            + if title.is_some() && description.is_some() {
                4.0
            } else {
                0.0
            };
        let first_height = copy_height
            .max(
                media_width
                    + if self.icon.is_some() && self.description.is_some() {
                        2.0
                    } else {
                        0.0
                    },
            )
            .max(if wrap_actions { 0.0 } else { action_height });
        let height = 2.0 * inset.y
            + first_height
            + if wrap_actions {
                gap + action_height
            } else {
                0.0
            };
        let rect = Rect::from_min_size(origin, vec2(width, height));
        ui.advance_cursor_after_rect(rect);
        let response = ui.interact(rect, id, Sense::hover());
        let mut input_ids = vec![id];
        let clip = ui.clip_rect();
        let mut items = Vec::new();
        let fill = match self.variant {
            ItemVariant::Default | ItemVariant::Outline => Color32::TRANSPARENT,
            ItemVariant::Muted => self.theme.colors.muted.background.gamma_multiply(0.5),
        };
        let border = if self.variant == ItemVariant::Outline {
            self.theme.colors.border
        } else {
            Color32::TRANSPARENT
        };
        let target = [fill, border];
        let now = ui.input(|input| input.time);
        assert!(now.is_finite(), "Item requires finite host time");
        let frame = ui.ctx().cumulative_frame_nr();
        let state_id = id.with("item-surface");
        let mut transition = ui
            .data(|data| data.get_temp::<SurfaceTransition>(state_id))
            .filter(|state| frame <= state.frame.saturating_add(1))
            .unwrap_or(SurfaceTransition {
                from: target,
                to: target,
                started: now,
                frame,
            });
        if transition.to != target {
            transition.from = transition.colors(now);
            transition.to = target;
            transition.started = now;
        }
        transition.frame = frame;
        let [fill, border] = transition.colors(now);
        if transition.from != transition.to
            && now - transition.started < 0.1
            && ui.is_rect_visible(rect)
            && ui.opacity() > 0.0
        {
            ui.ctx().request_repaint();
        }
        ui.data_mut(|data| data.insert_temp(state_id, transition));
        push(&mut items, Shape::rect_filled(rect, 8, fill), clip);
        push(
            &mut items,
            Shape::rect_stroke(rect, 8, Stroke::new(1.0, border), StrokeKind::Inside),
            clip,
        );
        let inner = origin + inset;
        if let Some(mut image) = self.icon {
            let y = if self.description.is_some() {
                2.0
            } else {
                (first_height - 32.0) * 0.5
            };
            let media = Rect::from_min_size(inner + vec2(0.0, y), vec2(32.0, 32.0));
            push(
                &mut items,
                Shape::rect_filled(media, 6, self.theme.colors.muted.background),
                clip,
            );
            push(
                &mut items,
                Shape::rect_stroke(
                    media,
                    6,
                    Stroke::new(1.0, self.theme.colors.border),
                    StrokeKind::Inside,
                ),
                clip,
            );
            if image.image_options().tint == Color32::WHITE {
                image = image.tint(self.theme.colors.background.foreground);
            }
            for shape in crate::button::image_shapes(ui, &image, media.shrink(8.0)) {
                push(&mut items, shape, clip);
            }
        }
        let mut y = inner.y + (first_height - copy_height) * 0.5;
        for (key, original, galley) in [
            ("title", self.title, title),
            ("description", self.description, description),
        ] {
            if let (Some(original), Some(galley)) = (original, galley) {
                let position = pos2(inner.x + prefix, y);
                let text_rect = Rect::from_min_size(position, vec2(content_width, galley.size().y));
                let label = ui.interact(text_rect, id.with(key), Sense::hover());
                input_ids.push(label.id);
                label.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::Role::Label, ui.is_enabled(), original)
                });
                push(
                    &mut items,
                    Shape::galley(
                        position,
                        galley.clone(),
                        self.theme.colors.background.foreground,
                    ),
                    clip,
                );
                y += galley.size().y + 4.0;
            }
        }
        let mut x = if wrap_actions || !has_copy {
            inner.x
                + if wrap_actions {
                    0.0
                } else {
                    prefix + action_gap
                }
        } else {
            inner.x + prefix + content_width + action_gap
        };
        let action_y = inner.y
            + if wrap_actions {
                first_height + gap
            } else {
                (first_height - action_height) * 0.5
            };
        let mut actions = Vec::new();
        let mut ids = std::collections::HashSet::new();
        for (action, size) in self.actions.into_iter().zip(action_sizes) {
            let action_id = id.with(action.id);
            assert!(
                ids.insert(action_id),
                "Item action IDs must be unique within the item"
            );
            let action_rect =
                Rect::from_min_size(pos2(x, action_y + (action_height - size.y) * 0.5), size);
            let mut child = ui.new_child(
                UiBuilder::new()
                    .scope_id(action_id)
                    .max_rect(action_rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );
            input_ids.push(child.unique_id());
            if !action.enabled {
                child.disable();
            }
            let mut output = match action.control {
                ActionControl::Button(button) => {
                    if handle_input {
                        button.show(&mut child)
                    } else {
                        button.register(&mut child)
                    }
                }
                ActionControl::Icon(button) => {
                    if handle_input {
                        button.show(&mut child)
                    } else {
                        button.register(&mut child)
                    }
                }
            };
            if let Some(group) = output.drawing.take() {
                items.push(DrawItem::Group(group));
            }
            // Layout overflow remains reachable to the host scroll area even
            // when the control was fully clipped and produced no drawing.
            ui.expand_to_include_rect(action_rect);
            input_ids.push(output.response.id);
            actions.push(output);
            x += size.x + 8.0;
        }
        let bounds = items.iter().fold(rect, |bounds, item| match item {
            DrawItem::Shape(shape) => bounds.union(shape.shape.visual_bounding_rect()),
            DrawItem::Group(group) => bounds.union(group.bounds()),
        });
        ui.expand_to_include_rect(bounds);
        let bounds = bounds.intersect(clip);
        let drawing = if ui.is_visible() && bounds.is_positive() {
            Some(Group::new(bounds, 1.0, items).expect("finite Item drawing"))
        } else {
            None
        };
        ItemOutput {
            ids: input_ids,
            response,
            actions,
            drawing,
        }
    }
}

/// Passive response, independent action registrations and a complete unfaded drawing.
pub struct ItemOutput {
    /// Every registered root, label, child-UI and action ID for host-owned removal.
    pub ids: Vec<egui::Id>,
    /// The root never activates, focuses or implies a selection state.
    pub response: Response,
    /// The caller routes each event to its matching ButtonOutput and interprets clicks.
    pub actions: Vec<ButtonOutput>,
    /// Includes nested action drawing once; action outputs have no separate drawing.
    pub drawing: Option<Group>,
}
impl ItemOutput {
    /// Submit the complete composition once at the parent painter's opacity.
    pub fn paint(self, painter: &egui::Painter) -> Result<Response, Error> {
        assert!(
            self.response.ctx == *painter.ctx(),
            "Item painter must use the allocating context"
        );
        assert_eq!(
            self.response.layer_id,
            painter.layer_id(),
            "Item painter must use the allocating layer"
        );
        if let Some(drawing) = self.drawing {
            compositing::paint(painter, drawing)?;
        }
        Ok(self.response)
    }
}
fn push(items: &mut Vec<DrawItem>, shape: Shape, clip: Rect) {
    items.push(DrawItem::Shape(egui::epaint::ClippedShape::new(
        clip, shape,
    )));
}
fn collapse(text: &str) -> String {
    text.split([' ', '\t', '\n', '\r', '\u{000c}'])
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}
fn text(
    ui: &Ui,
    value: &str,
    width: f32,
    title: bool,
    balance: bool,
    theme: &Theme,
) -> Arc<Galley> {
    let family = if title {
        TextStyle::Button
    } else {
        TextStyle::Body
    }
    .resolve(ui.style())
    .family;
    let mut job = egui::text::LayoutJob {
        text: value.to_owned(),
        sections: vec![egui::text::LayoutSection {
            leading_space: 0.0,
            byte_range: egui::text::ByteIndex(0)..egui::text::ByteIndex(value.len()),
            format: TextFormat {
                font_id: FontId::new(14.0, family),
                line_height: Some(if title { 19.25 } else { 21.0 }),
                color: if title {
                    theme.colors.background.foreground
                } else {
                    theme.colors.muted.foreground
                },
                ..Default::default()
            },
        }],
        wrap: egui::text::TextWrapping {
            max_width: width,
            ..Default::default()
        },
        break_on_newline: false,
        ..Default::default()
    };
    if value.is_empty() {
        job.wrap.max_rows = 0;
        return ui.painter().layout_job(job);
    }
    let full = ui.painter().layout_job(job.clone());
    if !balance {
        return full;
    }
    // Minimize width while preserving the original line count, then clamp to
    // two rows. This does not implement browser-specific balance or line breaks.
    let mut balanced = job;
    if full.rows.len() > 1 && full.rows.len() <= 6 {
        let mut low = value
            .split_inclusive(' ')
            .map(|word| text(ui, word, f32::INFINITY, false, false, theme).size().x)
            .fold(0.0_f32, f32::max);
        let mut high = width;
        while high - low > 0.25 {
            let mid = (low + high) * 0.5;
            balanced.wrap.max_width = mid;
            if ui.painter().layout_job(balanced.clone()).rows.len() <= full.rows.len() {
                high = mid;
            } else {
                low = mid;
            }
        }
        balanced.wrap.max_width = high;
    }
    balanced.wrap.max_rows = 2;
    ui.painter().layout_job(balanced)
}
