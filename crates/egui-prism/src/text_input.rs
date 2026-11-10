//! Labeled single-line editing with caller-owned values and validation.
//!
//! Labels and errors remain ordinary text sources. Only the editor is included
//! in the isolated drawing group.

use crate::{
    compositing::{self, Group, Item},
    egui,
    theme::Theme,
    transition,
};

const HEIGHT: f32 = 36.0;
// The text inset includes the 1px border and 12px CSS padding.
const HORIZONTAL_INSET: f32 = 13.0;
const VERTICAL_INSET: i8 = 5;
const VISUAL_OUTSET: f32 = 5.0;

/// A labeled single-line editor borrowing the caller's value for one invocation.
///
/// Use a stable UI scope or an explicit application-unique ID. The native
/// TextEdit retains selection, undo and IME state under that ID.
///
/// ```
/// use egui_prism::{egui, text_input::TextInput, theme::Theme};
/// let theme = Theme::light();
/// let mut name = String::new();
/// egui::__run_test_ui(|ui| {
///     let painter = ui.painter().clone();
///     let output = TextInput::new(&theme, &mut name, "Name").show(ui);
///     // Install a compositing host before submitting in an application.
///     let _result = output.paint(&painter);
/// });
/// ```
///
/// The host supplies phase-appropriate input before calling `show`. This widget
/// does not replay events, register a separate input target, or change focus.
#[must_use = "Call input.show(ui), then submit its editor group during final painting"]
pub struct TextInput<'theme, 'value> {
    theme: &'theme Theme,
    value: &'value mut String,
    label: String,
    hint: String,
    error: Option<String>,
    id: Option<egui::Id>,
}

impl<'theme, 'value> TextInput<'theme, 'value> {
    /// Create a labeled editor.
    pub fn new(theme: &'theme Theme, value: &'value mut String, label: impl Into<String>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "TextInput requires a nonempty label"
        );
        Self {
            theme,
            value,
            label,
            hint: String::new(),
            error: None,
            id: None,
        }
    }

    /// Show a clipped, non-ellipsized hint while the value is empty.
    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = hint.into();
        self
    }

    /// Display validation supplied by the caller without validating the value.
    pub fn error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    /// Retain native editing state across layout moves.
    pub fn id(mut self, id: egui::Id) -> Self {
        self.id = Some(id);
        self
    }
}

/// Interaction, metadata sources and deferred editor painting from one invocation.
#[must_use = "Submit the editor group during the host's final paint phase"]
pub struct TextInputOutput {
    /// The native editor response, linked to the visible label.
    pub response: egui::Response,
    /// The visible label, outside the editor's disabled-opacity group.
    pub label: egui::Response,
    /// The optional validation message, outside the editor's opacity group.
    pub error: Option<egui::Response>,
    /// Frame-local editor drawing, absent when there is nothing visible to submit.
    pub drawing: Option<Group>,
}

impl TextInputOutput {
    /// Submit this invocation's editor group on its original context and layer.
    ///
    /// Use a parent painter, not a painter carrying egui's disabled attenuation.
    /// The group already contains the editor's disabled opacity. Any intentional
    /// opacity on the supplied parent painter is applied once by the compositor.
    ///
    /// Submit only the final-paint invocation's output. Label and error text are
    /// ordinary egui sources and follow the host's existing text-source handling.
    pub fn paint(self, painter: &egui::Painter) -> Result<egui::Response, compositing::Error> {
        assert_eq!(painter.ctx(), &self.response.ctx, "input painter context");
        assert_eq!(
            painter.layer_id(),
            self.response.layer_id,
            "input painter layer"
        );
        if let Some(group) = self.drawing {
            compositing::paint(painter, group)?;
        }
        Ok(self.response)
    }
}

impl TextInput<'_, '_> {
    /// Run the native editor synchronously and retain only its drawing output.
    ///
    /// No copy of the caller's value is used for editing. In particular, disabled
    /// fields pass `interactive(false)` on their first disabled invocation.
    pub fn show(self, ui: &mut egui::Ui) -> TextInputOutput {
        let id = self
            .id
            .unwrap_or_else(|| ui.next_auto_id().with("text-input"));

        // This breakpoint is deliberately independent of the column width.
        let desktop = ui
            .ctx()
            .input(|input| input.viewport_rect().width() >= 768.0);
        let (font_size, line_height) = if desktop { (14.0, 20.0) } else { (16.0, 24.0) };

        // Border and fixed padding impose the same 26-point minimum as CSS.
        let width = ui.available_width().max(2.0 * HORIZONTAL_INSET);
        assert!(
            width.is_finite(),
            "TextInput needs a finite available width"
        );
        ui.vertical(|ui| {
            let enabled = ui.is_enabled();
            let context = ui.ctx().clone();
            let layer = ui.layer_id();

            self.theme.apply(ui.style_mut());

            let label = ui.add(
                egui::Label::new(
                    egui::RichText::new(&self.label).color(self.theme.colors.background.foreground),
                )
                .wrap_mode(egui::TextWrapMode::Wrap),
            );

            let mut font = egui::TextStyle::Body.resolve(ui.style());
            font.size = font_size;

            let foreground = animated_color(
                &context,
                id.with("foreground"),
                self.theme.colors.background.foreground,
            );
            let placeholder_color = animated_color(
                &context,
                id.with("placeholder"),
                self.theme.colors.muted.foreground,
            );

            let (native, captured) = ui
                .scope(|ui| {
                    // Capture unfaded colors; opacity belongs to the final group.
                    ui.set_opacity(1.0);

                    let row_height = ui.fonts_mut(|fonts| fonts.row_height(&font));
                    {
                        let style = ui.style_mut();
                        style.spacing.interact_size = egui::vec2(0.0, HEIGHT);
                        style.spacing.text_edit_width = width;
                        style.spacing.extra_text_line_spacing = line_height - row_height;
                        style.visuals.override_text_color = None;
                        style.visuals.selection.bg_fill = self.theme.colors.primary.background;
                        style.visuals.selection.stroke =
                            egui::Stroke::new(1.0, self.theme.colors.primary.foreground);
                        style.visuals.text_cursor.stroke.color = foreground;
                    }

                    let start = context.graphics_mut(|graphics| graphics.entry(layer).next_idx());

                    // Do not use native hint_text here: this builder truncates
                    // single-line hint atoms and explicitly delays their first
                    // edited galley by one invocation.
                    let native = egui::TextEdit::singleline(self.value)
                        .id(id)
                        .interactive(enabled)
                        .font(font.clone())
                        .text_color(foreground)
                        .align(egui::Align2::LEFT_CENTER)
                        .frame(egui::Frame::NONE.inner_margin(egui::Margin::symmetric(
                            HORIZONTAL_INSET as i8,
                            VERTICAL_INSET,
                        )))
                        .min_size(egui::vec2(2.0 * HORIZONTAL_INSET, HEIGHT))
                        .desired_width(width)
                        .clip_text(true)
                        .show(ui);

                    // Nothing else is painted between the two range boundaries.
                    // Move complete entries, including their clip and deferred
                    // transform; leave their original slots as Noop.
                    let captured = context.graphics_mut(|graphics| {
                        let list = graphics.entry(layer);
                        let end = list.next_idx();
                        let mut captured = Vec::with_capacity(end.0 - start.0);
                        for index in start.0..end.0 {
                            list.mutate_shape(egui::layers::ShapeIdx(index), |entry| {
                                let replacement = egui::epaint::ClippedShape::new(
                                    entry.clip_rect,
                                    egui::Shape::Noop,
                                );
                                captured.push(std::mem::replace(entry, replacement));
                            });
                        }
                        captured
                    });

                    (native, captured)
                })
                .inner;

            let galley_pos = native.galley_pos;
            let response = native.response.response.labelled_by(label.id);
            let rect = response.rect;
            // An empty native galley may have a zero-width atom rect. The
            // placeholder occupies the full CSS content box independently.
            let placeholder_clip =
                rect.shrink2(egui::vec2(HORIZONTAL_INSET, f32::from(VERTICAL_INSET)));

            let focused = enabled && response.has_focus();
            let invalid = self.error.is_some();
            let radius = self.theme.metrics.corner_radius;

            // Border color is not part of transition-[color,box-shadow].
            let border_color = if invalid {
                self.theme.colors.destructive.background
            } else if focused {
                self.theme.colors.ring
            } else {
                self.theme.colors.input
            };

            let ring_color = if invalid {
                self.theme
                    .colors
                    .destructive
                    .background
                    .gamma_multiply(if self.theme.dark { 0.4 } else { 0.2 })
            } else {
                self.theme.colors.ring.gamma_multiply(0.5)
            };
            let ring_components = transition::color_components(if focused {
                ring_color
            } else {
                egui::Color32::TRANSPARENT
            });
            let ring = transition::animate(
                &context,
                id.with("focus-shadow"),
                [
                    ring_components[0],
                    ring_components[1],
                    ring_components[2],
                    ring_components[3],
                    if focused { 3.0 } else { 0.0 },
                ],
            );
            let ring_color = transition::color([ring[0], ring[1], ring[2], ring[3]]);
            let ring_width = ring[4];

            let bounds = rect.expand(VISUAL_OUTSET);
            let clip = ui.clip_rect().intersect(bounds);
            let visible = ui.is_visible() && ui.is_rect_visible(bounds) && clip.is_positive();

            let group = if visible {
                let mut items = Vec::with_capacity(captured.len() + 5);
                let mut push = |shape| {
                    items.push(Item::Shape(egui::epaint::ClippedShape::new(clip, shape)));
                };

                // An exterior-only mesh avoids darkening the transparent input
                // interior, which a filled epaint shadow would do.
                push(crate::shadow::outer(&context, rect, radius));
                push(egui::Shape::rect_filled(
                    rect,
                    radius,
                    if self.theme.dark {
                        self.theme.colors.input.gamma_multiply(0.30)
                    } else {
                        egui::Color32::TRANSPARENT
                    },
                ));
                push(egui::Shape::rect_stroke(
                    rect,
                    radius,
                    egui::Stroke::new(ring_width, ring_color),
                    egui::StrokeKind::Outside,
                ));
                push(egui::Shape::rect_stroke(
                    rect,
                    radius,
                    egui::Stroke::new(1.0, border_color),
                    egui::StrokeKind::Inside,
                ));

                // Inspect the value after native event handling. This avoids
                // showing a stale hint on the first typed or pasted character.
                if self.value.is_empty() && !self.hint.is_empty() && placeholder_clip.is_positive()
                {
                    let mut job = egui::text::LayoutJob::simple_singleline(
                        self.hint.clone(),
                        font,
                        placeholder_color,
                    );
                    job.keep_trailing_whitespace = true;
                    for section in &mut job.sections {
                        section.format.line_height = Some(line_height);
                    }
                    let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
                    items.push(Item::Shape(egui::epaint::ClippedShape::new(
                        placeholder_clip.intersect(clip),
                        egui::Shape::galley(galley_pos, galley, placeholder_color),
                    )));
                }

                // Keep selection, caret and IME drawing exactly as generated by
                // the native editor. Only trim to the explicit group extent.
                for mut shape in captured {
                    shape.clip_rect = shape.clip_rect.intersect(bounds);
                    items.push(Item::Shape(shape));
                }

                Some(
                    Group::new(bounds, if enabled { 1.0 } else { 0.5 }, items)
                        .expect("native TextEdit produced unsupported group drawing"),
                )
            } else {
                None
            };

            context.accesskit_node_builder(response.id, |node| {
                if !self.hint.is_empty() {
                    node.set_placeholder(self.hint.clone());
                }
            });
            let error = self.error.map(|message| {
                let error_response = ui.add(
                    egui::Label::new(
                        egui::RichText::new(&message)
                            .small()
                            .color(self.theme.colors.destructive.background),
                    )
                    .wrap_mode(egui::TextWrapMode::Wrap),
                );
                context.accesskit_node_builder(response.id, |node| {
                    node.set_description(message);
                    node.set_invalid(egui::accesskit::Invalid::True);
                });
                error_response
            });

            TextInputOutput {
                response,
                label,
                error,
                drawing: group,
            }
        })
        .inner
    }
}

fn animated_color(context: &egui::Context, id: egui::Id, target: egui::Color32) -> egui::Color32 {
    transition::color(transition::animate(
        context,
        id,
        transition::color_components(target),
    ))
}
