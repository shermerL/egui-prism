//! Frame-local, ordered-input modal dialogs.
//!
//! Register background controls before dialogs. During dispatch, give popup and
//! targeted child handlers their opportunity to handle an event, then offer it
//! to dialogs, from topmost to bottommost, before background handlers.
//!
//! Accept a dismissal by changing caller-owned state and calling `refresh`
//! before requesting the next routed event. A declined dismissal does not
//! change the dialog's open state.
//!
//! Content closures execute immediately; neither closures nor borrowed Uis are
//! retained. They may run more than once while registering layout, but run once
//! during final painting. They must not perform business actions during layout.
//!
//! Only explicit `DialogUi::draw` groups participate in body opacity isolation.
//! Ordinary egui painting remains ordinary egui painting: it receives the layer
//! transform, but is not captured or included in the dialog's opacity group.
//!
//! The native ScrollArea owns scrolling, inertia, capture, and its persistent
//! state. The registration retains only this pass's measured geometry.

use std::sync::Arc;

use egui::{
    Align, Color32, Context, CornerRadius, Event, EventFilter, FontFamily, FontId, Id,
    InputTrigger, Key, LayerId, Layout, Order, PointerButton, Popup, Rect, Response, RoutedInput,
    ScrollArea, Sense, Shape, Stroke, StrokeKind, TextFormat, Ui, UiBuilder, Vec2,
    emath::TSTransform,
    epaint::{ClippedShape, Galley, Shadow},
    layers::ShapeIdx,
    pos2,
    scroll_area::ScrollSource,
    text::LayoutJob,
    vec2,
};

use crate::{
    compositing::{self, Group, Item},
    theme::Theme,
};

const DURATION: f64 = 0.2;
const INSET: f32 = 25.0; // 24 pt CSS padding, inside a 1 pt border.
const BODY_GAP: f32 = 16.0;
const RADIUS: u8 = 10;
const TITLE_FAMILY: &str = "prism-geist-semibold";

/// A requested dismissal. The caller decides whether to accept it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DialogClose {
    /// Escape requested a neutral dismissal.
    Escape,
    /// A primary pointer press occurred outside the panel.
    Backdrop,
    /// The shell close button was activated.
    CloseButton,
}

/// The result of offering one ordered event to a dialog.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DialogIntent {
    /// A neutral request that leaves the caller-owned open value unchanged.
    pub close_requested: Option<DialogClose>,
    /// Refresh this registration before requesting another routed event.
    pub layout_changed: bool,
}

/// Scoped access to a dialog's scrollable body.
pub struct DialogUi<'a> {
    ui: &'a mut Ui,
    groups: Vec<Group>,
    focus: Vec<Id>,
}

impl DialogUi<'_> {
    /// Borrow the scoped body for layout and child registration.
    pub fn ui(&mut self) -> &mut Ui {
        self.ui
    }

    /// Add an unfaded group, preserving its child isolation boundaries.
    ///
    /// The dialog clips it to the native ScrollArea's content viewport.
    pub fn draw(&mut self, group: Group) {
        self.groups.push(group);
    }

    /// Enroll an eligible body control for default initial focus.
    ///
    /// This does not replace the control's focus policy or install a parallel
    /// Tab dispatcher.
    pub fn focusable(&mut self, response: &Response) {
        assert_eq!(&response.ctx, self.ui.ctx(), "dialog child context");
        assert_eq!(response.layer_id, self.ui.layer_id(), "dialog child layer");

        // Keep all enrolled IDs, including temporarily clipped or disabled
        // controls, so refresh can revoke only children that actually vanished.
        if !self.focus.contains(&response.id) {
            self.focus.push(response.id);
        }
    }
}

/// Configuration borrowed only for the current frame.
pub struct Dialog<'a> {
    theme: &'a Theme,
    id: Id,
    title: &'a str,
    description: Option<&'a str>,
    initial_focus: Option<Id>,
    restore_focus: Option<Id>,
}

impl<'a> Dialog<'a> {
    /// Borrow a theme and title for a stable dialog identity.
    pub fn new(theme: &'a Theme, id: Id, title: &'a str) -> Self {
        Self {
            theme,
            id,
            title,
            description: None,
            initial_focus: None,
            restore_focus: None,
        }
    }

    /// Show supporting text below the title.
    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    /// Prefer an enrolled child when opening.
    pub fn initial_focus(mut self, id: Id) -> Self {
        self.initial_focus = Some(id);
        self
    }

    /// Prefer this eligible trigger when closing.
    pub fn restore_focus(mut self, id: Id) -> Self {
        self.restore_focus = Some(id);
        self
    }

    /// Register the current open value and execute the layout-only body now.
    pub fn register<R>(
        self,
        context: &Context,
        open: bool,
        content: impl FnMut(&mut DialogUi<'_>) -> R,
    ) -> DialogRegistration<'a, R> {
        assert!(
            context.options(|options| options.ordered_input),
            "Dialog requires ordered input"
        );

        let viewport = context.viewport_id();
        let owner = self.id.with(("prism-dialog", viewport));
        let key = owner.with("state");
        let state = context
            .data(|data| data.get_temp::<State>(key))
            .unwrap_or_default();

        let mut registration = DialogRegistration {
            dialog: self,
            context: context.clone(),
            viewport,
            pass: context.cumulative_pass_nr(),
            now: context.input(|input| input.time),
            owner,
            key,
            state,
            geometry: None,
            header: None,
            enrolled: Vec::new(),
            inner: None,
        };
        registration.refresh(open, content);
        registration
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct Fade {
    from: [f32; 2],
    to: f32,
    started: f64,
}

impl Fade {
    fn values(self, now: f64) -> [f32; 2] {
        std::array::from_fn(|index| {
            let duration = [DURATION, 0.15][index];
            let fraction = css_ease(((now - self.started) / duration).clamp(0.0, 1.0) as f32);
            self.from[index] + (self.to - self.from[index]) * fraction
        })
    }

    fn value(self, now: f64) -> f32 {
        self.values(now)[0]
    }

    fn retarget(&mut self, now: f64, to: f32) {
        if self.to != to {
            *self = Self {
                from: self.values(now),
                to,
                started: now,
            };
        }
    }

    fn moving(self, now: f64) -> bool {
        self.from.iter().any(|from| *from != self.to) && now - self.started < DURATION
    }
}

#[derive(Clone, Default)]
struct State {
    open: bool,
    fade: Fade,
    opener: Option<Id>,
    last_focus: Option<Id>,
    pending_focus: bool,
    // Closed but still mounted until the content exit deadline.
    restore_pending: bool,
    space_armed: bool,
    // Native ScrollArea state key, retained only for reopening/removal cleanup.
    scroll_id: Option<Id>,
}

/// Only a current-pass list of live Dialog layers.
#[derive(Clone, Default)]
struct LivePass {
    pass: u64,
    layers: Vec<LayerId>,
}

struct Header {
    title: Arc<Galley>,
    description: Option<Arc<Galley>>,
    height: f32,
    centered: bool,
}

#[derive(Clone, Copy)]
struct Geometry {
    outer: Rect,
    body: Rect,
    content_clip: Rect,
    natural: Vec2,
    /// The offset used to lay out the currently registered children.
    ///
    /// This is a frame-local geometry sample, not scrolling state.
    offset: Vec2,
}

struct ScrollBuilt<T> {
    id: Option<Id>,
    inner: T,
    natural: Vec2,
    layout_offset: Vec2,
    end_offset: Vec2,
    clip: Rect,
    chrome: Vec<ClippedShape>,
}

struct Built<T> {
    scroll_id: Option<Id>,
    inner: T,
    groups: Vec<Group>,
    enrolled: Vec<Id>,
    geometry: Geometry,
    end_offset: Vec2,
    scroll_chrome: Vec<ClippedShape>,
    close: Response,
}

/// A registration belongs to one viewport and one egui pass.
///
/// Do not retain it across frames. Keep registering a closed dialog while its
/// exit animation is wanted. Call `unregister` before abandoning a live
/// registration during dispatch.
#[must_use = "dispatch and paint, or explicitly unregister, this dialog"]
pub struct DialogRegistration<'a, R> {
    dialog: Dialog<'a>,
    context: Context,
    viewport: egui::ViewportId,
    pass: u64,
    now: f64,
    owner: Id,
    key: Id,
    state: State,
    geometry: Option<Geometry>,
    header: Option<Header>,
    enrolled: Vec<Id>,
    /// The last current-pass content result, absent when logically closed.
    pub inner: Option<R>,
}

impl<R> DialogRegistration<'_, R> {
    fn layer(&self) -> LayerId {
        LayerId::new(Order::Foreground, self.owner)
    }

    fn close_id(&self) -> Id {
        self.owner.with("close")
    }

    fn backdrop_id(&self) -> Id {
        self.owner.with("backdrop")
    }

    fn panel_id(&self) -> Id {
        self.owner.with("panel")
    }

    fn live_key(&self) -> Id {
        Id::unique(("prism-dialog/live-pass", self.viewport))
    }

    fn assert_pass(&self) {
        assert_eq!(self.viewport, self.context.viewport_id(), "dialog viewport");
        assert_eq!(
            self.pass,
            self.context.cumulative_pass_nr(),
            "dialog registrations are frame-local"
        );
    }

    fn save(&self) {
        self.context
            .data_mut(|data| data.insert_temp(self.key, self.state.clone()));
    }

    fn remember_layer(&self, live: bool) {
        self.context.data_mut(|data| {
            let mut list = data
                .get_temp::<LivePass>(self.live_key())
                .unwrap_or_default();

            if list.pass != self.pass {
                list = LivePass {
                    pass: self.pass,
                    layers: Vec::new(),
                };
            }

            if live {
                if !list.layers.contains(&self.layer()) {
                    list.layers.push(self.layer());
                }
            } else {
                list.layers.retain(|layer| *layer != self.layer());
            }

            data.insert_temp(self.live_key(), list);
        });
    }

    fn top(&self) -> bool {
        self.context.memory(|memory| memory.top_modal_layer()) == Some(self.layer())
    }

    fn transform(&self) -> TSTransform {
        let scale = 0.95 + 0.05 * self.state.fade.value(self.now);
        TSTransform {
            scaling: scale,
            translation: self.context.content_rect().center().to_vec2() * (1.0 - scale),
        }
    }

    fn local_screen(&self) -> Rect {
        self.transform().inverse() * self.context.content_rect()
    }

    fn live_response(&self, id: Id) -> Option<Response> {
        self.context.read_response(id).filter(eligible)
    }

    fn own_eligible(&self, id: Id) -> bool {
        (id == self.close_id() || self.enrolled.contains(&id))
            && self
                .live_response(id)
                .is_some_and(|response| response.layer_id == self.layer())
    }

    fn repair_focus(&mut self) {
        if !self.state.open || !self.top() || Popup::is_any_open(&self.context) {
            return;
        }

        let focused = self.context.memory(|memory| memory.focused());

        if let Some(id) = focused
            && self.own_eligible(id)
        {
            self.state.last_focus = Some(id);
            self.state.pending_focus = false;
            return;
        }

        // A separately focused, interactive higher layer is not ours to take.
        if !self.state.pending_focus
            && let Some(response) = focused.and_then(|id| self.live_response(id))
            && response.layer_id != self.layer()
            && self
                .context
                .memory(|memory| memory.allows_interaction(response.layer_id))
        {
            return;
        }

        let requested = if self.state.pending_focus {
            self.dialog.initial_focus
        } else {
            self.state.last_focus
        };

        let target = requested
            .filter(|id| self.own_eligible(*id))
            .or_else(|| {
                self.enrolled
                    .iter()
                    .copied()
                    .find(|id| self.own_eligible(*id))
            })
            .or_else(|| {
                self.own_eligible(self.close_id())
                    .then_some(self.close_id())
            });

        if let Some(id) = target {
            self.context.memory_mut(|memory| memory.request_focus(id));
            self.state.last_focus = Some(id);
            self.state.pending_focus = false;
        }
    }

    fn revoke(&mut self, restore: bool) {
        let was_top = self.top();
        let focused = self.context.memory(|memory| memory.focused());
        let owned_focus = focused.is_none()
            || focused == self.state.last_focus
            || focused.is_some_and(|id| {
                self.context
                    .read_response(id)
                    .is_some_and(|response| response.layer_id == self.layer())
            });

        // This is an actual close/removal, not a layout refresh.
        self.context.remove_input_layer(self.layer());
        self.remember_layer(false);

        // Some egui versions keep the modal stack themselves. Re-establish the
        // remaining Dialog barriers only when removal left no barrier.
        if was_top
            && self
                .context
                .memory(|memory| memory.top_modal_layer().is_none())
        {
            let layers = self.context.data(|data| {
                data.get_temp::<LivePass>(self.live_key())
                    .filter(|list| list.pass == self.pass)
                    .map_or_else(Vec::new, |list| list.layers)
            });
            self.context.memory_mut(|memory| {
                for layer in layers {
                    memory.set_modal_layer(layer);
                }
            });
        }

        if restore && was_top && owned_focus {
            let target = self.dialog.restore_focus.or(self.state.opener);
            if let Some(response) = target.and_then(|id| self.live_response(id)) {
                let allowed = self
                    .context
                    .memory(|memory| memory.allows_interaction(response.layer_id));
                let current = self.context.memory(|memory| memory.focused());
                if allowed && (current.is_none() || current == focused) {
                    response.request_focus();
                }
            }
        }

        self.state.last_focus = None;
        self.state.pending_focus = false;
        self.state.space_armed = false;
        self.context.request_repaint();
    }

    /// Rebuild geometry and child registrations without handling input.
    ///
    /// Layout callbacks may run more than once. Make business-state changes
    /// before calling this method.
    pub fn refresh(&mut self, open: bool, mut content: impl FnMut(&mut DialogUi<'_>) -> R) {
        self.assert_pass();

        let previous_children = self.enrolled.clone();
        // A closed shell remains mounted until its content exit completes,
        // including an open/close pair at a zero-opacity timestamp.
        if !self.state.open
            && self.state.restore_pending
            && self.now >= self.state.fade.started + DURATION
        {
            // Ordered input clears modal bookkeeping at each pass start.
            // Re-establish this retiring presence before removing it; an
            // already registered higher layer still wins the modal comparison.
            self.context
                .memory_mut(|memory| memory.set_modal_layer(self.layer()));
            self.revoke(true);
            self.state.restore_pending = false;
        }
        let opening = open && !self.state.open;
        let mounting = opening && !self.state.restore_pending;
        if opening {
            if let Some(id) = self.state.scroll_id.take() {
                self.context
                    .data_mut(|data| data.remove::<egui::scroll_area::State>(id));
            }
            if mounting {
                self.state.opener = self.context.memory(|memory| memory.focused());
            }
            self.state.restore_pending = false;
            self.state.pending_focus = true;
            self.state.space_armed = false;
        } else if !open && self.state.open {
            self.state.restore_pending = true;
            self.state.pending_focus = false;
            self.state.space_armed = false;
        }
        self.state.open = open;
        self.state
            .fade
            .retarget(self.now, if open { 1.0 } else { 0.0 });
        self.inner = None;
        let present = open || self.state.restore_pending;
        self.remember_layer(present);
        if !present {
            self.geometry = None;
            self.header = None;
            self.enrolled.clear();
            self.save();
            return;
        }

        let screen = self.context.content_rect();
        if !screen.is_finite() {
            self.geometry = None;
            self.header = None;
            self.enrolled.clear();
            self.save();
            return;
        }

        let screen_width = screen.width().max(0.0);
        let screen_height = screen.height().max(0.0);

        // Preserve the reference responsive sizing. Only genuinely tiny
        // viewports relax the outer margin to retain as much of X as possible.
        let normal_width = if screen_width >= 640.0 {
            512.0
        } else {
            (screen_width - 32.0).max(0.0)
        };
        let width = normal_width.max(screen_width.min(16.0)).min(screen_width);
        let maximum_height = (screen_height - 32.0)
            .max(0.0)
            .max(screen_height.min(16.0))
            .min(screen_height);

        self.header = Some(make_header(
            &self.context,
            &self.dialog,
            (width - 2.0 * INSET).max(1.0),
            screen_width < 640.0,
        ));

        self.context
            .set_transform_layer(self.layer(), self.transform());

        let mut height = self
            .geometry
            .map_or(maximum_height, |geometry| geometry.outer.height())
            .clamp(0.0, maximum_height);
        let context = self.context.clone();

        // Ordinarily this takes two builds: measure, then center at the measured
        // height. A native offset clamp after a resize can require another.
        for attempt in 0..4 {
            let outer = Rect::from_center_size(screen.center(), vec2(width, height));
            let header = self.header.as_ref().expect("measured header");

            let built = context.input_layout(|| {
                let modal = egui::Modal::new(self.owner)
                    .area(
                        egui::Area::new(self.owner)
                            .order(Order::Foreground)
                            .fixed_pos(outer.min)
                            .default_size(outer.size())
                            .movable(false)
                            .constrain(false)
                            .fade_in(false)
                            .sense(Sense::hover()),
                    )
                    .frame(egui::Frame::NONE)
                    .backdrop_color(Color32::TRANSPARENT)
                    .register(&context, |ui| {
                        let built = self.build(outer, open, header, &mut content);
                        ui.set_min_size(outer.size());
                        built
                    });

                context.set_input_owner(modal.backdrop_response.id, self.backdrop_id());
                modal.inner
            });

            let fixed_height = 2.0 * INSET + header.height + BODY_GAP;
            let next_height = (fixed_height + built.geometry.natural.y)
                .min(maximum_height)
                .max(0.0);
            let settled =
                (next_height - height).abs() <= 0.01 && built.end_offset == built.geometry.offset;

            if built.scroll_id.is_some() {
                self.state.scroll_id = built.scroll_id;
            }
            self.geometry = Some(built.geometry);
            self.enrolled = built.enrolled;
            if open {
                self.inner = Some(built.inner);
            }

            if settled {
                break;
            }
            if attempt == 3 {
                // Pathological geometry-dependent content is diagnosed by
                // paint's geometry check, not by a viewport-size assertion.
                context.request_repaint();
            } else {
                height = next_height;
            }
        }

        for id in previous_children {
            if !self.enrolled.contains(&id) {
                context.remove_input_widget(id);
            }
        }
        if mounting {
            // Define simultaneous opening order: the last newly opened
            // registration is topmost. Merely refreshing a live layer never
            // raises it over another modal.
            context.move_to_top(self.layer());
            context.memory_mut(|memory| memory.set_modal_layer(self.layer()));
        }

        if opening {
            self.repair_focus();
        }
        // Existing instances repair focus during dispatch, after every modal
        // has registered. Registration order must not steal an exiting peer's focus.
        if self.state.restore_pending || self.state.fade.moving(self.now) {
            context.request_repaint();
        }
        self.save();
    }

    fn build<T>(
        &self,
        outer: Rect,
        interactive: bool,
        header: &Header,
        content: impl FnOnce(&mut DialogUi<'_>) -> T,
    ) -> Built<T> {
        let screen = self.local_screen();
        let mut chrome = Ui::new(
            self.context.clone(),
            self.owner.with("chrome-ui"),
            UiBuilder::new().layer_id(self.layer()).max_rect(outer),
        );
        chrome.set_clip_rect(screen);
        if !interactive {
            chrome.disable();
        }

        // Both the paint backdrop and its hit area use inverse-transformed
        // viewport coordinates. The layer transform is applied exactly once.
        chrome.interact(screen, self.backdrop_id(), Sense::CLICK | Sense::DRAG);
        chrome.interact(outer, self.panel_id(), Sense::CLICK | Sense::DRAG);

        let interior = bounded_inset(outer, INSET);
        let body_top = (outer.top() + INSET + header.height + BODY_GAP)
            .min(interior.bottom())
            .max(interior.top());
        let body = Rect::from_min_max(pos2(interior.left(), body_top), interior.right_bottom());

        let scrolled = self.scroll(body, interactive, |ui, _viewport| {
            let mut scoped = DialogUi {
                ui,
                groups: Vec::new(),
                focus: Vec::new(),
            };
            let inner = content(&mut scoped);
            (inner, scoped.groups, scoped.focus)
        });

        assert!(
            scrolled.natural.is_finite(),
            "Dialog content must have a finite natural extent"
        );

        // Native focus navigation sees body controls first, then X.
        let close = chrome.interact(close_rect(outer), self.close_id(), Sense::click());
        close.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::Button, close.enabled(), "Close")
        });
        if interactive && close.interact_rect.is_positive() {
            self.context.memory_mut(|memory| {
                memory.register_focus_policy(self.close_id(), self.layer(), EventFilter::default());
            });
        }

        Built {
            scroll_id: scrolled.id,
            inner: scrolled.inner.0,
            groups: scrolled.inner.1,
            enrolled: scrolled.inner.2,
            geometry: Geometry {
                outer,
                body,
                content_clip: scrolled.clip,
                natural: scrolled.natural,
                offset: scrolled.layout_offset,
            },
            end_offset: scrolled.end_offset,
            scroll_chrome: scrolled.chrome,
            close,
        }
    }

    /// Construct the same native shell for layout, input, and final painting.
    ///
    /// Only native chrome appended after `content` returns is taken from the
    /// paint list. User shapes, including direct painter output, are untouched.
    fn scroll<T>(
        &self,
        body: Rect,
        interactive: bool,
        content: impl FnOnce(&mut Ui, Rect) -> T,
    ) -> ScrollBuilt<T> {
        let mut ui = Ui::new(
            self.context.clone(),
            self.owner.with("scroll-ui"),
            UiBuilder::new()
                .layer_id(self.layer())
                .max_rect(body)
                .layout(Layout::top_down(Align::Min)),
        );
        ui.set_clip_rect(body.intersect(self.local_screen()));
        ui.spacing_mut().item_spacing.y = BODY_GAP;

        if !interactive {
            ui.disable();
        }

        // There is no usable scroll viewport in this case. Still execute the
        // scoped layout, disabled and clipped, without asking native scrollbar
        // geometry to operate on an empty track.
        if !body.is_positive() {
            ui.disable();
            let clip = ui.clip_rect();
            let viewport = Rect::from_min_size(egui::Pos2::ZERO, body.size());
            let inner = content(&mut ui, viewport);
            return ScrollBuilt {
                id: None,
                inner,
                natural: ui.min_size(),
                layout_offset: Vec2::ZERO,
                end_offset: Vec2::ZERO,
                clip,
                chrome: Vec::new(),
            };
        }

        {
            let scroll = &mut ui.spacing_mut().scroll;
            // Floating bars keep body wrapping independent of bar animation.
            scroll.floating = true;
            scroll.handle_min_length = scroll.handle_min_length.min(body.height());
            scroll.bar_width = scroll.bar_width.min(body.width());
            scroll.bar_inner_margin = scroll.bar_inner_margin.min(body.width() * 0.25);
            scroll.bar_outer_margin = scroll.bar_outer_margin.min(body.width() * 0.25);
        }

        let mut marker = None;
        let mut layout_offset = Vec2::ZERO;
        let mut content_clip = ui.clip_rect();

        let output = ScrollArea::vertical()
            .id_salt("scroll")
            .max_height(body.height())
            .min_scrolled_height(0.0)
            .auto_shrink([false, false])
            .content_margin(0.0)
            .scroll_source(if interactive {
                ScrollSource::default()
            } else {
                ScrollSource::NONE
            })
            .show_viewport(&mut ui, |ui, viewport| {
                layout_offset = viewport.min.to_vec2();
                content_clip = ui.clip_rect();
                let inner = content(ui, viewport);
                marker = Some(paint_marker(&self.context, self.layer()));
                inner
            });

        let chrome = take_scroll_chrome(
            &self.context,
            self.layer(),
            marker.expect("ScrollArea content was called"),
        );

        ScrollBuilt {
            id: Some(output.id),
            inner: output.inner,
            natural: output.content_size,
            layout_offset,
            end_offset: output.state.unclamped_offset(),
            clip: content_clip,
            chrome,
        }
    }

    fn drive_scroll(&self) -> bool {
        let Some(geometry) = self.geometry else {
            return false;
        };
        if !geometry.body.is_positive() {
            return false;
        }

        // No borrowed Ui, user closure, user widgets, or user drawing is retained
        // or replayed here. Native ScrollArea receives its measured extent.
        let output = self.scroll(geometry.body, true, |ui, _viewport| {
            ui.expand_to_include_rect(Rect::from_min_size(ui.max_rect().min, geometry.natural));
        });

        output.end_offset != geometry.offset || output.clip != geometry.content_clip
    }

    fn pointer_layer_is_ours(&self) -> bool {
        self.context
            .input(|input| input.pointer.hover_pos())
            .is_some_and(|point| self.context.layer_id_at(point) == Some(self.layer()))
    }

    fn absorb_scroll(&self) {
        // Native children/popups get their opportunity first. Once input reaches
        // this modal boundary, unused wheel motion must not reach background
        // ScrollAreas, including when the body is already at either bound.
        self.context.input_mut(|input| {
            input.smooth_scroll_delta = Vec2::ZERO;
            input
                .events
                .retain(|event| !matches!(event, Event::MouseWheel { .. }));
        });
    }

    /// Offer an event before background handlers.
    ///
    /// Popup and targeted child handlers should precede the shell. Accept a
    /// returned close request by updating caller-owned state and refreshing.
    pub fn handle(&mut self, routed: &RoutedInput) -> DialogIntent {
        self.assert_pass();
        let intent = self.handle_event(routed);
        self.save();
        intent
    }

    fn handle_event(&mut self, routed: &RoutedInput) -> DialogIntent {
        let mut intent = DialogIntent::default();

        if self.state.restore_pending && self.top() {
            self.absorb_scroll();
        }
        if !self.state.open || !self.top() {
            self.state.space_armed = false;
            return intent;
        }

        self.repair_focus();

        let focused = self.context.memory(|memory| memory.focused());
        if let Some(id) = focused
            && self.own_eligible(id)
        {
            self.state.last_focus = Some(id);
        }
        if focused != Some(self.close_id()) {
            self.state.space_armed = false;
        }

        let event = match &routed.trigger {
            InputTrigger::Time => {
                intent.layout_changed = self.drive_scroll();
                if !Popup::is_any_open(&self.context) || self.pointer_layer_is_ours() {
                    self.absorb_scroll();
                }
                if intent.layout_changed {
                    self.context.request_repaint();
                }
                return intent;
            }
            InputTrigger::Event { event, .. } => event,
        };

        if let Event::Key {
            key: Key::Escape,
            pressed: true,
            modifiers,
            ..
        } = event
            && !Popup::is_any_open(&self.context)
            && self
                .context
                .input_mut(|input| input.consume_key(*modifiers, Key::Escape))
        {
            // Consuming the active input event prevents a newly exposed lower
            // dialog from dismissing on this same Escape.
            intent.close_requested = Some(DialogClose::Escape);
            return intent;
        }

        if let Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed: true,
            modifiers,
            ..
        } = event
        {
            let neutral =
                routed.target == Some(self.backdrop_id()) || routed.target == Some(self.panel_id());

            if neutral && let Some(id) = self.state.last_focus.filter(|id| self.own_eligible(*id)) {
                // A neutral press must not make declining a close lose focus.
                self.context.memory_mut(|memory| memory.request_focus(id));
            }

            if !modifiers.ctrl
                && routed.target == Some(self.backdrop_id())
                && self.geometry.is_some_and(|geometry| {
                    !geometry.outer.contains(self.transform().inverse() * *pos)
                })
                && consume_event(&self.context, event)
            {
                // Radix pointer-down-outside semantics: not release, right
                // button, or Ctrl+primary.
                intent.close_requested = Some(DialogClose::Backdrop);
                return intent;
            }
        }

        if let Event::PointerButton {
            button: PointerButton::Primary,
            pressed: false,
            ..
        } = event
            && routed.target == Some(self.close_id())
            && self
                .context
                .read_response(self.close_id())
                .is_some_and(|response| {
                    response.enabled() && response.clicked_by(PointerButton::Primary)
                })
            && consume_event(&self.context, event)
        {
            intent.close_requested = Some(DialogClose::CloseButton);
            return intent;
        }

        if routed.target == Some(self.close_id()) && focused == Some(self.close_id()) {
            match event {
                Event::Key {
                    key: Key::Enter,
                    pressed: true,
                    repeat: false,
                    ..
                } if consume_event(&self.context, event) => {
                    intent.close_requested = Some(DialogClose::CloseButton);
                    return intent;
                }
                Event::Key {
                    key: Key::Space,
                    pressed: true,
                    repeat: false,
                    ..
                } if consume_event(&self.context, event) => {
                    self.state.space_armed = true;
                }
                Event::Key {
                    key: Key::Space,
                    pressed: false,
                    ..
                } => {
                    let armed = std::mem::take(&mut self.state.space_armed);
                    if armed && consume_event(&self.context, event) {
                        intent.close_requested = Some(DialogClose::CloseButton);
                        return intent;
                    }
                }
                _ => {}
            }
        }
        if matches!(event, Event::WindowFocused(false)) {
            self.state.space_armed = false;
        }
        if matches!(
            event,
            Event::MouseWheel { .. } | Event::PointerMoved(_) | Event::PointerButton { .. }
        ) {
            intent.layout_changed = self.drive_scroll();
            if !Popup::is_any_open(&self.context) || self.pointer_layer_is_ours() {
                self.absorb_scroll();
            }
        }
        if intent.layout_changed {
            self.context.request_repaint();
        }
        intent
    }

    /// Reconstruct and submit the final explicit drawing.
    ///
    /// The callback runs once. If it changes measured geometry, painting fails:
    /// perform that change and refresh before exhausting the input cursor instead.
    pub fn paint<T>(
        self,
        content: impl FnOnce(&mut DialogUi<'_>) -> T,
    ) -> Result<(), compositing::Error> {
        self.assert_pass();
        let Some(geometry) = self.geometry else {
            return Ok(());
        };
        let header = self.header.as_ref().expect("measured header");
        let transform = self.transform();

        // The host derives the Group transform from this layer's callback rect.
        // Keep the same transform for ordinary painting and input registration.
        self.context.set_transform_layer(self.layer(), transform);
        let painter = self.context.layer_painter(self.layer());
        let shell_slot = painter.add(Shape::Noop);
        let built = self.build(geometry.outer, self.state.open, header, content);

        if (built.geometry.natural - geometry.natural).length() > 0.01
            || (built.geometry.outer.height() - geometry.outer.height()).abs() > 0.01
        {
            return Err(compositing::Error::InvalidGroup(
                "dialog geometry changed during paint; refresh before exhausting input",
            ));
        }

        let opacity = self.state.fade.value(self.now);
        let screen = self.local_screen();
        if !screen.is_positive() {
            return Ok(());
        }
        let clip = screen;
        let colors = &self.dialog.theme.colors;
        let mut items = Vec::new();

        // Tailwind shadow-lg's two black/10 shadows. Shrinking the source
        // rectangles expresses its negative spread without clipping the blur.
        for (shrink, offset, blur) in [(3.0, [0, 10], 15), (4.0, [0, 4], 6)] {
            let shadow = Shadow {
                offset,
                blur,
                spread: 0,
                color: Color32::from_black_alpha(26),
            };
            items.push(shape_item(
                clip,
                shadow
                    .as_shape(geometry.outer.shrink(shrink), CornerRadius::same(RADIUS))
                    .into(),
            ));
        }
        items.push(shape_item(
            clip,
            Shape::rect_filled(
                geometry.outer,
                f32::from(RADIUS),
                colors.background.background,
            ),
        ));
        items.push(shape_item(
            clip,
            Shape::rect_stroke(
                geometry.outer,
                f32::from(RADIUS),
                Stroke::new(1.0, colors.border),
                StrokeKind::Inside,
            ),
        ));

        let header_origin = geometry.outer.min + vec2(INSET, INSET);
        let text_width = geometry.outer.width() - 2.0 * INSET;
        let text_position = |galley: &Galley, y: f32| {
            pos2(
                if header.centered {
                    header_origin.x + text_width * 0.5
                } else {
                    header_origin.x
                },
                y,
            ) - vec2(
                if header.centered {
                    galley.rect.center().x
                } else {
                    galley.rect.min.x
                },
                galley.rect.min.y,
            )
        };
        items.push(shape_item(
            clip,
            Shape::galley(
                text_position(&header.title, header_origin.y),
                header.title.clone(),
                colors.background.foreground,
            ),
        ));
        if let Some(description) = &header.description {
            items.push(shape_item(
                clip,
                Shape::galley(
                    text_position(description, header_origin.y + header.title.size().y + 8.0),
                    description.clone(),
                    colors.muted.foreground,
                ),
            ));
        }

        for group in &built.groups {
            if let Some(group) = clip_group(group, geometry.content_clip.intersect(clip))? {
                items.push(Item::Group(group));
            }
        }

        items.extend(built.scroll_chrome.into_iter().map(Item::Shape));

        let close = close_rect(geometry.outer);
        let focused = self.state.open
            && self
                .context
                .memory(|memory| memory.has_focus(self.close_id()));
        if focused {
            items.push(shape_item(
                clip,
                Shape::rect_stroke(
                    close.expand(2.0),
                    2.0,
                    Stroke::new(2.0, colors.background.background),
                    StrokeKind::Outside,
                ),
            ));
            items.push(shape_item(
                clip,
                Shape::rect_stroke(
                    close.expand(4.0),
                    2.0,
                    Stroke::new(2.0, colors.ring),
                    StrokeKind::Outside,
                ),
            ));
        }

        let hovered = self.state.open && built.close.hovered();
        let ink = colors
            .background
            .foreground
            .gamma_multiply(if hovered || focused { 1.0 } else { 0.7 });
        let icon = close.shrink(4.0);
        items.push(shape_item(
            clip,
            Shape::line_segment(
                [icon.left_top(), icon.right_bottom()],
                Stroke::new(4.0 / 3.0, ink),
            ),
        ));
        items.push(shape_item(
            clip,
            Shape::line_segment(
                [icon.right_top(), icon.left_bottom()],
                Stroke::new(4.0 / 3.0, ink),
            ),
        ));

        // Bounds include complete shadow/ring pixels, not merely the panel.
        let content = Group::new(geometry.outer.expand(32.0), opacity, items)?;
        let overlay = Group::new(
            screen,
            self.state.fade.values(self.now)[1],
            vec![shape_item(
                screen,
                Shape::rect_filled(screen, 0.0, Color32::from_black_alpha(128)),
            )],
        )?;
        let bounds = screen.union(content.bounds());
        let root = Group::new(
            bounds,
            1.0,
            vec![Item::Group(overlay), Item::Group(content)],
        )?;
        if self.state.restore_pending || self.state.fade.moving(self.now) {
            self.context.request_repaint();
        }
        let callback_slot = paint_marker(&self.context, self.layer());
        compositing::paint(&painter, root)?;
        // Submit our known callback below direct body painting. This does not
        // capture the caller's ordinary egui shapes into the animation group.
        self.context.graphics_mut(|graphics| {
            let list = graphics.entry(self.layer());
            let callback = list
                .all_entries()
                .nth(callback_slot.0)
                .cloned()
                .expect("submitted dialog callback");
            list.reset_shape(callback_slot);
            list.mutate_shape(shell_slot, |shape| *shape = callback);
        });
        Ok(())
    }

    /// Revoke this layer and erase its transient state without restoring focus.
    ///
    /// Call during dispatch, before abandoning a live registration. Unlike an
    /// accepted close, explicit removal does not retain an exit animation.
    pub fn unregister(mut self) {
        self.assert_pass();
        self.revoke(false);
        if let Some(id) = self.state.scroll_id.take() {
            self.context
                .data_mut(|data| data.remove::<egui::scroll_area::State>(id));
        }
        self.context
            .set_transform_layer(self.layer(), TSTransform::IDENTITY);
        self.context.data_mut(|data| data.remove::<State>(self.key));
    }
}

fn eligible(response: &Response) -> bool {
    response.enabled()
        && response.visible()
        && response.sense.contains(Sense::FOCUSABLE)
        && response.interact_rect.is_positive()
}

fn close_rect(outer: Rect) -> Rect {
    let size = outer.size().min(Vec2::splat(16.0)).max(Vec2::ZERO);
    let inset = vec2(
        (outer.width() - size.x).clamp(0.0, 17.0),
        (outer.height() - size.y).clamp(0.0, 17.0),
    );
    Rect::from_min_size(
        pos2(outer.right() - inset.x - size.x, outer.top() + inset.y),
        size,
    )
}

fn bounded_inset(rect: Rect, inset: f32) -> Rect {
    rect.shrink2(Vec2::splat(inset).min(rect.size().max(Vec2::ZERO) * 0.5))
}

fn consume_event(context: &Context, event: &Event) -> bool {
    context.input_mut(|input| {
        if let Some(index) = input.events.iter().position(|candidate| candidate == event) {
            input.events.remove(index);
            true
        } else {
            false
        }
    })
}

fn paint_marker(context: &Context, layer: LayerId) -> ShapeIdx {
    context.graphics_mut(|graphics| graphics.entry(layer).next_idx())
}

fn take_scroll_chrome(context: &Context, layer: LayerId, start: ShapeIdx) -> Vec<ClippedShape> {
    context.graphics_mut(|graphics| {
        let list = graphics.entry(layer);
        let end = list.next_idx();
        let shapes = list.all_entries().skip(start.0).cloned().collect();
        for index in start.0..end.0 {
            list.reset_shape(ShapeIdx(index));
        }
        shapes
    })
}

fn make_header(context: &Context, dialog: &Dialog<'_>, width: f32, centered: bool) -> Header {
    let requested_family = FontFamily::Name(TITLE_FAMILY.into());
    let title_family = if context.fonts(|fonts| fonts.families().contains(&requested_family)) {
        requested_family
    } else {
        FontFamily::Proportional
    };

    let layout = |text: &str, size, line_height, family, color| {
        let mut job = LayoutJob::default();
        job.wrap.max_width = width;
        job.halign = if centered { Align::Center } else { Align::Min };
        job.append(
            text,
            0.0,
            TextFormat {
                font_id: FontId::new(size, family),
                color,
                line_height: Some(line_height),
                ..Default::default()
            },
        );
        context.fonts_mut(|fonts| fonts.layout_job(job))
    };

    let title = layout(
        dialog.title,
        18.0,
        18.0,
        title_family,
        dialog.theme.colors.background.foreground,
    );
    let description = dialog
        .description
        .filter(|text| !text.is_empty())
        .map(|text| {
            layout(
                text,
                14.0,
                20.0,
                FontFamily::Proportional,
                dialog.theme.colors.muted.foreground,
            )
        });
    let height = title.size().y
        + description
            .as_ref()
            .map_or(0.0, |description| 8.0 + description.size().y);

    Header {
        title,
        description,
        height,
        centered,
    }
}

fn shape_item(clip_rect: Rect, shape: Shape) -> Item {
    Item::Shape(ClippedShape {
        clip_rect,
        shape,
        transform_after_tessellation: TSTransform::IDENTITY,
    })
}

/// Clip recursively without flattening the child's isolation boundaries.
fn clip_group(group: &Group, clip: Rect) -> Result<Option<Group>, compositing::Error> {
    let bounds = group.bounds().intersect(clip);
    if !bounds.is_positive() {
        return Ok(None);
    }
    let mut items = Vec::new();
    for item in group.items() {
        match item {
            Item::Shape(shape) => {
                let mut shape = shape.clone();
                shape.clip_rect = shape.clip_rect.intersect(clip);
                if shape.clip_rect.is_positive() {
                    items.push(Item::Shape(shape));
                }
            }
            Item::Group(child) => {
                if let Some(child) = clip_group(child, clip)? {
                    items.push(Item::Group(child));
                }
            }
        }
    }
    Group::new(bounds, group.opacity(), items).map(Some)
}

// CSS ease = cubic-bezier(0.25, 0.1, 0.25, 1).
fn css_ease(x: f32) -> f32 {
    if x == 0.0 || x == 1.0 {
        return x;
    }
    let (mut low, mut high) = (0.0_f32, 1.0_f32);
    for _ in 0..16 {
        let t = (low + high) * 0.5;
        let bx = 0.75 * (1.0 - t).powi(2) * t + 0.75 * (1.0 - t) * t * t + t * t * t;
        if bx < x {
            low = t;
        } else {
            high = t;
        }
    }
    let t = (low + high) * 0.5;
    0.3 * (1.0 - t).powi(2) * t + 3.0 * (1.0 - t) * t * t + t * t * t
}
