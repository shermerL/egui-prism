use super::*;
use crate::{Event, InputState, Key, RawInput};

#[derive(Default)]
pub(super) struct DispatchState {
    previous: Option<InputState>,
    aggregate: Option<InputState>,
    // The active composition can outlive a pass; the focus generation invalidates
    // late terminal events after a host interruption or focus transfer.
    composition: Option<Composition>,
    press_focus_revision: Option<u64>,
    events: Option<alloc::vec::IntoIter<Event>>,
    header: Option<RawInput>,
    pub(super) shortcut_events: Vec<Event>,
    index: usize,
    timed_targets: alloc::vec::IntoIter<Id>,
    owners: IdMap<Id>,
    rounding: IdMap<f32>,
    paint_before: Option<crate::layers::GraphicLayers>,
    pub(super) finished: bool,
    pub(super) in_event: bool,
}
struct Composition {
    owner: Id,
    generation: u64,
    cleared: bool,
}
impl DispatchState {
    pub(super) fn owner(&self, id: Id) -> Id {
        self.owners.get(&id).copied().unwrap_or(id)
    }
    pub(super) fn begin(&mut self, input: &InputState, enabled: bool) {
        *self = Self {
            composition: enabled.then(|| self.composition.take()).flatten(),
            press_focus_revision: enabled.then_some(self.press_focus_revision).flatten(),
            previous: enabled.then(|| input.clone()),
            ..Default::default()
        };
    }
}
/// What caused an ordered handler to run.
#[derive(Clone, Debug)]
pub enum InputTrigger {
    /// An original event left by beginning-of-pass plugins.
    Event {
        /// Index in that remaining event stream.
        index: usize,
        /// The original event, unchanged.
        event: Event,
    },
    /// Frame time advanced, producing a long touch or drag transition without a raw event.
    Time,
}

/// An input trigger and the widget selected by authoritative current-pass routing.
#[derive(Clone, Debug)]
pub struct RoutedInput {
    /// Target widget, if the trigger has a recipient.
    pub target: Option<Id>,
    /// The original event or an explicit time-driven handling opportunity.
    pub trigger: InputTrigger,
}
impl Context {
    /// Start the explicit current-pass input phase before registering controls.
    pub fn begin_input(&self) {
        self.write(|ctx| {
            assert!(
                ctx.memory.options.ordered_input,
                "enable ordered input before using its phases"
            );
            let vp = ctx.viewport_id();
            let v = ctx
                .viewports
                .get_mut(&vp)
                .expect("input dispatch requires an active viewport");
            assert!(
                v.dispatch.events.is_none(),
                "begin_input may only run once per pass"
            );
            v.dispatch.events = Some(core::mem::take(&mut v.input.events).into_iter());
            v.dispatch.aggregate = Some(v.input.clone());
            let original_raw = core::mem::take(&mut v.input.raw);
            let mut header = original_raw.clone();
            header.events.clear();
            header.time = Some(v.input.time);
            v.dispatch.header = Some(header);
            v.input.raw = original_raw;
            v.dispatch.paint_before = Some(core::mem::take(&mut v.graphics));
            v.input.events.clear();
        });
        crate::text_selection::LabelSelectionState::clear_input_text(self, self.viewport_id());
    }
    /// Register changed structure without consuming the event currently being handled.
    pub fn input_layout<R>(&self, layout: impl FnOnce() -> R) -> R {
        struct Restore {
            ctx: Context,
            viewport: ViewportId,
            events: Vec<Event>,
            in_event: bool,
        }
        impl Drop for Restore {
            fn drop(&mut self) {
                self.ctx.write(|ctx| {
                    let v = ctx
                        .viewports
                        .get_mut(&self.viewport)
                        .expect("the active viewport must survive layout");
                    v.input.events = core::mem::take(&mut self.events);
                    v.dispatch.in_event = self.in_event;
                });
            }
        }
        let (viewport, events, in_event) = self.write(|ctx| {
            let viewport = ctx.viewport_id();
            let v = ctx
                .viewports
                .get_mut(&viewport)
                .expect("input layout requires an active viewport");
            assert!(
                v.dispatch.events.is_some() && !v.dispatch.finished,
                "modify registration between begin_input and the end of dispatch"
            );
            let in_event = core::mem::replace(&mut v.dispatch.in_event, false);
            (viewport, core::mem::take(&mut v.input.events), in_event)
        });
        let _restore = Restore {
            ctx: self.clone(),
            viewport,
            events,
            in_event,
        };
        layout()
    }
    /// Route a registered part through a live owner in the same layer for this pass.
    ///
    /// Register both widgets first. The owner's rectangle remains its navigation
    /// rectangle; this does not expand either widget's hit area. Owners cannot be
    /// parts themselves. Renew the binding each pass before dispatching input.
    pub fn set_input_owner(&self, part: Id, owner: Id) {
        self.write(|ctx| {
            assert!(
                ctx.memory.options.ordered_input,
                "enable ordered input before using its phases"
            );
            let v = ctx.viewport();
            assert!(
                v.dispatch.events.is_some() && !v.dispatch.finished,
                "modify registration between begin_input and the end of dispatch"
            );
            assert_ne!(part, owner, "a part cannot own itself");
            assert!(
                !v.dispatch.owners.contains_key(&owner),
                "an owner cannot itself be a part"
            );
            assert!(
                !v.dispatch.owners.values().any(|id| *id == part),
                "a part cannot also own other parts"
            );
            let part_rect = v
                .this_pass
                .widgets
                .get(part)
                .expect("register the part first");
            let owner_rect = v
                .this_pass
                .widgets
                .get(owner)
                .expect("register the owner first");
            assert_eq!(
                part_rect.layer_id, owner_rect.layer_id,
                "parts and owners must share one layer"
            );
            v.dispatch.owners.insert(part, owner);
        });
    }

    /// Restrict a registered widget to rounded corners in its local rectangle this pass.
    ///
    /// Clipping and layer transforms still apply. The radius is clamped to half
    /// the shortest side; a square with that radius has a circular hit area.
    pub fn set_input_rounding(&self, id: Id, radius: f32) {
        assert!(
            radius.is_finite() && radius >= 0.0,
            "hit rounding must be finite and nonnegative"
        );
        self.write(|ctx| {
            assert!(
                ctx.memory.options.ordered_input,
                "enable ordered input before using its phases"
            );
            let v = ctx.viewport();
            assert!(
                v.dispatch.events.is_some() && !v.dispatch.finished,
                "modify registration between begin_input and the end of dispatch"
            );
            assert!(
                v.this_pass.widgets.contains(id),
                "register the widget first"
            );
            v.dispatch.rounding.insert(id, radius);
        });
    }

    pub(crate) fn discard_input_layout(&self, layer: LayerId) {
        self.write(|ctx| {
            let vp = ctx.viewport_id();
            let v = ctx
                .viewports
                .get_mut(&vp)
                .expect("input dispatch requires an active viewport");
            for id in v.this_pass.widgets.remove_layer(layer) {
                v.this_pass.used_ids.remove(&id);
            }
        });
    }
    /// Remove a vanished widget and its bound parts before routing another event.
    pub fn remove_input_widget(&self, id: Id) {
        self.write(|ctx| {
            assert!(
                ctx.memory.options.ordered_input,
                "enable ordered input before using its phases"
            );
            let vp = ctx.viewport_id();
            let viewport = ctx
                .viewports
                .get_mut(&vp)
                .expect("input dispatch requires an active viewport");
            assert!(
                viewport.dispatch.events.is_some() && !viewport.dispatch.finished,
                "remove registration before dispatch finishes"
            );
            let mut ids = vec![id];
            ids.extend(
                viewport
                    .dispatch
                    .owners
                    .iter()
                    .filter_map(|(part, owner)| (*owner == id).then_some(*part)),
            );
            for id in &ids {
                viewport.this_pass.widgets.remove(*id);
            }
            forget_input_widgets(viewport, &mut ctx.memory, ids);
        });
    }

    /// Remove a closed layer from this pass before routing another event.
    pub fn remove_input_layer(&self, layer: LayerId) {
        self.write(|ctx| {
            let vp = ctx.viewport_id();
            let v = ctx
                .viewports
                .get_mut(&vp)
                .expect("input dispatch requires an active viewport");
            assert!(
                ctx.memory.options.ordered_input,
                "enable ordered input before using its phases"
            );
            assert!(
                v.dispatch.events.is_some() && !v.dispatch.finished,
                "modify registration between begin_input and the end of dispatch"
            );
            let ids = v.this_pass.widgets.remove_layer(layer);
            forget_input_widgets(v, &mut ctx.memory, ids);
            ctx.memory.remove_input_modal_layer(layer);
        });
    }
    /// Dispatch the next raw event or time-driven interaction against current geometry.
    pub fn next_input_event(&self) -> Option<RoutedInput> {
        let (finish_event, already_finished) = self.read(|ctx| {
            let dispatch = &ctx.viewports[&ctx.viewport_id()].dispatch;
            (dispatch.in_event && !dispatch.finished, dispatch.finished)
        });
        if finish_event {
            crate::text_selection::LabelSelectionState::finish_input_event(self);
            self.write(|ctx| {
                let viewport = ctx.viewport();
                viewport.dispatch.shortcut_events.extend(
                    viewport
                        .input
                        .events
                        .drain(..)
                        .filter(|event| matches!(event, Event::Key { .. })),
                );
            });
        }
        let routed = self.write(|ctx| {
            assert!(
                ctx.memory.options.ordered_input,
                "enable ordered input before using its phases"
            );
            let vp = ctx.viewport_id();
            let viewport = ctx
                .viewports
                .get_mut(&vp)
                .expect("input dispatch requires an active viewport");
            assert!(
                viewport.dispatch.events.is_some(),
                "call begin_input before registration"
            );
            if viewport.dispatch.finished {
                return None;
            }
            viewport.graphics = Default::default();
            let event = viewport
                .dispatch
                .events
                .as_ref()
                .expect("begin_input initializes the event cursor")
                .as_slice()
                .first()
                .cloned();
            if matches!(event, Some(Event::WindowFocused(false))) {
                ctx.memory.interrupt_ime();
            }
            let is_live = |id: Id| {
                viewport.this_pass.widgets.get(id).is_some_and(|w| {
                    w.enabled && w.visible && ctx.memory.allows_interaction(w.layer_id)
                })
            };
            let cancelled = matches!(
                event,
                Some(Event::PointerGone | Event::WindowFocused(false))
            );
            let click_valid = ctx
                .memory
                .interaction()
                .potential_click_id
                .is_some_and(is_live)
                && !cancelled;
            let drag_valid = ctx
                .memory
                .interaction()
                .potential_drag_id
                .is_some_and(is_live)
                && !cancelled;
            if !click_valid {
                ctx.memory.interaction_mut().potential_click_id = None;
            }
            if !drag_valid {
                ctx.memory.interaction_mut().potential_drag_id = None;
                viewport.interact_widgets.dragged = None;
            }
            if let Some(id) = ctx.memory.focused()
                && !viewport.this_pass.widgets.get(id).is_some_and(|w| {
                    w.enabled && w.visible && ctx.memory.allows_interaction(w.layer_id)
                })
            {
                ctx.memory.surrender_focus(id);
            }
            if let Some(previous) = viewport.dispatch.previous.take() {
                // Advance the clock once, before the raw batch. This preserves
                // time-driven pointer transitions even when there are no events.
                let original_raw = core::mem::take(&mut viewport.input.raw);
                let header = viewport
                    .dispatch
                    .header
                    .as_ref()
                    .expect("begin_input saves the frame header")
                    .clone();
                viewport.input = previous.begin_pass(
                    header,
                    true,
                    viewport.input.pixels_per_point,
                    ctx.memory.options.input_options,
                );
                viewport.input.raw = original_raw;
                let (_, hits) = current_hits(viewport, &ctx.memory);
                viewport.hits = hits;
                viewport.interact_widgets = crate::interaction::interact(
                    &viewport.interact_widgets,
                    &viewport.this_pass.widgets,
                    &viewport.hits,
                    &viewport.input,
                    ctx.memory.interaction_mut(),
                );
                let mut targets = Vec::new();
                for target in [
                    viewport.interact_widgets.long_touched,
                    viewport.interact_widgets.drag_started,
                    viewport.interact_widgets.drag_stopped,
                ]
                .into_iter()
                .flatten()
                {
                    if !targets.contains(&target) {
                        targets.push(target);
                    }
                }
                viewport.dispatch.timed_targets = targets.into_iter();
            }
            for target in viewport.dispatch.timed_targets.by_ref() {
                if viewport.this_pass.widgets.get(target).is_some_and(|w| {
                    w.enabled && w.visible && ctx.memory.allows_interaction(w.layer_id)
                }) {
                    viewport.dispatch.in_event = true;
                    return Some(RoutedInput {
                        target: Some(target),
                        trigger: InputTrigger::Time,
                    });
                }
            }
            let event = viewport
                .dispatch
                .events
                .as_mut()
                .expect("begin_input initializes the event cursor")
                .next();
            let Some(event) = event else {
                let (_, hits) = current_hits(viewport, &ctx.memory);
                viewport.hits = hits;
                let mut resting = viewport
                    .dispatch
                    .aggregate
                    .as_ref()
                    .expect("begin_input saves the aggregate input")
                    .clone();
                resting.events.clear();
                resting.pointer.pointer_events.clear();
                let hover = crate::interaction::interact(
                    &viewport.interact_widgets,
                    &viewport.this_pass.widgets,
                    &viewport.hits,
                    &resting,
                    ctx.memory.interaction_mut(),
                );
                viewport.interact_widgets.hovered = hover.hovered;
                viewport.interact_widgets.contains_pointer = hover.contains_pointer;
                viewport.input = viewport
                    .dispatch
                    .aggregate
                    .take()
                    .expect("aggregate input is restored once");
                viewport.input.events.clear();
                viewport.dispatch.in_event = false;
                viewport.dispatch.finished = true;
                viewport.dispatch.previous = None;
                viewport.dispatch.header = None;
                viewport.graphics = viewport
                    .dispatch
                    .paint_before
                    .take()
                    .expect("begin_input saves plugin graphics");
                return None;
            };
            let index = viewport.dispatch.index;
            viewport.dispatch.index += 1;
            viewport.dispatch.in_event = true;
            let original_raw = core::mem::take(&mut viewport.input.raw);
            let mut raw = viewport
                .dispatch
                .header
                .as_ref()
                .expect("begin_input saves the frame header")
                .clone();
            raw.events = vec![event.clone()];
            // Time and frame metadata were already advanced by the explicit time step.
            raw.predicted_dt = 0.;
            raw.dropped_files.clear();
            let pixels_per_point = viewport.input.pixels_per_point;
            let previous = core::mem::take(&mut viewport.input);
            viewport.input = previous.begin_pass(
                raw,
                true,
                pixels_per_point,
                ctx.memory.options.input_options,
            );
            viewport.input.raw = original_raw;
            let (layers, hits) = if let Event::MiddleClickPaste { pos, .. } = &event {
                current_hits_at(viewport, &ctx.memory, Some(*pos))
            } else {
                current_hits(viewport, &ctx.memory)
            };
            viewport.hits = hits;
            let release_owner = ctx
                .memory
                .interaction()
                .potential_drag_id
                .or_else(|| ctx.memory.interaction().potential_click_id);
            viewport.interact_widgets = crate::interaction::interact(
                &viewport.interact_widgets,
                &viewport.this_pass.widgets,
                &viewport.hits,
                &viewport.input,
                ctx.memory.interaction_mut(),
            );
            if viewport
                .interact_widgets
                .clicked
                .is_some_and(|id| viewport.hits.click.is_none_or(|hit| hit.id != id))
            {
                viewport.interact_widgets.clicked = None;
            }
            let mut target = match &event {
                Event::PointerButton { pressed: true, .. } => {
                    viewport.hits.click.or(viewport.hits.drag).map(|w| w.id)
                }
                Event::PointerButton { pressed: false, .. } => viewport
                    .interact_widgets
                    .clicked
                    .or(viewport.interact_widgets.drag_stopped)
                    .or(release_owner),
                Event::PointerMoved(_) => ctx
                    .memory
                    .interaction()
                    .potential_drag_id
                    .or_else(|| viewport.hits.drag.or(viewport.hits.click).map(|w| w.id)),
                Event::PointerGone => None,
                Event::MiddleClickPaste { .. } => {
                    viewport.hits.click.or(viewport.hits.drag).map(|w| w.id)
                }
                _ => ctx.memory.focused(),
            };
            if let Event::AccessKitActionRequest(request) = &event {
                target = if request.target_tree == accesskit::TreeId::ROOT {
                    viewport
                        .this_pass
                        .widgets
                        .layers()
                        .flat_map(|(_, widgets)| widgets)
                        .find(|w| {
                            w.enabled
                                && w.visible
                                && ctx.memory.allows_interaction(w.layer_id)
                                && w.id.accesskit_id() == request.target_node
                        })
                        .map(|w| w.id)
                } else {
                    None
                };
                if request.action == accesskit::Action::Focus {
                    if let Some(id) = target {
                        ctx.memory.request_focus(id);
                    }
                    target = None;
                }
            }
            if let Event::Ime(ime) = &event {
                match ime {
                    crate::ImeEvent::Preedit { text, .. }
                        if !text.is_empty()
                            && viewport
                                .dispatch
                                .composition
                                .as_ref()
                                .is_none_or(|c| c.cleared) =>
                    {
                        viewport.dispatch.composition =
                            ctx.memory.focused().map(|owner| Composition {
                                owner,
                                generation: ctx.memory.ime_generation(),
                                cleared: false,
                            });
                    }
                    _ => {}
                }
                if let Some(composition) = &mut viewport.dispatch.composition {
                    target = (ctx.memory.focused() == Some(composition.owner)
                        && composition.generation == ctx.memory.ime_generation())
                    .then_some(composition.owner);
                    // winit may clear the preedit before committing, even across frames.
                    // Keep the old owner until that commit or the next composition starts.
                    if matches!(ime, crate::ImeEvent::Preedit { text, .. } if text.is_empty()) {
                        composition.cleared = true;
                    }
                }
                if matches!(ime, crate::ImeEvent::Commit(_)) {
                    viewport.dispatch.composition = None;
                }
            }
            if let Event::Key {
                key,
                pressed: true,
                modifiers,
                ..
            } = &event
            {
                let filter = ctx.memory.dispatch_focus_filter();
                if !filter.matches(&event) {
                    if *key == Key::Tab && (!modifiers.any() || modifiers.shift_only()) {
                        let mut ids = Vec::new();
                        for &layer in &layers {
                            for w in viewport.this_pass.widgets.get_layer(layer) {
                                if w.enabled
                                    && w.visible
                                    && w.sense.is_focusable()
                                    && !viewport.dispatch.owners.contains_key(&w.id)
                                    && !ids.contains(&w.id)
                                {
                                    ids.push(w.id);
                                }
                            }
                        }
                        let focused = ctx.memory.focused();
                        let next = if modifiers.shift {
                            match ids.iter().position(|id| Some(*id) == focused) {
                                Some(0) | None => ids.last().copied(),
                                Some(i) => Some(ids[i - 1]),
                            }
                        } else {
                            match ids.iter().position(|id| Some(*id) == focused) {
                                Some(i) => ids.get(i + 1).or_else(|| ids.first()).copied(),
                                None => ids.first().copied(),
                            }
                        };
                        if let Some(id) = next
                            && focused != Some(id)
                        {
                            ctx.memory.request_focus(id);
                        }
                        target = None;
                    } else if *key == Key::Escape && !modifiers.any() {
                        if let Some(id) = ctx.memory.focused() {
                            ctx.memory.surrender_focus(id);
                        }
                        // Navigation still surrenders focus, but the drag owner
                        // must finish its session before another event replaces
                        // this one-shot interaction transition.
                        target = viewport.interact_widgets.drag_stopped;
                    } else if !modifiers.any() {
                        let rects = layers
                            .iter()
                            .flat_map(|layer| viewport.this_pass.widgets.get_layer(*layer))
                            .filter(|w| {
                                w.enabled
                                    && w.visible
                                    && w.sense.is_focusable()
                                    && !viewport.dispatch.owners.contains_key(&w.id)
                            })
                            .map(|w| {
                                let rect = ctx
                                    .memory
                                    .to_global
                                    .get(&w.layer_id)
                                    .map_or(w.rect, |transform| *transform * w.rect);
                                (w.id, rect)
                            })
                            .collect();
                        if ctx.memory.dispatch_focus_direction(*key, &rects) {
                            target = None;
                        }
                    }
                }
            }
            let should_surrender = match ctx.memory.options.input_options.surrender_focus_on {
                crate::SurrenderFocusOn::Presses => viewport.input.pointer.any_pressed(),
                crate::SurrenderFocusOn::Clicks => {
                    viewport.input.pointer.any_click()
                        && viewport
                            .dispatch
                            .press_focus_revision
                            .is_none_or(|revision| revision == ctx.memory.focus_revision())
                }
                crate::SurrenderFocusOn::Never => false,
            };
            if should_surrender
                && let Some(id) = ctx.memory.focused()
                && !viewport.interact_widgets.hovered.contains(&id)
            {
                ctx.memory.surrender_focus(id);
            }
            if let Event::PointerButton {
                pressed: true,
                button: crate::PointerButton::Primary,
                ..
            } = event
                && let Some(target) = target
                && viewport
                    .this_pass
                    .widgets
                    .get(target)
                    .is_some_and(|w| w.sense.is_focusable())
                && !ctx.memory.has_focus(target)
            {
                ctx.memory.request_focus(target);
            }
            if matches!(event, Event::PointerButton { pressed: true, .. }) {
                viewport.dispatch.press_focus_revision = Some(ctx.memory.focus_revision());
            } else if !viewport.input.pointer.any_down() || cancelled {
                viewport.dispatch.press_focus_revision = None;
            }
            Some(RoutedInput {
                target,
                trigger: InputTrigger::Event { index, event },
            })
        });
        if routed.is_some() {
            crate::text_selection::LabelSelectionState::begin_input_event(self);
        } else if !already_finished {
            crate::text_selection::LabelSelectionState::prepare_input_paint(self);
        }
        routed
    }

    pub(crate) fn is_handling_input(&self) -> bool {
        self.read(|ctx| {
            !ctx.memory.options.ordered_input || ctx.viewports[&ctx.viewport_id()].dispatch.in_event
        })
    }
}

fn current_hits(viewport: &ViewportState, memory: &Memory) -> (Vec<LayerId>, WidgetHits) {
    current_hits_at(viewport, memory, viewport.input.pointer.interact_pos())
}

fn current_hits_at(
    viewport: &ViewportState,
    memory: &Memory,
    pos: Option<Pos2>,
) -> (Vec<LayerId>, WidgetHits) {
    let mut layers: Vec<_> = viewport
        .this_pass
        .widgets
        .layer_ids()
        .filter(|layer| memory.areas().is_interactable(*layer) && memory.allows_interaction(*layer))
        .collect();
    let order = memory.areas().order_current();
    layers.sort_by_key(|layer| (layer.order, order.iter().position(|entry| entry == layer)));
    let mut hits = pos
        .map(|pos| {
            crate::hit_test::hit_test_filtered(
                &viewport.this_pass.widgets,
                &layers,
                &memory.to_global,
                pos,
                0.,
                |widget, local_pos| {
                    let owner = viewport.dispatch.owner(widget.id);
                    if owner != widget.id
                        && !viewport
                            .this_pass
                            .widgets
                            .get(owner)
                            .is_some_and(|w| w.enabled && w.visible)
                    {
                        return false;
                    }
                    viewport
                        .dispatch
                        .rounding
                        .get(&widget.id)
                        .is_none_or(|radius| {
                            let radius =
                                radius.min(widget.rect.width().min(widget.rect.height()) * 0.5);
                            widget.rect.shrink(radius).distance_sq_to_pos(local_pos)
                                <= radius * radius
                        })
                },
            )
        })
        .unwrap_or_default();
    // Resolve geometry first, then bind native click/drag capture to its
    // semantic owner. There is still one egui capture state.
    for hit in hits
        .close
        .iter_mut()
        .chain(hits.contains_pointer.iter_mut())
        .chain(hits.click.iter_mut())
        .chain(hits.drag.iter_mut())
    {
        hit.id = viewport.dispatch.owner(hit.id);
    }

    (layers, hits)
}

fn forget_input_widgets(
    viewport: &mut ViewportState,
    memory: &mut Memory,
    ids: impl IntoIterator<Item = Id>,
) {
    for id in ids {
        viewport.this_pass.used_ids.remove(&id);
        viewport
            .dispatch
            .owners
            .retain(|part, owner| *part != id && *owner != id);
        viewport.dispatch.rounding.remove(&id);
        memory.ignore_focus(id);
        let interaction = memory.interaction_mut();
        if interaction.potential_click_id == Some(id) {
            interaction.potential_click_id = None;
        }
        if interaction.potential_drag_id == Some(id) {
            interaction.potential_drag_id = None;
        }
        if viewport.interact_widgets.dragged == Some(id) {
            viewport.interact_widgets.dragged = None;
        }
    }
}
