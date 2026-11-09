//! Controlled boolean switches using the pinned shadcn geometry and behavior.

use crate::{
    compositing::{self, Group, Item},
    egui,
    theme::Theme,
};

use crate::transition::{animate, color, color_components};

/// The two sizes supported by the pinned shadcn switch.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SwitchSize {
    /// A 32 by 18.4 point track with a 16 point thumb.
    #[default]
    Default,
    /// A 24 by 14 point track with a 12 point thumb.
    Small,
}

/// A labeled switch borrowing the caller's boolean and theme.
///
/// Register before dispatching input, then reconstruct the switch with the
/// current boolean for final layout and drawing. An explicit ID is recommended
/// when registration and final layout use separately constructed UIs.
///
/// ```
/// use egui_prism::{egui, switch::Switch, theme::Theme};
/// let ctx = egui::Context::default();
/// ctx.options_mut(|options| options.ordered_input = true);
/// let theme = Theme::light();
/// let mut previews = false;
/// ctx.run_ui(Default::default(), |ui| {
///     ctx.begin_input();
///     let id = egui::Id::unique("previews");
///     let rect = ui.max_rect();
///     let layout = || egui::UiBuilder::new().max_rect(rect);
///     let registration = Switch::new(&theme, &previews, "Show previews")
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
///     let output = Switch::new(&theme, &previews, "Show previews")
///         .id(id).show(&mut ui.new_child(layout()));
///     // Submit `output.paint(ui.painter())` with an installed compositor.
///     assert!(output.drawing.is_some());
/// }).drop_without_applying_deltas();
/// ```
#[must_use = "call register before input dispatch or show for final drawing"]
pub struct Switch<'theme, 'value> {
    theme: &'theme Theme,
    checked: &'value bool,
    label: egui::WidgetText,
    size: SwitchSize,
    invalid: bool,
    id: Option<egui::Id>,
}

impl<'theme, 'value> Switch<'theme, 'value> {
    /// Create a switch with a visible label; an empty label is a programmer error.
    pub fn new(
        theme: &'theme Theme,
        checked: &'value bool,
        label: impl Into<egui::WidgetText>,
    ) -> Self {
        let label = label.into();
        assert!(
            !label.text().trim().is_empty(),
            "Switch requires a nonempty label"
        );

        Self {
            theme,
            checked,
            label,
            size: SwitchSize::Default,
            invalid: false,
            id: None,
        }
    }

    /// Use the same identity for registration, handling, and final drawing.
    pub fn id(mut self, id: egui::Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Select the default or small reference geometry.
    pub fn size(mut self, size: SwitchSize) -> Self {
        self.size = size;
        self
    }

    /// Expose invalid semantics without adding reference-absent styling.
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

    /// Compare current focus with our last observed focus, not just egui's
    /// frame-level gained_focus flag. Several transfers can occur in one batch.
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
    id.with("switch-interaction")
}

fn load_interaction(ctx: &egui::Context, id: egui::Id) -> Interaction {
    let frame = ctx.cumulative_frame_nr();
    let mut state = ctx
        .data(|data| data.get_temp::<Interaction>(interaction_id(id)))
        .unwrap_or_else(|| Interaction::new(frame));

    // A control absent for a complete frame must not retain a Space arm.
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
    usable(response) && response.has_focus()
}

fn refresh_metadata(response: &egui::Response, checked: bool, label: &str, invalid: bool) {
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::Role::Switch, response.enabled(), checked, label)
    });

    // Explicitly clear this property too: invalid can change within a batch.
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
    track: egui::Rect,
    diameter: f32,
    galley: std::sync::Arc<egui::Galley>,
    label_text: String,
    dark: bool,
}

impl Layout {
    fn outer_bounds(&self) -> egui::Rect {
        self.rect.union(self.track.expand(4.0))
    }

    /// The native owner remains the actual track. The public response describes
    /// the complete labeled allocation, without changing registered hit geometry.
    fn row_response(&self, mut response: egui::Response, clip: egui::Rect) -> egui::Response {
        response.rect = self.rect;
        response.interact_rect = self.rect.intersect(clip);
        response
    }
}

impl Switch<'_, '_> {
    /// Allocate the same row in both phases, without registering interaction.
    fn layout(&self, ui: &mut egui::Ui) -> Layout {
        assert!(
            ui.ctx().options(|options| options.ordered_input),
            "Switch requires ordered input"
        );

        let previous_style = ui.style().clone();
        self.theme.apply(ui.style_mut());

        let (track_width, track_height, diameter) = match self.size {
            SwitchSize::Default => (32.0, 18.4, 16.0),
            SwitchSize::Small => (24.0, 14.0, 12.0),
        };

        let label_text = self.label.text().to_owned();
        let mut job = std::sync::Arc::unwrap_or_clone(self.label.clone().into_layout_job(
            ui.style(),
            egui::TextStyle::Body.into(),
            egui::Align::Center,
        ));
        job.wrap.max_width = (ui.available_width() - track_width - 8.0).max(0.0);
        for section in &mut job.sections {
            if section.format.line_height.is_none() {
                section.format.line_height = Some(20.0);
            }
        }

        let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
        let size = egui::vec2(
            track_width + 8.0 + galley.size().x,
            20.0_f32.max(galley.size().y),
        );
        let (automatic_id, rect) = ui.allocate_space(size);
        let track = egui::Rect::from_center_size(
            egui::pos2(rect.left() + track_width * 0.5, rect.center().y),
            egui::vec2(track_width, track_height),
        );
        let dark = ui.visuals().dark_mode;

        ui.set_style(previous_style);

        assert!(
            rect.is_finite() && rect.is_positive() && track.is_finite(),
            "Switch requires finite layout bounds"
        );

        Layout {
            id: self.id.unwrap_or(automatic_id),
            rect,
            track,
            diameter,
            galley,
            label_text,
            dark,
        }
    }

    /// Allocate the row and register interaction, focus policy, and metadata.
    ///
    /// This does not draw, animate, or modify the caller's boolean. Call before
    /// dispatching the input cursor. If same-batch changes alter interaction
    /// flags or geometry, the host must make those authoritative before routing
    /// further input.
    pub fn register(self, ui: &mut egui::Ui) -> SwitchRegistration {
        let layout = self.layout(ui);
        let response = ui.interact(layout.track, layout.id, egui::Sense::click_and_drag());

        // The inner capsule axis must survive egui's inset after rounding.
        let center = layout.track.center().y;
        let radius = (center - layout.track.top()).min(layout.track.bottom() - center);
        ui.ctx().set_input_rounding(layout.id, radius);

        // The gap belongs to the label hit region, as in the reference layout.
        // This part has click and drag senses but is not a second tab stop.
        let label_id = layout.id.with("label-hit");
        let label_rect = egui::Rect::from_min_max(
            egui::pos2(layout.track.right(), layout.rect.top()),
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

        SwitchRegistration {
            response: layout.row_response(response, ui.clip_rect()),
        }
    }

    /// Build final drawing after the host has exhausted the input cursor.
    ///
    /// Replay registration's position, layout constraints, size, and stable ID.
    /// This allocates the row but does not re-register interaction, ownership,
    /// rounding, or focus policy. Current hover/focus comes from `read_response`,
    /// and checked state comes from the caller.
    pub fn show(self, ui: &mut egui::Ui) -> SwitchOutput {
        let layout = self.layout(ui);
        let response = ui
            .ctx()
            .read_response(layout.id)
            .expect("Switch::show requires prior registration");

        assert_eq!(
            response.layer_id,
            ui.layer_id(),
            "Switch registration and final drawing must share a layer"
        );
        assert_eq!(
            response.rect, layout.track,
            "Switch final layout must match its registered track"
        );

        let mut state = load_interaction(ui.ctx(), layout.id);
        state.observe_focus(focused(&response));
        let focus_visible = state.focused && state.keyboard;
        save_interaction(ui.ctx(), layout.id, state);

        refresh_metadata(&response, *self.checked, &layout.label_text, self.invalid);

        let [progress] = animate(
            ui.ctx(),
            layout.id.with("switch-thumb"),
            [f32::from(*self.checked)],
        );
        let [focus] = animate(
            ui.ctx(),
            layout.id.with("switch-focus"),
            [f32::from(focus_visible)],
        );
        let [opacity] = animate(
            ui.ctx(),
            layout.id.with("switch-opacity"),
            [if response.enabled() { 1.0 } else { 0.5 }],
        );

        let colors = self.theme.colors;
        let target_fill = if *self.checked {
            colors.primary.background
        } else if layout.dark {
            colors.input.gamma_multiply(0.8)
        } else {
            colors.input
        };
        let fill = color(animate(
            ui.ctx(),
            layout.id.with("switch-fill"),
            color_components(target_fill),
        ));

        let outer_bounds = layout.outer_bounds();
        // An infinite source clip is legal for a Ui, but not for the immutable
        // compositing drawing. Intersect with finite outer group bounds first.
        let clip = ui.clip_rect().intersect(outer_bounds);

        let drawing = if response.visible()
            && ui.is_visible()
            && ui.is_rect_visible(layout.rect)
            && clip.is_finite()
            && clip.is_positive()
        {
            let track = layout.track;
            let rect = layout.rect;
            let diameter = layout.diameter;

            let thumb = if layout.dark {
                if *self.checked {
                    colors.primary.foreground
                } else {
                    colors.background.foreground
                }
            } else {
                colors.background.background
            };

            let center = egui::pos2(
                track.left() + 1.0 + diameter * 0.5 + (diameter - 2.0) * progress,
                track.center().y,
            );
            let item = |shape| Item::Shape(egui::epaint::ClippedShape::new(clip, shape));

            let mut shapes = vec![item(
                egui::epaint::Shadow {
                    offset: [0, 1],
                    blur: 2,
                    spread: 0,
                    color: egui::Color32::from_black_alpha(13),
                }
                .as_shape(track, 255)
                .into(),
            )];

            if focus > 0.0 {
                shapes.push(item(egui::Shape::rect_stroke(
                    track.expand(1.5 * focus),
                    255,
                    egui::Stroke::new(3.0 * focus, colors.ring.gamma_multiply(0.5 * focus)),
                    egui::StrokeKind::Middle,
                )));
            }

            shapes.push(item(egui::Shape::rect_filled(track, 255, fill)));
            shapes.push(item(egui::Shape::circle_filled(
                center,
                diameter * 0.5,
                thumb,
            )));

            if focus > 0.0 {
                shapes.push(item(egui::Shape::rect_stroke(
                    track,
                    255,
                    egui::Stroke::new(1.0, colors.ring.gamma_multiply(focus)),
                    egui::StrokeKind::Inside,
                )));
            }

            let control =
                Group::new(track.expand(4.0), opacity, shapes).expect("finite Switch drawing");
            let label = item(egui::Shape::galley(
                egui::pos2(
                    track.right() + 8.0,
                    rect.center().y - layout.galley.size().y * 0.5,
                ) - layout.galley.rect.min.to_vec2(),
                layout.galley.clone(),
                colors.background.foreground,
            ));

            // Only the control receives the switch's disabled opacity.
            // Parent opacity is supplied once by the submission painter.
            Some(
                Group::new(outer_bounds, 1.0, vec![Item::Group(control), label])
                    .expect("finite labeled Switch"),
            )
        } else {
            None
        };

        SwitchOutput {
            response: layout.row_response(response, ui.clip_rect()),
            drawing,
        }
    }
}

/// Registered interaction geometry and the full labeled row response.
///
/// This does not borrow the caller's boolean. Pass its current value to `handle`
/// for each event targeted at this registration.
pub struct SwitchRegistration {
    /// The owner ID and complete labeled row geometry.
    pub response: egui::Response,
}

impl SwitchRegistration {
    fn row_response(&self, mut response: egui::Response) -> egui::Response {
        response.rect = self.response.rect;
        response.interact_rect = self.response.interact_rect;
        response
    }

    /// Handle exactly one current routed event for this owner.
    ///
    /// Consume `changed()` on the returned response before advancing the cursor.
    /// Keyboard and accessibility activation are handled explicitly; `clicked()`
    /// is deliberately not used because it includes synthesized primary clicks.
    pub fn handle(&self, event: &egui::RoutedInput, checked: &mut bool) -> egui::Response {
        assert_eq!(event.target, Some(self.response.id), "Switch event target");

        let ctx = &self.response.ctx;
        let mut response = ctx
            .read_response(self.response.id)
            .expect("registered Switch response");
        let egui::InputTrigger::Event { event, .. } = &event.trigger else {
            return self.row_response(response);
        };
        let mut state = load_interaction(ctx, response.id);
        state.observe_focus(focused(&response));

        // A focus gain defaults to keyboard modality. The actual event can
        // override that immediately, including pointer-originated focus gains.
        match event {
            egui::Event::PointerButton { pressed: true, .. } => {
                state.keyboard = false;
            }
            egui::Event::Key { pressed: true, .. } => {
                state.keyboard = true;
            }
            egui::Event::WindowFocused(false) => {
                state.observe_focus(false);
            }
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
                    // Short clicks must be accepted by egui's authoritative
                    // click hit. Stale capture/hover must not revive a rejected
                    // short click under a cover.
                    activate = response.clicked_by(egui::PointerButton::Primary);

                    if !activate
                        && !ctx.input(|input| {
                            input.pointer.button_clicked(egui::PointerButton::Primary)
                        })
                        && response.drag_stopped_by(egui::PointerButton::Primary)
                        && response.hovered()
                    {
                        // On a no-click/no-drag release, hover includes both
                        // authoritative hits. A pure-click cover can coexist
                        // there with this owner's drag hit.
                        //
                        // Copy IDs before read_response: both APIs lock the
                        // context, so they must not be nested.
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
                    key: egui::Key::Enter,
                    pressed: true,
                    ..
                } if state.focused => {
                    // Enter intentionally activates on keydown and repeats.
                    activate = true;
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
                    // Routing already selected and asserted this owner.
                    activate = true;
                }

                // Arrow keys and Escape stay with the focused switch through
                // its registered focus policy; they do not toggle its value.
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

    /// Drop a pending Space activation if this control is no longer eligible.
    ///
    /// The host must call this after every relevant loss, including window
    /// focus loss and same-batch disable/hide/focus changes, before routing the
    /// next event. For structural removal, surrender this owner's focus before
    /// calling: `read_response` may still find a previous-pass widget.
    ///
    /// This also observes newly acquired focus, so modality does not depend on
    /// receiving a subsequent key event at the new owner.
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
pub struct SwitchOutput {
    /// Final authoritative response with the complete labeled row rectangle.
    ///
    /// Event-local changes are returned by `SwitchRegistration::handle`.
    pub response: egui::Response,
    /// Unfaded label and isolated control, absent when not visible.
    pub drawing: Option<Group>,
}

impl SwitchOutput {
    /// Submit with a same-context/layer painter captured before disabled scopes.
    ///
    /// The painter supplies parent opacity once. Submit in the same frame as
    /// final layout, while this owner's registered track remains authoritative.
    pub fn paint(self, painter: &egui::Painter) -> Result<egui::Response, compositing::Error> {
        assert_eq!(&self.response.ctx, painter.ctx(), "Switch painter context");
        assert_eq!(
            self.response.layer_id,
            painter.layer_id(),
            "Switch painter layer"
        );

        if let Some(group) = self.drawing {
            let painter = painter.with_clip_rect(group.bounds());
            compositing::paint(&painter, group)?;
        }

        Ok(self.response)
    }
}
