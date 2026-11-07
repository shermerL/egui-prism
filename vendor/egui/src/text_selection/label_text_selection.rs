use alloc::sync::Arc;

use emath::TSTransform;

use crate::{
    Context, CursorIcon, Event, Galley, Id, LayerId, Plugin, Pos2, Rect, Response, Ui,
    ViewportIdMap, layers::ShapeIdx, text::CCursor, text_selection::CCursorRange,
};

use super::{
    TextCursorState,
    text_cursor_state::{
        GranularDragSelect, SelectGranularity, cursor_rect, extend_granular_select, select_unit_at,
    },
    visuals::{RowVertexIndices, paint_text_selection},
};

/// Turn on to help debug this
const DEBUG: bool = false; // Don't merge `true`!

/// One end of a text selection, inside any widget.
#[derive(Clone, Copy)]
struct WidgetTextCursor {
    widget_id: Id,
    ccursor: CCursor,

    /// Last known screen position
    pos: Pos2,
}

impl WidgetTextCursor {
    fn new(
        widget_id: Id,
        cursor: impl Into<CCursor>,
        global_from_galley: TSTransform,
        galley: &Galley,
    ) -> Self {
        let ccursor = cursor.into();
        let pos = global_from_galley * pos_in_galley(galley, ccursor);
        Self {
            widget_id,
            ccursor,
            pos,
        }
    }
}

fn pos_in_galley(galley: &Galley, ccursor: CCursor) -> Pos2 {
    galley.pos_from_cursor(ccursor).center()
}

impl core::fmt::Debug for WidgetTextCursor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let Self {
            widget_id,
            ccursor,
            pos: _,
        } = self;
        f.debug_struct("WidgetTextCursor")
            .field("widget_id", &widget_id.short_debug_format())
            .field("ccursor", &ccursor.index)
            .finish_non_exhaustive()
    }
}

/// A label selection that started with a double- or triple-click,
/// remembered so that dragging and shift-clicking extend it by whole words or lines,
/// while keeping the originally clicked word or line selected.
///
/// Both ends of the anchor word/line are always in the same widget,
/// but the drag may extend the selection into other widgets.
#[derive(Clone, Copy, Debug)]
struct GranularDrag {
    /// The unit of the current gesture (e.g. a shift-click extends by characters).
    granularity: SelectGranularity,

    /// Start of the word/line that was initially clicked.
    anchor_min: WidgetTextCursor,

    /// End of the word/line that was initially clicked.
    anchor_max: WidgetTextCursor,
}

#[derive(Clone, Copy, Debug)]
struct CurrentSelection {
    /// The selection is in this layer.
    ///
    /// This is to constrain a selection to a single Window.
    pub layer_id: LayerId,

    /// When selecting with a mouse, this is where the mouse was released.
    /// When moving with e.g. shift+arrows, this is what moves.
    /// Note that the two ends can come in any order, and also be equal (no selection).
    pub primary: WidgetTextCursor,

    /// When selecting with a mouse, this is where the mouse was first pressed.
    /// This part of the cursor does not move when shift is down.
    pub secondary: WidgetTextCursor,
}

/// Handles text selection in labels (NOT in [`crate::TextEdit`])s.
///
/// Each viewport has its own state, because viewports are rendered in separate passes.
#[derive(Clone, Debug, Default)]
pub struct LabelSelectionState {
    states: ViewportIdMap<ViewportLabelSelectionState>,
}

// Immutable text sources for this pass's selection output. Geometry ownership and
// hit testing stay in the existing widget tree; no Ui or input state is retained.
#[derive(Clone, Debug)]
struct LabelText {
    id: Id,
    // Widget re-registration revokes the old source until its label renews it.
    registered: bool,
    galley: Arc<Galley>,
    global_from_galley: TSTransform,
    multi_widget_text_select: bool,
    clip_rect: Rect,
}

/// Text selection state for all labels in one viewport.
#[derive(Clone, Debug)]
struct ViewportLabelSelectionState {
    /// The current selection, if any.
    selection: Option<CurrentSelection>,
    ordered_labels: Vec<LabelText>,

    selection_bbox_last_frame: Rect,
    selection_bbox_this_frame: Rect,

    /// Any label hovered this frame?
    any_hovered: bool,

    /// Are we in drag-to-select state?
    is_dragging: bool,

    /// Set if the current selection started with a double- or triple-click,
    /// so that dragging extends the selection by whole words or lines.
    granular_drag: Option<GranularDrag>,

    /// Have we reached the widget containing the primary selection?
    has_reached_primary: bool,

    /// Have we reached the widget containing the secondary selection?
    has_reached_secondary: bool,

    /// Accumulated text to copy.
    text_to_copy: String,
    last_copied_galley_rect: Option<Rect>,

    /// Has the selection changed since it was last reported with [`crate::OutputCommand::TextSelectionSettled`]?
    selection_is_dirty: bool,

    /// Should we report the selection with [`crate::OutputCommand::TextSelectionSettled`] at the end of this pass?
    report_selection: bool,

    /// Painted selections this frame.
    ///
    /// Kept so we can undo a bad selection visualization if we don't see both ends of the selection this frame.
    painted_selections: Vec<(ShapeIdx, Vec<RowVertexIndices>)>,
}

impl Default for ViewportLabelSelectionState {
    fn default() -> Self {
        Self {
            selection: Default::default(),
            ordered_labels: Vec::new(),
            selection_bbox_last_frame: Rect::NOTHING,
            selection_bbox_this_frame: Rect::NOTHING,
            any_hovered: Default::default(),
            is_dragging: Default::default(),
            granular_drag: Default::default(),
            has_reached_primary: Default::default(),
            has_reached_secondary: Default::default(),
            text_to_copy: Default::default(),
            last_copied_galley_rect: Default::default(),
            selection_is_dirty: Default::default(),
            report_selection: Default::default(),
            painted_selections: Default::default(),
        }
    }
}

impl Plugin for LabelSelectionState {
    fn debug_name(&self) -> &'static str {
        "LabelSelectionState"
    }

    fn on_begin_pass(&mut self, ui: &mut Ui) {
        self.states
            .entry(ui.ctx().viewport_id())
            .or_default()
            .on_begin_pass(ui);
    }

    fn on_end_pass(&mut self, ui: &mut Ui) {
        let viewport_id = ui.ctx().viewport_id();
        let state = self.states.entry(viewport_id).or_default();
        state.on_end_pass(ui);
        if !state.is_active() {
            self.states.remove(&viewport_id);
        }
    }

    fn output_hook(&mut self, ctx: &Context, output: &mut crate::FullOutput) {
        if ctx.options(|options| options.ordered_input) {
            self.states
                .retain(|id, _| output.viewport_output.contains_key(id));
        }
    }
}

impl LabelSelectionState {
    pub(crate) fn invalidate_input_text(ctx: &Context, id: Id) {
        let plugin = ctx.plugin::<Self>();
        if let Some(state) = plugin.lock().states.get_mut(&ctx.viewport_id())
            && let Some(source) = state
                .ordered_labels
                .iter_mut()
                .find(|source| source.id == id)
        {
            source.registered = false;
        }
    }

    pub(crate) fn clear_input_text(ctx: &Context, viewport: crate::ViewportId) {
        let plugin = ctx.plugin::<Self>();
        if let Some(state) = plugin.lock().states.get_mut(&viewport) {
            state.ordered_labels.clear();
        }
    }

    pub(crate) fn finish_input_event(ctx: &Context) {
        let plugin = ctx.plugin::<Self>();
        let mut plugin = plugin.lock();
        if let Some(state) = plugin.states.get_mut(&ctx.viewport_id()) {
            state.finish_input_event(ctx);
        }
    }

    pub(crate) fn begin_input_event(ctx: &Context) {
        let plugin = ctx.plugin::<Self>();
        if let Some(state) = plugin.lock().states.get_mut(&ctx.viewport_id()) {
            state.on_pointer_press(ctx);
        }
    }

    pub(crate) fn prepare_input_paint(ctx: &Context) {
        let plugin = ctx.plugin::<Self>();
        let mut plugin = plugin.lock();
        if let Some(state) = plugin.states.get_mut(&ctx.viewport_id()) {
            state.has_reached_primary = false;
            state.has_reached_secondary = false;
            state.any_hovered = false;
            state.selection_bbox_this_frame = Rect::NOTHING;
            state.painted_selections.clear();
        }
    }

    /// Is there a label text selection in any viewport?
    pub fn has_selection(&self) -> bool {
        self.states
            .values()
            .any(ViewportLabelSelectionState::has_selection)
    }

    /// Clear all label text selections in all viewports.
    pub fn clear_selection(&mut self) {
        self.states.clear();
    }

    /// Handle text selection state for a label or similar widget.
    /// This also takes care of painting the galley.
    pub fn label_text_selection(
        ui: &Ui,
        response: &Response,
        galley_pos: Pos2,
        mut galley: Arc<Galley>,
        fallback_color: epaint::Color32,
        underline: epaint::Stroke,
    ) {
        let plugin = ui.ctx().plugin::<Self>();
        let mut plugin = plugin.lock();
        let state = plugin.states.entry(ui.ctx().viewport_id()).or_default();
        let new_vertex_indices = state.on_label(ui, response, galley_pos, &mut galley);

        let shape_idx = ui.painter().add(
            epaint::TextShape::new(galley_pos, galley, fallback_color).with_underline(underline),
        );

        if !new_vertex_indices.is_empty() {
            state
                .painted_selections
                .push((shape_idx, new_vertex_indices));
        }
    }
}

impl ViewportLabelSelectionState {
    fn ordered_selected_text(&mut self, ctx: &Context) -> Option<String> {
        let selection = self.selection?;
        let sources: Vec<_> = self
            .ordered_labels
            .iter()
            .filter(|source| {
                source.registered
                    && ctx.pass_state(|pass| {
                        pass.widgets.get(source.id).is_some_and(|widget| {
                            widget.enabled
                                && widget.visible
                                && widget.layer_id == selection.layer_id
                        })
                    })
                    && ctx.memory(|memory| memory.allows_interaction(selection.layer_id))
            })
            .cloned()
            .collect();
        let primary = sources
            .iter()
            .position(|source| source.id == selection.primary.widget_id)?;
        let secondary = sources
            .iter()
            .position(|source| source.id == selection.secondary.widget_id)?;
        self.text_to_copy.clear();
        self.last_copied_galley_rect = None;
        for (index, source) in sources
            .iter()
            .enumerate()
            .take(primary.max(secondary) + 1)
            .skip(primary.min(secondary))
        {
            let range = if primary == secondary {
                CCursorRange::two(selection.primary.ccursor, selection.secondary.ccursor)
            } else {
                let first = if index == primary.min(secondary) {
                    if primary < secondary {
                        selection.primary.ccursor
                    } else {
                        selection.secondary.ccursor
                    }
                } else {
                    source.galley.begin()
                };
                let last = if index == primary.max(secondary) {
                    if primary > secondary {
                        selection.primary.ccursor
                    } else {
                        selection.secondary.ccursor
                    }
                } else {
                    source.galley.end()
                };
                CCursorRange::two(first, last)
            };
            let rect =
                source.global_from_galley * Rect::from_min_size(Pos2::ZERO, source.galley.size());
            self.copy_text(rect, &source.galley, &range);
        }
        Some(core::mem::take(&mut self.text_to_copy))
    }

    fn finish_selection_key_event(&mut self, ctx: &Context) {
        let Some(selection) = &mut self.selection else {
            return;
        };
        let Some(primary) = self
            .ordered_labels
            .iter()
            .position(|source| source.id == selection.primary.widget_id)
        else {
            return;
        };
        let Some(secondary) = self
            .ordered_labels
            .iter()
            .position(|source| source.id == selection.secondary.widget_id)
        else {
            return;
        };
        let source = &self.ordered_labels[primary];
        if !ctx.pass_state(|pass| {
            pass.widgets
                .get(source.id)
                .is_some_and(|widget| widget.enabled && widget.visible)
        }) || !ctx.memory(|memory| memory.allows_interaction(selection.layer_id))
        {
            return;
        }
        let other = if primary == secondary {
            selection.secondary.ccursor
        } else if primary < secondary {
            source.galley.end()
        } else {
            source.galley.begin()
        };
        let previous = CCursorRange {
            primary: selection.primary.ccursor,
            secondary: other,
            h_pos: None,
        };
        let mut range = previous;
        if process_selection_key_events(ctx, &source.galley, source.id, &mut range) {
            let changed = range != previous;
            if changed && (range.primary != previous.primary || !source.multi_widget_text_select) {
                selection.primary = WidgetTextCursor::new(
                    source.id,
                    range.primary,
                    source.global_from_galley,
                    &source.galley,
                );
            }
            if changed
                && (range.secondary != previous.secondary || !source.multi_widget_text_select)
            {
                selection.secondary = WidgetTextCursor::new(
                    source.id,
                    range.secondary,
                    source.global_from_galley,
                    &source.galley,
                );
            }
            self.selection_is_dirty |= changed;
            self.granular_drag = None;
        }
    }

    fn finish_input_event(&mut self, ctx: &Context) {
        self.ordered_labels.retain(|source| {
            source.registered && ctx.pass_state(|pass| pass.widgets.contains(source.id))
        });
        if self.selection.is_some_and(|selection| {
            [selection.primary.widget_id, selection.secondary.widget_id]
                .iter()
                .any(|id| !self.ordered_labels.iter().any(|source| source.id == *id))
        }) {
            self.selection = None;
            self.is_dragging = false;
            self.granular_drag = None;
        }
        self.finish_selection_key_event(ctx);
        if got_copy_event(ctx)
            && let Some(text) = self.ordered_selected_text(ctx)
            && !text.is_empty()
        {
            ctx.copy_text(text);
        }
        if ctx.input(|input| input.key_pressed(crate::Key::Escape)) {
            self.selection = None;
            self.is_dragging = false;
            self.granular_drag = None;
        }
        let pointer_event = ctx.input(|input| {
            input
                .events
                .iter()
                .any(|event| matches!(event, Event::PointerMoved(_) | Event::PointerButton { .. }))
        });
        if !pointer_event {
            return;
        }
        let hovered = self.ordered_labels.iter().rev().find(|source| {
            ctx.pass_state(|pass| {
                pass.widgets
                    .get(source.id)
                    .is_some_and(|widget| widget.enabled && widget.visible)
            }) && ctx
                .read_response(source.id)
                .is_some_and(|response| response.contains_pointer())
        });
        if ctx.input(|input| input.pointer.any_pressed()) && hovered.is_none() {
            self.selection = None;
            self.is_dragging = false;
            self.granular_drag = None;
        }
        if self.is_dragging {
            let previous = self.selection;
            self.has_reached_primary = false;
            self.has_reached_secondary = false;
            // Visit only the registered text sources, using the same selection
            // calculation as ordinary labels. This performs no layout or paint.
            for source in self.ordered_labels.clone() {
                let eligible = ctx.pass_state(|pass| {
                    pass.widgets
                        .get(source.id)
                        .is_some_and(|widget| widget.enabled && widget.visible)
                });
                if eligible
                    && let Some(response) = ctx.read_response(source.id)
                    && ctx.memory(|memory| memory.allows_interaction(response.layer_id))
                {
                    self.cursor_for(ctx, &response, &source);
                }
            }
            if let (Some(previous), Some(current)) = (previous, self.selection) {
                self.selection_is_dirty |= previous.primary.widget_id != current.primary.widget_id
                    || previous.primary.ccursor != current.primary.ccursor
                    || previous.secondary.widget_id != current.secondary.widget_id
                    || previous.secondary.ccursor != current.secondary.ccursor;
            }
        }
        if ctx.input(|input| input.pointer.any_released()) {
            self.is_dragging = false;
        }
    }

    fn on_pointer_press(&mut self, ctx: &Context) {
        if ctx.input(|i| i.pointer.any_pressed() && !i.modifiers.shift) {
            // Maybe a new selection is about to begin, but the old one is over:
            // state.selection = None; // TODO(emilk): this makes sense, but doesn't work as expected.
        }

        if ctx.input(|i| i.pointer.any_pressed()) {
            if ctx.input(|i| i.modifiers.shift) && self.selection.is_some() {
                // A shift-click extends the current selection, keeping any double- or
                // triple-clicked word/line selected, but now by the granularity of this press:
                if let Some(granular) = &mut self.granular_drag {
                    granular.granularity = SelectGranularity::of_press(ctx);
                }
            } else {
                // Any other new press ends the previous word/line selection.
                // If this press is a double- or triple-click on a label,
                // `on_label` will set this again later this pass:
                self.granular_drag = None;
            }
        }
    }

    fn on_begin_pass(&mut self, ui: &Ui) {
        self.ordered_labels.clear();
        if !ui.ctx().options(|options| options.ordered_input) {
            self.on_pointer_press(ui.ctx());
        }

        self.selection_bbox_last_frame = self.selection_bbox_this_frame;
        self.selection_bbox_this_frame = Rect::NOTHING;

        self.any_hovered = false;
        self.has_reached_primary = false;
        self.has_reached_secondary = false;
        self.text_to_copy.clear();
        self.last_copied_galley_rect = None;
        self.painted_selections.clear();
        self.report_selection = self.selection_is_dirty && !ui.input(|i| i.pointer.any_down());
    }

    fn on_end_pass(&mut self, ui: &Ui) {
        if self.is_dragging {
            ui.set_cursor_icon(CursorIcon::Text);
        }

        if !self.has_reached_primary || !self.has_reached_secondary {
            // We didn't see both cursors this frame,
            // maybe because they are outside the visible area (scrolling),
            // or one disappeared. In either case we will have horrible glitches, so let's just deselect.

            let prev_selection = self.selection.take();
            if let Some(selection) = prev_selection {
                // This was the first frame of glitch, so hide the
                // glitching by removing all painted selections:
                ui.graphics_mut(|layers| {
                    if let Some(list) = layers.get_mut(selection.layer_id) {
                        for (shape_idx, row_selections) in self.painted_selections.drain(..) {
                            list.mutate_shape(shape_idx, |shape| {
                                if let epaint::Shape::Text(text_shape) = &mut shape.shape {
                                    let galley = Arc::make_mut(&mut text_shape.galley);
                                    for row_selection in row_selections {
                                        if let Some(placed_row) =
                                            galley.rows.get_mut(row_selection.row)
                                        {
                                            let row = Arc::make_mut(&mut placed_row.row);
                                            for vertex_index in row_selection.vertex_indices {
                                                if let Some(vertex) = row
                                                    .visuals
                                                    .mesh
                                                    .vertices
                                                    .get_mut(vertex_index as usize)
                                                {
                                                    vertex.color = epaint::Color32::TRANSPARENT;
                                                }
                                            }
                                        }
                                    }
                                }
                            });
                        }
                    }
                });
            }
        }

        let ordered = ui.ctx().options(|o| o.ordered_input);
        let pressed_escape = !ordered && ui.input(|i| i.key_pressed(crate::Key::Escape));
        let clicked_something_else =
            !ordered && ui.input(|i| i.pointer.any_pressed()) && !self.any_hovered;
        let delected_everything = pressed_escape || clicked_something_else;

        if delected_everything {
            self.selection = None;
        }

        if !ordered && ui.input(|i| i.pointer.any_released()) {
            self.is_dragging = false;
        }

        let text_to_copy = if ordered {
            if self.report_selection {
                self.ordered_selected_text(ui.ctx()).unwrap_or_default()
            } else {
                String::new()
            }
        } else {
            core::mem::take(&mut self.text_to_copy)
        };
        self.ordered_labels.clear();
        if self.report_selection {
            self.selection_is_dirty = false;
            if !text_to_copy.is_empty() {
                ui.ctx()
                    .send_cmd(crate::OutputCommand::TextSelectionSettled(
                        text_to_copy.clone(),
                    ));
            }
        }
        if !ordered && !text_to_copy.is_empty() && got_copy_event(ui.ctx()) {
            ui.copy_text(text_to_copy);
        }
    }

    fn is_active(&self) -> bool {
        self.selection.is_some() || self.is_dragging
    }

    fn has_selection(&self) -> bool {
        self.selection.is_some()
    }

    fn copy_text(&mut self, new_galley_rect: Rect, galley: &Galley, cursor_range: &CCursorRange) {
        let new_text = selected_text(galley, cursor_range);
        if new_text.is_empty() {
            return;
        }

        if self.text_to_copy.is_empty() {
            self.text_to_copy = new_text;
            self.last_copied_galley_rect = Some(new_galley_rect);
            return;
        }

        let Some(last_copied_galley_rect) = self.last_copied_galley_rect else {
            self.text_to_copy = new_text;
            self.last_copied_galley_rect = Some(new_galley_rect);
            return;
        };

        // We need to append or prepend the new text to the already copied text.
        // We need to do so intelligently.

        if last_copied_galley_rect.bottom() <= new_galley_rect.top() {
            self.text_to_copy.push('\n');
            let vertical_distance = new_galley_rect.top() - last_copied_galley_rect.bottom();
            if estimate_row_height(galley) * 0.5 < vertical_distance {
                self.text_to_copy.push('\n');
            }
        } else {
            let existing_ends_with_space =
                self.text_to_copy.chars().last().map(|c| c.is_whitespace());

            let new_text_starts_with_space_or_punctuation = new_text
                .chars()
                .next()
                .is_some_and(|c| c.is_whitespace() || c.is_ascii_punctuation());

            if existing_ends_with_space == Some(false) && !new_text_starts_with_space_or_punctuation
            {
                self.text_to_copy.push(' ');
            }
        }

        self.text_to_copy.push_str(&new_text);
        self.last_copied_galley_rect = Some(new_galley_rect);
    }

    fn cursor_for(
        &mut self,
        ctx: &Context,
        response: &Response,
        source: &LabelText,
    ) -> TextCursorState {
        let global_from_galley = source.global_from_galley;
        let galley = &source.galley;
        let Some(selection) = &mut self.selection else {
            // Nothing selected.
            return TextCursorState::default();
        };

        if selection.layer_id != response.layer_id {
            // Selection is in another layer
            return TextCursorState::default();
        }

        let galley_from_global = global_from_galley.inverse();

        let multi_widget_text_select = source.multi_widget_text_select;

        let may_select_widget =
            multi_widget_text_select || selection.primary.widget_id == response.id;

        if self.is_dragging
            && ctx.is_handling_input()
            && may_select_widget
            && let Some(pointer_pos) = ctx.pointer_interact_pos()
        {
            let galley_rect = global_from_galley * Rect::from_min_size(Pos2::ZERO, galley.size());
            let galley_rect = galley_rect.intersect(source.clip_rect);

            let is_in_same_column = galley_rect
                .x_range()
                .intersects(self.selection_bbox_last_frame.x_range());

            let has_reached_primary =
                self.has_reached_primary || response.id == selection.primary.widget_id;
            let has_reached_secondary =
                self.has_reached_secondary || response.id == selection.secondary.widget_id;

            let new_primary = if response.contains_pointer() {
                // Dragging into this widget - easy case:
                let cursor_at_pointer =
                    galley.cursor_from_pos((galley_from_global * pointer_pos).to_vec2());

                if let Some(granular) = &self.granular_drag {
                    // The selection started with a double- or triple-click,
                    // so extend it by whole words or lines:
                    if response.id == granular.anchor_min.widget_id {
                        // We are in the same widget as the anchor word/line:
                        let anchor = CCursorRange::two(
                            granular.anchor_min.ccursor,
                            granular.anchor_max.ccursor,
                        );
                        let range = extend_granular_select(
                            granular.granularity,
                            anchor,
                            galley.text(),
                            cursor_at_pointer,
                        );
                        selection.secondary = WidgetTextCursor::new(
                            response.id,
                            range.secondary,
                            global_from_galley,
                            galley,
                        );
                        Some(range.primary)
                    } else {
                        // The drag has left the anchor's widget.
                        // Are we before or after the anchor? Decide by screen position:
                        let after_anchor = granular.anchor_max.pos.y < pointer_pos.y
                            || (granular.anchor_min.pos.y <= pointer_pos.y
                                && granular.anchor_max.pos.x <= pointer_pos.x);

                        let unit =
                            select_unit_at(granular.granularity, galley.text(), cursor_at_pointer);
                        let [unit_min, unit_max] = unit.sorted_cursors();

                        if after_anchor {
                            selection.secondary = granular.anchor_min;
                            Some(unit_max)
                        } else {
                            selection.secondary = granular.anchor_max;
                            Some(unit_min)
                        }
                    }
                } else {
                    Some(cursor_at_pointer)
                }
            } else if is_in_same_column
                && !self.has_reached_primary
                && selection.primary.pos.y <= selection.secondary.pos.y
                && pointer_pos.y <= galley_rect.top()
                && galley_rect.top() <= selection.secondary.pos.y
            {
                // The user is dragging the text selection upwards, above the first selected widget (this one):
                if DEBUG {
                    ctx.debug_text(format!("Upwards drag; include {:?}", response.id));
                }
                Some(galley.begin())
            } else if is_in_same_column
                && has_reached_secondary
                && has_reached_primary
                && selection.secondary.pos.y <= selection.primary.pos.y
                && selection.secondary.pos.y <= galley_rect.bottom()
                && galley_rect.bottom() <= pointer_pos.y
            {
                // The user is dragging the text selection downwards, below this widget.
                // We move the cursor to the end of this widget,
                // (and we may do the same for the next widget too).
                if DEBUG {
                    ctx.debug_text(format!("Downwards drag; include {:?}", response.id));
                }
                Some(galley.end())
            } else {
                None
            };

            if let Some(new_primary) = new_primary {
                selection.primary =
                    WidgetTextCursor::new(response.id, new_primary, global_from_galley, galley);

                // We don't want the latency of `drag_started`.
                let drag_started = ctx.input(|i| i.pointer.any_pressed());
                if drag_started {
                    if selection.layer_id == response.layer_id {
                        if ctx.input(|i| i.modifiers.shift) {
                            // A continuation of a previous selection.
                        } else {
                            // A new selection in the same layer.
                            selection.secondary = selection.primary;
                        }
                    } else {
                        // A new selection in a new layer.
                        selection.layer_id = response.layer_id;
                        selection.secondary = selection.primary;
                    }
                }
            }
        }

        let has_primary = response.id == selection.primary.widget_id;
        let has_secondary = response.id == selection.secondary.widget_id;

        if has_primary {
            selection.primary.pos =
                global_from_galley * pos_in_galley(galley, selection.primary.ccursor);
        }
        if has_secondary {
            selection.secondary.pos =
                global_from_galley * pos_in_galley(galley, selection.secondary.ccursor);
        }

        self.has_reached_primary |= has_primary;
        self.has_reached_secondary |= has_secondary;

        let primary = has_primary.then_some(selection.primary.ccursor);
        let secondary = has_secondary.then_some(selection.secondary.ccursor);

        // The following code assumes we will encounter both ends of the cursor
        // at some point (but in any order).
        // If we don't (e.g. because one endpoint is outside the visible scroll areas),
        // we will have annoying failure cases.

        match (primary, secondary) {
            (Some(primary), Some(secondary)) => {
                // This is the only selected label.
                TextCursorState::from(CCursorRange {
                    primary,
                    secondary,
                    h_pos: None,
                })
            }

            (Some(primary), None) => {
                // This labels contains only the primary cursor.
                let secondary = if self.has_reached_secondary {
                    // Secondary was before primary.
                    // Select everything up to the cursor.
                    // We assume normal left-to-right and top-down layout order here.
                    galley.begin()
                } else {
                    // Select everything from the cursor onward:
                    galley.end()
                };
                TextCursorState::from(CCursorRange {
                    primary,
                    secondary,
                    h_pos: None,
                })
            }

            (None, Some(secondary)) => {
                // This labels contains only the secondary cursor
                let primary = if self.has_reached_primary {
                    // Primary was before secondary.
                    // Select everything up to the cursor.
                    // We assume normal left-to-right and top-down layout order here.
                    galley.begin()
                } else {
                    // Select everything from the cursor onward:
                    galley.end()
                };
                TextCursorState::from(CCursorRange {
                    primary,
                    secondary,
                    h_pos: None,
                })
            }

            (None, None) => {
                // This widget has neither the primary or secondary cursor.
                let is_in_middle = self.has_reached_primary != self.has_reached_secondary;
                if is_in_middle {
                    if DEBUG {
                        response.ctx.debug_text(format!(
                            "widget in middle: {:?}, between {:?} and {:?}",
                            response.id, selection.primary.widget_id, selection.secondary.widget_id,
                        ));
                    }
                    // …but it is between the two selection endpoints, and so is fully selected.
                    TextCursorState::from(CCursorRange::two(galley.begin(), galley.end()))
                } else {
                    // Outside the selected range
                    TextCursorState::default()
                }
            }
        }
    }

    /// Returns the painted selections, if any.
    fn on_label(
        &mut self,
        ui: &Ui,
        response: &Response,
        galley_pos_in_layer: Pos2,
        galley: &mut Arc<Galley>,
    ) -> Vec<RowVertexIndices> {
        let widget_id = response.id;

        let global_from_layer = ui
            .ctx()
            .layer_transform_to_global(ui.layer_id())
            .unwrap_or_default();
        let layer_from_galley = TSTransform::from_translation(galley_pos_in_layer.to_vec2());
        let galley_from_layer = layer_from_galley.inverse();
        let layer_from_global = global_from_layer.inverse();
        let galley_from_global = galley_from_layer * layer_from_global;
        let global_from_galley = global_from_layer * layer_from_galley;
        let ordered = ui.ctx().options(|o| o.ordered_input);
        let source = LabelText {
            id: response.id,
            registered: true,
            galley: Arc::clone(galley),
            global_from_galley,
            multi_widget_text_select: ui.style().interaction.multi_widget_text_select,
            clip_rect: ui.clip_rect(),
        };
        if ordered {
            if let Some(previous) = self
                .ordered_labels
                .iter_mut()
                .find(|source| source.id == response.id)
            {
                *previous = source.clone();
            } else {
                self.ordered_labels.push(source.clone());
            }
        }

        if response.hovered() {
            ui.set_cursor_icon(CursorIcon::Text);
        }

        self.any_hovered |= response.hovered();
        self.is_dragging |= ui.ctx().is_handling_input() && response.is_pointer_button_down_on(); // we don't want the initial latency of drag vs click decision

        let old_selection = self.selection;

        let mut cursor_state = self.cursor_for(ui.ctx(), response, &source);

        let old_range = cursor_state.range(galley);

        if ui.ctx().is_handling_input()
            && let Some(pointer_pos) = ui.ctx().pointer_interact_pos()
            && response.contains_pointer()
        {
            let cursor_at_pointer =
                galley.cursor_from_pos((galley_from_global * pointer_pos).to_vec2());

            // This is where we handle start-of-drag and double-click-to-select.
            // Actual drag-to-select happens elsewhere.
            let dragged = false;
            let shift = ui.input(|i| i.modifiers.shift);

            if shift
                && let Some(granular) = &self.granular_drag
                && granular.anchor_min.widget_id == response.id
            {
                // A shift-click in the widget with the double- or triple-clicked word/line,
                // which should stay selected:
                cursor_state.set_granular_drag(Some(GranularDragSelect {
                    granularity: granular.granularity,
                    anchor: CCursorRange::two(
                        granular.anchor_min.ccursor,
                        granular.anchor_max.ccursor,
                    ),
                }));
            }

            cursor_state.pointer_interaction(ui, response, cursor_at_pointer, galley, dragged);

            if shift && self.granular_drag.is_some() {
                // A shift-click extending a double- or triple-click selection:
                // keep the original word/line as the anchor (see `on_pointer_press`).
                // If the anchor is in another widget, `cursor_for` has already
                // extended the selection relative to it.
            } else if let Some(granular) = cursor_state.granular_drag() {
                // If this was a double- or triple-click, remember the clicked word/line
                // so that dragging extends the selection by that granularity:
                let [anchor_min, anchor_max] = if let Some(selection) = &self.selection
                    && shift
                    && selection.secondary.widget_id != response.id
                {
                    // A shift-double-click extending a selection that is anchored in another widget.
                    // `cursor_state` only sees the edge of this widget, so use the real anchor:
                    [selection.secondary; 2]
                } else {
                    granular.anchor.sorted_cursors().map(|ccursor| {
                        WidgetTextCursor::new(response.id, ccursor, global_from_galley, galley)
                    })
                };
                self.granular_drag = Some(GranularDrag {
                    granularity: granular.granularity,
                    anchor_min,
                    anchor_max,
                });
            }
        }

        if let Some(mut cursor_range) = cursor_state.range(galley) {
            let galley_rect = global_from_galley * Rect::from_min_size(Pos2::ZERO, galley.size());
            self.selection_bbox_this_frame |= galley_rect;

            if let Some(selection) = &self.selection
                && selection.primary.widget_id == response.id
                && !ordered
                && process_selection_key_events(ui.ctx(), galley, response.id, &mut cursor_range)
            {
                // The keyboard changed the selection, so forget any double- or triple-clicked word/line:
                self.granular_drag = None;
            }

            if !ordered && (got_copy_event(ui.ctx()) || self.report_selection) {
                self.copy_text(galley_rect, galley, &cursor_range);
            }

            cursor_state.set_char_range(Some(cursor_range));
        }

        // Look for changes due to keyboard and/or mouse interaction:
        let new_range = cursor_state.range(galley);
        let selection_changed = old_range != new_range;
        self.selection_is_dirty |= selection_changed;

        if let (true, Some(range)) = (selection_changed, new_range) {
            // --------------
            // Store results:

            if let Some(selection) = &mut self.selection {
                let primary_changed = Some(range.primary) != old_range.map(|r| r.primary);
                let secondary_changed = Some(range.secondary) != old_range.map(|r| r.secondary);

                selection.layer_id = response.layer_id;

                if primary_changed || !ui.style().interaction.multi_widget_text_select {
                    selection.primary =
                        WidgetTextCursor::new(widget_id, range.primary, global_from_galley, galley);
                    self.has_reached_primary = true;
                }
                if secondary_changed || !ui.style().interaction.multi_widget_text_select {
                    selection.secondary = WidgetTextCursor::new(
                        widget_id,
                        range.secondary,
                        global_from_galley,
                        galley,
                    );
                    self.has_reached_secondary = true;
                }
            } else {
                // Start of a new selection
                self.selection = Some(CurrentSelection {
                    layer_id: response.layer_id,
                    primary: WidgetTextCursor::new(
                        widget_id,
                        range.primary,
                        global_from_galley,
                        galley,
                    ),
                    secondary: WidgetTextCursor::new(
                        widget_id,
                        range.secondary,
                        global_from_galley,
                        galley,
                    ),
                });
                self.has_reached_primary = true;
                self.has_reached_secondary = true;
            }
        }

        // Scroll containing ScrollArea on cursor change:
        if let Some(range) = new_range {
            let old_primary = old_selection.map(|s| s.primary);
            let new_primary = self.selection.as_ref().map(|s| s.primary);
            if let Some(new_primary) = new_primary {
                let primary_changed = old_primary.is_none_or(|old| {
                    old.widget_id != new_primary.widget_id || old.ccursor != new_primary.ccursor
                });
                if primary_changed && new_primary.widget_id == widget_id {
                    let is_fully_visible = ui.clip_rect().contains_rect(response.rect); // TODO(emilk): remove this HACK workaround for https://github.com/emilk/egui/issues/1531
                    if selection_changed && !is_fully_visible {
                        // Scroll to keep primary cursor in view:
                        let row_height = estimate_row_height(galley);
                        let primary_cursor_rect =
                            global_from_galley * cursor_rect(galley, &range.primary, row_height);
                        ui.scroll_to_rect(primary_cursor_rect, None);
                    }
                }
            }
        }

        let cursor_range = cursor_state.range(galley);

        let mut new_vertex_indices = vec![];

        if let Some(cursor_range) = cursor_range {
            paint_text_selection(
                galley,
                ui.visuals(),
                &cursor_range,
                Some(&mut new_vertex_indices),
            );
        }

        super::accesskit_text::update_accesskit_for_text_widget(
            ui.ctx(),
            response.id,
            cursor_range,
            global_from_galley,
            galley,
        );

        new_vertex_indices
    }
}

fn got_copy_event(ctx: &Context) -> bool {
    ctx.input(|i| {
        i.events
            .iter()
            .any(|e| matches!(e, Event::Copy | Event::Cut))
    })
}

/// Returns true if the cursor changed
fn process_selection_key_events(
    ctx: &Context,
    galley: &Galley,
    widget_id: Id,
    cursor_range: &mut CCursorRange,
) -> bool {
    let os = ctx.os();

    let mut changed = false;

    ctx.input(|i| {
        // NOTE: we have a lock on ui/ctx here,
        // so be careful to not call into `ui` or `ctx` again.
        for event in &i.events {
            changed |= cursor_range.on_event(os, event, galley, widget_id);
        }
    });

    changed
}

fn selected_text(galley: &Galley, cursor_range: &CCursorRange) -> String {
    // This logic means we can select everything in an elided label (including the `…`)
    // and still copy the entire un-elided text!
    let everything_is_selected = cursor_range.contains(CCursorRange::select_all(galley));

    let copy_everything = cursor_range.is_empty() || everything_is_selected;

    if copy_everything {
        galley.text().to_owned()
    } else {
        cursor_range.slice_str(galley).to_owned()
    }
}

fn estimate_row_height(galley: &Galley) -> f32 {
    if let Some(placed_row) = galley.rows.first() {
        placed_row.height()
    } else {
        galley.size().y
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RawInput, ViewportId, ViewportInfo};

    fn child_viewport_input(viewport_id: ViewportId) -> RawInput {
        let mut input = RawInput {
            viewport_id,
            ..Default::default()
        };
        input.viewports.insert(
            viewport_id,
            ViewportInfo {
                parent: Some(ViewportId::ROOT),
                ..Default::default()
            },
        );
        input
    }

    fn test_selection() -> CurrentSelection {
        let cursor = WidgetTextCursor {
            widget_id: Id::unique("selected_label"),
            ccursor: CCursor::default(),
            pos: Pos2::ZERO,
        };
        CurrentSelection {
            layer_id: LayerId::background(),
            primary: cursor,
            secondary: cursor,
        }
    }

    #[test]
    fn viewport_passes_only_clean_up_their_own_label_selection() {
        let ctx = Context::default();
        let child_viewport_id = ViewportId::from_hash_of("child_viewport");
        let plugin = ctx.plugin::<LabelSelectionState>();
        plugin
            .lock()
            .states
            .entry(child_viewport_id)
            .or_default()
            .selection = Some(test_selection());

        let output = ctx.run_ui(RawInput::default(), |_| {});
        assert!(
            plugin
                .lock()
                .states
                .get(&child_viewport_id)
                .is_some_and(ViewportLabelSelectionState::has_selection),
            "a pass in another viewport must not clear the child viewport selection"
        );
        output.drop_without_applying_deltas();

        let output = ctx.run_ui(child_viewport_input(child_viewport_id), |_| {});
        assert!(
            !plugin.lock().has_selection(),
            "the selection must be cleared when its labels disappear from the same viewport"
        );
        output.drop_without_applying_deltas();
    }
}
