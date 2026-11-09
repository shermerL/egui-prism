//! Controlled boolean checkboxes using the pinned shadcn geometry and behavior.

use crate::{
    compositing::{self, Group, Item},
    egui,
    theme::Theme,
    transition::{animate, color, color_components},
};

/// A labeled checkbox borrowing the caller's boolean and theme.
///
/// Register before dispatching ordered input, then reconstruct with the current
/// value for final layout. Registration and drawing only borrow the value;
/// `CheckboxRegistration::handle` is the only stage that mutates it.
///
/// ```
/// use egui_prism::{checkbox::Checkbox, egui, theme::Theme};
/// let ctx = egui::Context::default();
/// ctx.options_mut(|options| options.ordered_input = true);
/// let theme = Theme::light();
/// let mut previews = false;
/// ctx.run_ui(Default::default(), |ui| {
///     ctx.begin_input();
///     let id = egui::Id::unique("previews");
///     let rect = ui.max_rect();
///     let layout = || egui::UiBuilder::new().max_rect(rect);
///     let registration = Checkbox::new(&theme, &previews, "Show previews")
///         .id(id).register(&mut ui.new_child(layout()));
///     while let Some(event) = ctx.next_input_event() {
///         if event.target == Some(id) {
///             let response = registration.handle(&event, &mut previews);
///             if response.changed() {
///                 // Apply the new value in the application.
///             }
///         }
///         registration.cancel_unfocused();
///     }
///     let output = Checkbox::new(&theme, &previews, "Show previews")
///         .id(id).show(&mut ui.new_child(layout()));
///     // Submit `output.paint(ui.painter())` with an installed compositor.
///     assert!(output.drawing.is_some());
/// }).drop_without_applying_deltas();
/// ```
#[must_use = "call register before input dispatch or show for final drawing"]
pub struct Checkbox<'theme, 'value> {
    theme: &'theme Theme,
    checked: &'value bool,
    label: egui::WidgetText,
    invalid: bool,
    id: Option<egui::Id>,
}

impl<'theme, 'value> Checkbox<'theme, 'value> {
    /// Create a checkbox with a visible label; an empty label is a programmer error.
    pub fn new(
        theme: &'theme Theme,
        checked: &'value bool,
        label: impl Into<egui::WidgetText>,
    ) -> Self {
        let label = label.into();
        assert!(
            !label.text().trim().is_empty(),
            "Checkbox requires a nonempty label"
        );
        Self {
            theme,
            checked,
            label,
            invalid: false,
            id: None,
        }
    }

    /// Use the same identity for registration, handling, and final drawing.
    pub fn id(mut self, id: egui::Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Set invalid semantics, the destructive border, and the focus ring color.
    ///
    /// Checked primary borders take precedence over invalid borders. Invalid
    /// alone does not introduce a ring: its width still requires focus-visible.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }
}

#[derive(Clone)]
struct Interaction {
    keyboard: bool,
    space_armed: bool,
    focused: bool,
    last_frame: u64,
}

impl Interaction {
    fn new(frame: u64) -> Self {
        Self {
            keyboard: true,
            space_armed: false,
            focused: false,
            last_frame: frame,
        }
    }

    // Observe each transfer, not just egui's frame-level gained_focus flag.
    fn observe_focus(&mut self, focused: bool) {
        if focused && !self.focused {
            self.keyboard = true;
        }
        if !focused {
            self.space_armed = false;
        }
        self.focused = focused;
    }
}

fn interaction_id(id: egui::Id) -> egui::Id {
    id.with("checkbox-interaction")
}

fn load_interaction(ctx: &egui::Context, id: egui::Id) -> Interaction {
    let frame = ctx.cumulative_frame_nr();
    let mut state = ctx
        .data(|data| data.get_temp::<Interaction>(interaction_id(id)))
        .unwrap_or_else(|| Interaction::new(frame));
    // A complete absent frame cannot preserve a pending Space activation.
    // Same-batch removal is covered by the host's cancellation contract.
    if frame > state.last_frame.saturating_add(1) {
        state = Interaction::new(frame);
    }
    state.last_frame = frame;
    state
}

fn save_interaction(ctx: &egui::Context, id: egui::Id, state: Interaction) {
    ctx.data_mut(|data| data.insert_temp(interaction_id(id), state));
}

fn usable(response: &egui::Response) -> bool {
    response.enabled() && response.visible()
}

fn focused(response: &egui::Response) -> bool {
    usable(response) && response.has_focus() && response.ctx.input(|input| input.focused)
}

fn refresh_metadata(response: &egui::Response, checked: bool, label: &str, invalid: bool) {
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::Role::CheckBox, response.enabled(), checked, label)
    });
    response.ctx.accesskit_node_builder(response.id, |node| {
        if invalid {
            node.set_invalid(egui::accesskit::Invalid::True);
        } else {
            node.clear_invalid();
        }
    });
}

struct Layout {
    id: egui::Id,
    rect: egui::Rect,
    square: egui::Rect,
    galley: std::sync::Arc<egui::Galley>,
    label_text: String,
    dark: bool,
}

impl Layout {
    fn label_position(&self) -> egui::Pos2 {
        egui::pos2(
            self.square.right() + 8.0,
            self.rect.center().y - self.galley.size().y * 0.5,
        ) - self.galley.rect.min.to_vec2()
    }

    fn outer_bounds(&self) -> egui::Rect {
        // Explicit font sizes and italics can extend ink beyond the line box.
        // Keep that visual overflow without changing the registered hit area.
        self.rect.union(self.square.expand(4.0)).union(
            self.galley
                .mesh_bounds
                .translate(self.label_position().to_vec2()),
        )
    }

    // The native owner remains the square. Public responses describe the row
    // without replacing the owner's rounded hit geometry.
    fn row_response(&self, mut response: egui::Response, clip: egui::Rect) -> egui::Response {
        response.rect = self.rect;
        response.interact_rect = self.rect.intersect(clip);
        response
    }
}

// Lucide Check: M20 6 L9 17 L4 12, viewBox 0 0 24 24, stroke-width 2.
// Two butt-ended segments and disks at the vertices form round caps and a
// round join, without relying on epaint's polyline miter joins. All pieces are
// inside the completed control group before disabled opacity is applied.
fn check_mark(square: egui::Rect, ink: egui::Color32) -> egui::Shape {
    let origin = square.center() - egui::vec2(7.0, 7.0);
    let scale = 14.0 / 24.0;
    let points =
        [(20.0, 6.0), (9.0, 17.0), (4.0, 12.0)].map(|(x, y)| origin + egui::vec2(x, y) * scale);
    let stroke = egui::Stroke::new(2.0 * scale, ink);
    egui::Shape::Vec(vec![
        egui::Shape::line_segment([points[0], points[1]], stroke),
        egui::Shape::line_segment([points[1], points[2]], stroke),
        egui::Shape::circle_filled(points[0], scale, ink),
        egui::Shape::circle_filled(points[1], scale, ink),
        egui::Shape::circle_filled(points[2], scale, ink),
    ])
}

impl Checkbox<'_, '_> {
    fn layout(&self, ui: &mut egui::Ui) -> Layout {
        assert!(
            ui.ctx().options(|options| options.ordered_input),
            "Checkbox requires ordered input"
        );
        let previous_style = ui.style().clone();
        self.theme.apply(ui.style_mut());
        // The labeled fixture uses regular body text, not a button-weight face.
        // Keep caller font families and explicit rich-text formatting.
        if let Some(font) = ui.style_mut().text_styles.get_mut(&egui::TextStyle::Body) {
            font.size = 14.0;
        }
        let label_text = self.label.text().to_owned();
        let mut job = std::sync::Arc::unwrap_or_clone(self.label.clone().into_layout_job(
            ui.style(),
            egui::TextStyle::Body.into(),
            egui::Align::Center,
        ));
        job.wrap.max_width = (ui.available_width() - 16.0 - 8.0).max(0.0);
        for section in &mut job.sections {
            if section.format.line_height.is_none() {
                section.format.line_height = Some(20.0);
            }
        }
        let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
        let size = egui::vec2(16.0 + 8.0 + galley.size().x, 20.0_f32.max(galley.size().y));
        let (automatic_id, rect) = ui.allocate_space(size);
        let square = egui::Rect::from_center_size(
            egui::pos2(rect.left() + 8.0, rect.center().y),
            egui::vec2(16.0, 16.0),
        );
        let dark = ui.visuals().dark_mode;
        ui.set_style(previous_style);
        assert!(
            rect.is_finite() && rect.is_positive() && square.is_finite(),
            "Checkbox requires finite layout bounds"
        );
        Layout {
            id: self.id.unwrap_or(automatic_id),
            rect,
            square,
            galley,
            label_text,
            dark,
        }
    }

    /// Allocate and register interaction, focus policy, and metadata.
    ///
    /// This does not draw, animate, or modify the caller's value. If same-batch
    /// changes affect geometry or eligibility, refresh the authoritative input
    /// layout before routing further events. Remove vanished owners with
    /// `remove_input_widget`; their bound label parts are removed with them.
    pub fn register(self, ui: &mut egui::Ui) -> CheckboxRegistration {
        let layout = self.layout(ui);
        let response = ui.interact(layout.square, layout.id, egui::Sense::click_and_drag());
        ui.ctx().set_input_rounding(layout.id, 4.0);

        // The gap is clickable, but the label is not another Tab stop.
        let label_id = layout.id.with("label-hit");
        let label_rect = egui::Rect::from_min_max(
            egui::pos2(layout.square.right(), layout.rect.top()),
            layout.rect.max,
        );
        ui.interact(label_rect, label_id, egui::Sense::CLICK | egui::Sense::DRAG);
        ui.ctx().set_input_owner(label_id, layout.id);
        if usable(&response) {
            ui.memory_mut(|memory| {
                memory.register_focus_policy(
                    layout.id,
                    response.layer_id,
                    egui::EventFilter {
                        horizontal_arrows: true,
                        vertical_arrows: true,
                        escape: true,
                        ..Default::default()
                    },
                );
            });
        }
        let mut state = load_interaction(ui.ctx(), layout.id);
        state.observe_focus(focused(&response));
        save_interaction(ui.ctx(), layout.id, state);
        refresh_metadata(&response, *self.checked, &layout.label_text, self.invalid);
        CheckboxRegistration {
            response: layout.row_response(response, ui.clip_rect()),
        }
    }

    /// Build final drawing after exhausting the ordered input cursor.
    ///
    /// Replay registration's position, constraints, and identity. This allocates
    /// the row without re-registering interaction or focus policy. Only the
    /// focus-ring shadow transitions; colors, the mark, the border and disabled
    /// opacity all use the caller's current state immediately.
    pub fn show(self, ui: &mut egui::Ui) -> CheckboxOutput {
        let layout = self.layout(ui);
        let response = ui
            .ctx()
            .read_response(layout.id)
            .expect("Checkbox::show requires prior registration");
        assert_eq!(
            response.layer_id,
            ui.layer_id(),
            "Checkbox registration and final drawing must share a layer"
        );
        assert_eq!(
            response.rect, layout.square,
            "Checkbox final layout must match its registered square"
        );
        let mut state = load_interaction(ui.ctx(), layout.id);
        state.observe_focus(focused(&response));
        let focus_visible = state.focused && state.keyboard;
        save_interaction(ui.ctx(), layout.id, state);
        refresh_metadata(&response, *self.checked, &layout.label_text, self.invalid);

        let colors = self.theme.colors;
        let fill = if *self.checked {
            colors.primary.background
        } else if layout.dark {
            colors.input.gamma_multiply(0.3)
        } else {
            egui::Color32::TRANSPARENT
        };
        // Tailwind's checked variant follows invalid, which follows focus-visible.
        let border = if *self.checked {
            colors.primary.background
        } else if self.invalid {
            colors.destructive.background
        } else if focus_visible {
            colors.ring
        } else {
            colors.input
        };
        let ring = if self.invalid {
            colors
                .destructive
                .background
                .gamma_multiply(if layout.dark { 0.4 } else { 0.2 })
        } else {
            colors.ring.gamma_multiply(0.5)
        };
        let [l, a, b, alpha] = color_components(if focus_visible {
            ring
        } else {
            egui::Color32::TRANSPARENT
        });
        // One CSS box-shadow transition, including color changes while focused.
        let [spread, l, a, b, alpha] = animate(
            ui.ctx(),
            layout.id.with("checkbox-focus-shadow"),
            [if focus_visible { 3.0 } else { 0.0 }, l, a, b, alpha],
        );
        let outer_bounds = layout.outer_bounds();
        let clip = ui.clip_rect().intersect(outer_bounds);
        let drawing = if response.visible()
            && ui.is_visible()
            && ui.is_rect_visible(outer_bounds)
            && clip.is_finite()
            && clip.is_positive()
        {
            let square = layout.square;
            let item = |shape| Item::Shape(egui::epaint::ClippedShape::new(clip, shape));
            let mut shapes = vec![item(crate::shadow::outer(ui.ctx(), square, 4))];
            if spread > 0.0 {
                // Outer radius 7 and inner radius 4 at the settled 3pt ring.
                shapes.push(item(egui::Shape::rect_stroke(
                    square.expand(spread),
                    (4.0 + spread).round() as u8,
                    egui::Stroke::new(spread, color([l, a, b, alpha])),
                    egui::StrokeKind::Inside,
                )));
            }
            shapes.push(item(egui::Shape::rect_filled(square, 4, fill)));
            shapes.push(item(egui::Shape::rect_stroke(
                square,
                4,
                egui::Stroke::new(1.0, border),
                egui::StrokeKind::Inside,
            )));
            if *self.checked {
                shapes.push(item(check_mark(square, colors.primary.foreground)));
            }
            let control = Group::new(
                square.expand(4.0),
                if response.enabled() { 1.0 } else { 0.5 },
                shapes,
            )
            .expect("finite Checkbox drawing");
            let label = item(egui::Shape::galley(
                layout.label_position(),
                layout.galley.clone(),
                colors.background.foreground,
            ));
            // Disable the completed control, not each overlapping shape or the
            // label. The submission painter supplies parent opacity only once.
            Some(
                Group::new(outer_bounds, 1.0, vec![Item::Group(control), label])
                    .expect("finite labeled Checkbox"),
            )
        } else {
            None
        };
        CheckboxOutput {
            response: layout.row_response(response, ui.clip_rect()),
            drawing,
        }
    }
}

/// Registered interaction geometry and the complete labeled row response.
///
/// This does not borrow the caller's boolean. Pass its current value to `handle`
/// for each routed event targeted at this owner.
pub struct CheckboxRegistration {
    /// The owner identity and complete labeled row geometry.
    pub response: egui::Response,
}

impl CheckboxRegistration {
    fn row_response(&self, mut response: egui::Response) -> egui::Response {
        response.rect = self.response.rect;
        response.interact_rect = self.response.interact_rect;
        response
    }

    /// Handle exactly one current routed event for this owner.
    ///
    /// Consume `changed()` before advancing the cursor. Space activates once on
    /// release; Enter never activates, matching Radix Checkbox. Do not use
    /// `clicked()` to change the value: it also includes synthesized key clicks.
    pub fn handle(&self, event: &egui::RoutedInput, checked: &mut bool) -> egui::Response {
        assert_eq!(
            event.target,
            Some(self.response.id),
            "Checkbox event target"
        );
        let ctx = &self.response.ctx;
        let mut response = ctx
            .read_response(self.response.id)
            .expect("registered Checkbox response");
        let egui::InputTrigger::Event { event, .. } = &event.trigger else {
            return self.row_response(response);
        };
        let mut state = load_interaction(ctx, response.id);
        state.observe_focus(focused(&response));
        match event {
            egui::Event::PointerButton { pressed: true, .. } => state.keyboard = false,
            egui::Event::Key { pressed: true, .. } => state.keyboard = true,
            egui::Event::WindowFocused(false) => state.observe_focus(false),
            _ => {}
        }
        let active = usable(&response) && ctx.input(|input| input.focused);
        let mut activate = false;
        if active {
            match event {
                egui::Event::PointerButton {
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    ..
                } => {
                    // Short clicks require the dispatcher's authoritative hit.
                    activate = response.clicked_by(egui::PointerButton::Primary);
                    if !activate
                        && !ctx.input(|input| {
                            input.pointer.button_clicked(egui::PointerButton::Primary)
                        })
                        && response.drag_stopped_by(egui::PointerButton::Primary)
                        && response.hovered()
                    {
                        // A long hold may outlive egui's click duration. Do not
                        // let its drag hit bypass a covering click-only widget.
                        // Copy IDs first: both context APIs acquire its lock.
                        let hovered_ids: Vec<egui::Id> = ctx.interaction_snapshot(|snapshot| {
                            snapshot.hovered.iter().copied().collect()
                        });
                        let another_click_hit = hovered_ids.into_iter().any(|id| {
                            id != response.id
                                && ctx.read_response(id).is_some_and(|other| {
                                    other.enabled() && other.sense.senses_click()
                                })
                        });
                        activate = !another_click_hit;
                    }
                }
                egui::Event::Key {
                    key: egui::Key::Space,
                    pressed,
                    repeat,
                    ..
                } if state.focused => {
                    if *pressed {
                        if !*repeat {
                            state.space_armed = true;
                        }
                    } else {
                        activate = state.space_armed;
                        state.space_armed = false;
                    }
                }
                egui::Event::AccessKitActionRequest(request)
                    if request.action == egui::accesskit::Action::Click =>
                {
                    activate = true;
                }
                // Enter (including repeats), arrows and Escape never toggle.
                _ => {}
            }
        } else {
            state.observe_focus(false);
        }
        if activate {
            *checked = !*checked;
            response.mark_changed();
        }
        save_interaction(ctx, response.id, state);
        self.row_response(response)
    }

    /// Cancel pending Space activation when this owner is no longer eligible.
    ///
    /// Call after every relevant loss, including window focus loss and
    /// same-batch disable, hide or focus changes, before routing another event.
    /// For structural removal, surrender this owner's focus before calling:
    /// `read_response` may still find a previous-pass widget. Observe the loss
    /// before any subsequent focus gain. This also observes newly acquired
    /// focus, without requiring a key event at the new owner.
    pub fn cancel_unfocused(&self) {
        let ctx = &self.response.ctx;
        let mut state = load_interaction(ctx, self.response.id);
        let now_focused = ctx
            .read_response(self.response.id)
            .is_some_and(|response| focused(&response));
        state.observe_focus(now_focused);
        save_interaction(ctx, self.response.id, state);
    }
}

/// A response and immutable frame-local drawing, submitted by the caller.
#[must_use = "paint the final output or include its drawing in a parent Group"]
pub struct CheckboxOutput {
    /// Final presentation response; event-local changes come from `handle`.
    pub response: egui::Response,
    /// Independent label and isolated control, absent when not visible.
    pub drawing: Option<Group>,
}

impl CheckboxOutput {
    /// Submit with a same-context/layer painter captured before disabled scopes.
    ///
    /// The painter supplies parent opacity once. Submit in the same frame as
    /// final layout, while the registered square remains authoritative.
    pub fn paint(self, painter: &egui::Painter) -> Result<egui::Response, compositing::Error> {
        assert_eq!(
            &self.response.ctx,
            painter.ctx(),
            "Checkbox painter context"
        );
        assert_eq!(
            self.response.layer_id,
            painter.layer_id(),
            "Checkbox painter layer"
        );
        if let Some(group) = self.drawing {
            let painter = painter.with_clip_rect(group.bounds());
            compositing::paint(&painter, group)?;
        }
        Ok(self.response)
    }
}
