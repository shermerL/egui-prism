//! Controlled, single-value selects for ordered egui input.
//!
//! Register every live select before dispatch. Offer **every** routed event to
//! every registration, including untargeted releases, time opportunities, and
//! window-focus events. Run the select's handler before background scroll
//! handlers. Paint after draining the input cursor.
//!
//! A registration borrows the theme and options, not the value or the `Ui`.
//! Option values must be unique. Use a stable, explicitly supplied widget ID.
//!
//! The popup uses layout-only modal registration for current-pass hit testing
//! and its barrier. Rows retain independent responses and focus policies: an
//! input-owner binding would alias their hit results.
//!
//! Geometry and behavior adapt the workspace's pinned shadcn/Radix references.
//! Icon paths adapt Lucide; the workspace contains the applicable notices.

use std::{
    collections::HashSet,
    hash::{Hash, Hasher},
    sync::Arc,
    time::Duration,
};

use crate::{
    compositing,
    egui::{self, Id, LayerId, Pos2, Rect, Response},
    theme::Theme,
};

mod paint;

use crate::transition;

const ROW_HEIGHT: f32 = 32.0;
const BORDER: f32 = 1.0;
const VIEWPORT_PADDING: f32 = 4.0;
const CONTENT_INSET: f32 = BORDER + VIEWPORT_PADDING;
const SCROLL_BUTTON_HEIGHT: f32 = 24.0;
const REPEAT_SECONDS: f64 = 0.050;
const SEARCH_SECONDS: f64 = 1.0;

/// A flat option. Values, rather than positions or labels, identify options.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectItem<T> {
    /// The caller-owned value copied on commit.
    pub value: T,
    /// Visible text, also used for typeahead and accessibility.
    pub label: String,
    /// Disabled options remain visible but cannot become active or be selected.
    pub disabled: bool,
}

impl<T> SelectItem<T> {
    /// Create an enabled option.
    pub fn new(value: T, label: impl Into<String>) -> Self {
        Self {
            value,
            label: label.into(),
            disabled: false,
        }
    }

    /// Set option enablement.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Reference trigger heights.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SelectSize {
    /// A 36-point trigger.
    #[default]
    Default,
    /// A 32-point trigger.
    Small,
}

/// A select awaiting frame-local registration.
#[must_use = "register the select before dispatching ordered input"]
pub struct Select<'a, T> {
    theme: &'a Theme,
    id: Id,
    items: &'a [SelectItem<T>],
    placeholder: String,
    size: SelectSize,
    width: Option<f32>,
    invalid: bool,
}

impl<'a, T: Clone + Eq + Hash> Select<'a, T> {
    /// Construct a controlled select with an explicit stable ID.
    pub fn new(theme: &'a Theme, id: Id, items: &'a [SelectItem<T>]) -> Self {
        Self {
            theme,
            id,
            items,
            placeholder: String::new(),
            size: SelectSize::Default,
            width: None,
            invalid: false,
        }
    }

    /// Text shown only when the caller's value is `None`.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Choose the reference trigger height.
    pub fn size(mut self, size: SelectSize) -> Self {
        self.size = size;
        self
    }

    /// Set an explicit trigger width. The default fits the displayed value.
    pub fn width(mut self, width: f32) -> Self {
        assert!(
            width.is_finite() && width >= 48.0,
            "Select width must be finite and at least 48 points"
        );
        self.width = Some(width);
        self
    }

    /// Set invalid styling and accessibility metadata.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Allocate and register the trigger and any live popup.
    ///
    /// This never changes `selected`. Parent enablement and visibility come from
    /// `ui`. Capture the submission painter before entering disabled UI scopes.
    pub fn register(self, ui: &mut egui::Ui, selected: &Option<T>) -> SelectRegistration<'a, T> {
        assert!(
            ui.ctx().options(|options| options.ordered_input),
            "Select requires ordered input"
        );

        let mut values = HashSet::with_capacity(self.items.len());
        assert!(
            self.items.iter().all(|item| values.insert(&item.value)),
            "Select option values must be unique"
        );

        let viewport = ui.ctx().viewport_id();
        let key = self.id.with(("egui-prism/select", viewport));
        let ids: Vec<_> = self
            .items
            .iter()
            .map(|item| option_id(key, &item.value))
            .collect();
        assert_eq!(
            ids.iter().copied().collect::<HashSet<_>>().len(),
            ids.len(),
            "Select option ID collision"
        );
        let mut layout_hasher = std::collections::hash_map::DefaultHasher::new();
        for (id, item) in ids.iter().zip(self.items) {
            id.hash(&mut layout_hasher);
            item.disabled.hash(&mut layout_hasher);
        }
        let layout_identity = layout_hasher.finish();

        let mut font = egui::TextStyle::Body.resolve(ui.style());
        font.size = 14.0;
        let measure = |text: &str| {
            let mut job = egui::text::LayoutJob::simple(
                text.to_owned(),
                font.clone(),
                egui::Color32::PLACEHOLDER,
                f32::INFINITY,
            );
            job.wrap.max_rows = 1;
            for section in &mut job.sections {
                section.format.line_height = Some(20.0);
            }
            ui.fonts_mut(|fonts| fonts.layout_job(job))
        };
        let galleys: Vec<_> = self.items.iter().map(|item| measure(&item.label)).collect();
        let placeholder_galley = measure(&self.placeholder);
        let displayed_width = match selected {
            None => placeholder_galley.size().x,
            Some(value) => self
                .items
                .iter()
                .position(|item| &item.value == value)
                .map_or(0.0, |index| galleys[index].size().x),
        };
        // Chevron + gap + horizontal padding + borders.
        let width = self
            .width
            .unwrap_or(displayed_width + 16.0 + 8.0 + 24.0 + 2.0);
        let height = match self.size {
            SelectSize::Default => 36.0,
            SelectSize::Small => 32.0,
        };
        let (_, rect) = ui.allocate_space(egui::vec2(width, height));
        assert!(
            rect.is_finite() && rect.is_positive(),
            "Select requires finite positive layout bounds"
        );

        let response = ui.interact(rect, self.id, egui::Sense::click_and_drag());
        ui.ctx().set_input_rounding(self.id, 8.0);
        if response.enabled() && response.visible() {
            ui.memory_mut(|memory| {
                memory.register_focus_policy(self.id, response.layer_id, focus_filter(false));
            });
        }

        let screen = ui.ctx().content_rect();
        let global_trigger = ui
            .ctx()
            .layer_transform_to_global(response.layer_id)
            .map_or(rect, |transform| transform * rect);

        let registration = SelectRegistration {
            response,
            theme: self.theme,
            items: self.items,
            placeholder: self.placeholder,
            invalid: self.invalid,
            key,
            viewport,
            frame: ui.ctx().cumulative_frame_nr(),
            pass: ui.ctx().cumulative_pass_nr(),
            clip: ui.clip_rect(),
            screen,
            global_trigger,
            ids,
            layout_identity,
            galleys,
            placeholder_galley,
            removed: false,
        };

        let mut state = registration.load();
        if registration.frame > state.last_frame.saturating_add(1) {
            registration.close(&mut state, false);
            state = State::new(registration.frame, screen);
        }

        // Only a current-pass enclosing scope can be restored on dismissal.
        if let Some(previous) = state.previous_modal
            && registration.top_modal() == Some(previous)
        {
            state.previous_modal_pass = registration.pass;
        }

        if state.screen != screen {
            registration.close(&mut state, true);
            state.screen = screen;
        }

        if !registration.usable() || !registration.response.ctx.input(|input| input.focused) {
            registration.close(&mut state, false);
            state.reset_transient();
            state.fade = Fade::default();
        } else if state.open {
            let top = registration.top_modal();
            // Ordered begin_pass clears the modal stack. None here is not an
            // explicit dismissal: renew our live popup before dispatch.
            let scope_valid = top.is_none()
                || top == Some(registration.popup_layer())
                || (top == state.previous_modal && state.previous_modal_pass == registration.pass);
            if !scope_valid || !registration.owns_focus(&state) {
                registration.close(&mut state, false);
            }
        }

        let trigger_focused = registration.response.has_focus();
        if trigger_focused && !state.trigger_focused {
            state.keyboard = true;
        }
        state.trigger_focused = trigger_focused;
        state.last_frame = registration.frame;

        let reconciled = registration.reconcile(&mut state, selected);
        if state.open {
            registration.register_popup(&state, selected);
            if reconciled {
                registration.focus_active(&state);
            }
            if state.repeat_direction != 0 {
                registration.response.ctx.request_input_time();
            }
        }

        state.expire_search(registration.now());
        registration.metadata(selected, &state);
        registration.save(state);
        registration
    }
}

/// Pass-local geometry and interaction authority for one select.
///
/// Call `handle` for every routed event, before background scroll handlers.
/// A drag-through release may have no target.
pub struct SelectRegistration<'a, T> {
    /// The trigger response and its stable caller-supplied ID.
    pub response: Response,
    theme: &'a Theme,
    items: &'a [SelectItem<T>],
    placeholder: String,
    invalid: bool,
    key: Id,
    viewport: egui::ViewportId,
    frame: u64,
    pass: u64,
    clip: Rect,
    screen: Rect,
    global_trigger: Rect,
    ids: Vec<Id>,
    layout_identity: u64,
    galleys: Vec<Arc<egui::Galley>>,
    placeholder_galley: Arc<egui::Galley>,
    removed: bool,
}

#[derive(Clone, Copy, Default)]
struct Fade {
    from: f32,
    to: f32,
    start: f64,
}

impl Fade {
    fn value(self, now: f64) -> f32 {
        let x = ((now - self.start) / 0.15).clamp(0.0, 1.0) as f32;
        egui::lerp(self.from..=self.to, popup_ease(x))
    }

    fn retarget(&mut self, now: f64, to: f32) {
        *self = Self {
            from: self.value(now),
            to,
            start: now,
        };
    }

    fn moving(self, now: f64) -> bool {
        self.from != self.to && now < self.start + 0.15
    }
}

fn popup_ease(x: f32) -> f32 {
    if x == 0.0 || x == 1.0 {
        return x;
    }
    // CSS ease: invert cubic-bezier(.25, .1, .25, 1)'s x coordinate.
    let (mut low, mut high) = (0.0_f32, 1.0_f32);
    for _ in 0..20 {
        let t = (low + high) * 0.5;
        let u = 1.0 - t;
        let bx = 0.75 * u * u * t + 0.75 * u * t * t + t * t * t;
        if bx < x {
            low = t;
        } else {
            high = t;
        }
    }
    let t = (low + high) * 0.5;
    let u = 1.0 - t;
    0.3 * u * u * t + 3.0 * u * t * t + t * t * t
}

// Release authority is separate from hit/capture routing. Only a live primary
// press can create this token; the trigger origin also suppresses its first click.
#[derive(Clone, Copy)]
enum PrimaryGesture {
    Trigger(Pos2),
    Popup,
}

// No T, borrowed data, galleys, responses, UIs, or renderer resources enter the
// Context. Selection identity is only an external-change detector; displayed
// text, checked state, and committed values always come from the caller.
#[derive(Clone)]
struct State {
    last_frame: u64,
    screen: Rect,
    open: bool,
    active: Option<Id>,
    observed_selection: Option<Id>,
    layout_identity: u64,
    wrapper_height: f32,
    bottom_anchored: bool,
    scroll: f32,
    primary_gesture: Option<PrimaryGesture>,
    keyboard: bool,
    trigger_focused: bool,
    previous_modal: Option<LayerId>,
    previous_modal_pass: u64,
    search: String,
    search_at: f64,
    repeat_direction: i8,
    repeat_at: f64,
    fade: Fade,
}

impl State {
    fn new(frame: u64, screen: Rect) -> Self {
        Self {
            last_frame: frame,
            screen,
            open: false,
            active: None,
            observed_selection: None,
            layout_identity: 0,
            wrapper_height: 0.0,
            bottom_anchored: false,
            scroll: 0.0,
            primary_gesture: None,
            keyboard: true,
            trigger_focused: false,
            previous_modal: None,
            previous_modal_pass: 0,
            search: String::new(),
            search_at: f64::NEG_INFINITY,
            repeat_direction: 0,
            repeat_at: f64::INFINITY,
            fade: Fade::default(),
        }
    }

    fn expire_search(&mut self, now: f64) {
        if now - self.search_at >= SEARCH_SECONDS {
            self.search.clear();
        }
    }

    fn stop_repeat(&mut self) {
        self.repeat_direction = 0;
        self.repeat_at = f64::INFINITY;
    }

    fn reset_transient(&mut self) {
        self.primary_gesture = None;
        self.search.clear();
        self.stop_repeat();
        self.scroll = 0.0;
        self.active = None;
    }
}

struct PopupGeometry {
    menu: Rect,
    viewport: Rect,
    rows: Vec<Rect>,
    up: Option<Rect>,
    down: Option<Rect>,
    max_scroll: f32,
}

impl PopupGeometry {
    fn animation_transform(&self, progress: f32) -> egui::emath::TSTransform {
        let scale = 0.95 + 0.05 * progress;
        egui::emath::TSTransform {
            scaling: scale,
            translation: self.menu.center().to_vec2() * (1.0 - scale),
        }
    }
}

fn option_id(key: Id, value: &impl Hash) -> Id {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    key.with(("option", hasher.finish()))
}

fn focus_filter(open: bool) -> egui::EventFilter {
    egui::EventFilter {
        tab: open,
        horizontal_arrows: true,
        vertical_arrows: true,
        escape: true,
    }
}

fn unmodified(modifiers: egui::Modifiers) -> bool {
    !modifiers.ctrl && !modifiers.alt && !modifiers.command && !modifiers.mac_cmd
}

impl<T: Clone + Eq + Hash> SelectRegistration<'_, T> {
    fn assert_pass(&self) {
        assert_eq!(
            self.viewport,
            self.response.ctx.viewport_id(),
            "Select viewport"
        );
        assert_eq!(
            self.pass,
            self.response.ctx.cumulative_pass_nr(),
            "Select registrations are pass-local"
        );
    }

    fn now(&self) -> f64 {
        self.response.ctx.input(|input| input.time)
    }

    fn load(&self) -> State {
        self.response
            .ctx
            .data(|data| data.get_temp::<State>(self.key))
            .unwrap_or_else(|| State::new(self.frame, self.screen))
    }

    fn save(&self, state: State) {
        self.response
            .ctx
            .data_mut(|data| data.insert_temp(self.key, state));
    }

    fn popup_layer(&self) -> LayerId {
        LayerId::new(egui::Order::Foreground, self.key.with("popup-layer"))
    }

    fn popup_id(&self) -> Id {
        self.key.with("popup-focus")
    }

    fn up_id(&self) -> Id {
        self.key.with("scroll-up")
    }

    fn down_id(&self) -> Id {
        self.key.with("scroll-down")
    }

    fn top_modal(&self) -> Option<LayerId> {
        self.response.ctx.memory(|memory| memory.top_modal_layer())
    }

    fn focused_id(&self) -> Option<Id> {
        self.response.ctx.memory(|memory| memory.focused())
    }

    fn owns_focus(&self, state: &State) -> bool {
        let Some(id) = self.focused_id() else {
            return false;
        };
        // The previously active ID remains ours while options are reconciled,
        // even if that row was removed or disabled. Require an exact focus
        // match: host surrender/transfer must not be mistaken for a mutation.
        if id == self.popup_id() || state.active == Some(id) {
            return true;
        }
        if self
            .index(Some(id))
            .is_some_and(|index| !self.items[index].disabled)
        {
            return true;
        }
        let geometry = self.geometry(state);
        (id == self.up_id() && geometry.up.is_some())
            || (id == self.down_id() && geometry.down.is_some())
    }

    fn focus_active(&self, state: &State) {
        let id = state.active.unwrap_or_else(|| self.popup_id());
        if self.focused_id() != Some(id) {
            self.response
                .ctx
                .memory_mut(|memory| memory.request_focus(id));
        }
    }

    fn current_response(&self) -> Response {
        self.response
            .ctx
            .read_response(self.response.id)
            .unwrap_or_else(|| self.response.clone())
    }

    fn usable(&self) -> bool {
        !self.removed && self.response.enabled() && self.response.visible()
    }

    fn index(&self, id: Option<Id>) -> Option<usize> {
        id.and_then(|id| self.ids.iter().position(|candidate| *candidate == id))
    }

    fn selected_index(&self, selected: &Option<T>) -> Option<usize> {
        selected
            .as_ref()
            .and_then(|value| self.items.iter().position(|item| &item.value == value))
    }

    fn selected_id(&self, selected: &Option<T>) -> Option<Id> {
        selected.as_ref().map(|value| option_id(self.key, value))
    }

    fn first_enabled(&self) -> Option<usize> {
        self.items.iter().position(|item| !item.disabled)
    }

    fn initial_active(&self, selected: &Option<T>) -> Option<Id> {
        self.selected_index(selected)
            .filter(|index| !self.items[*index].disabled)
            .or_else(|| self.first_enabled())
            .map(|index| self.ids[index])
    }

    fn reconcile(&self, state: &mut State, selected: &Option<T>) -> bool {
        let selected_id = self.selected_id(selected);
        let selection_changed = state.observed_selection != selected_id;
        let layout_changed = state.layout_identity != self.layout_identity;
        state.observed_selection = selected_id;
        state.layout_identity = self.layout_identity;

        // None is intentional after pointer leave, not an invalid active row.
        let active_removed = state.active.is_some()
            && self
                .index(state.active)
                .is_none_or(|index| self.items[index].disabled);
        if selection_changed || active_removed {
            state.active = self.initial_active(selected);
        }
        let changed = selection_changed || layout_changed || active_removed;
        if state.open && changed {
            self.place(state, selected);
            self.reveal_active(state);
        }
        changed
    }

    fn available(&self) -> Rect {
        let margin = 10.0_f32
            .min(self.screen.width() * 0.25)
            .min(self.screen.height() * 0.25);
        self.screen.shrink(margin)
    }

    fn full_height(&self) -> f32 {
        self.items.len() as f32 * ROW_HEIGHT + 2.0 * CONTENT_INSET
    }

    fn place(&self, state: &mut State, selected: &Option<T>) {
        let available = self.available();
        let full = self.full_height();
        let anchor = self
            .selected_index(selected)
            .or_else(|| self.first_enabled())
            .unwrap_or(0);
        let offset = CONTENT_INSET + ROW_HEIGHT * (anchor as f32 + 0.5);
        let above = self.global_trigger.center().y - available.top();
        let below = available.height() - above;
        let minimum = (5.0 * ROW_HEIGHT).min(full);

        state.bottom_anchored = offset <= above;
        let height = if state.bottom_anchored {
            let last_padding = if anchor + 1 == self.items.len() {
                VIEWPORT_PADDING
            } else {
                0.0
            };
            offset + below.max(ROW_HEIGHT * 0.5 + last_padding + BORDER)
        } else {
            let first_padding = if anchor == 0 { VIEWPORT_PADDING } else { 0.0 };
            above.max(BORDER + first_padding + ROW_HEIGHT * 0.5) + full - offset
        };
        state.wrapper_height = height.max(minimum).min(available.height()).max(1.0);
        state.scroll = 0.0;

        if !state.bottom_anchored {
            // The initial up button moves the viewport down by 24 points.
            // Its appearance is part of initial placement, not scroll expansion.
            let geometry = self.geometry(state);
            if geometry.max_scroll > 0.0 {
                state.scroll = geometry.max_scroll;
                let geometry = self.geometry(state);
                state.scroll = (geometry.viewport.top()
                    + VIEWPORT_PADDING
                    + ROW_HEIGHT * (anchor as f32 + 0.5)
                    - self.global_trigger.center().y)
                    .clamp(0.0, geometry.max_scroll);
            }
        }
    }

    fn geometry(&self, state: &State) -> PopupGeometry {
        let available = self.available();
        let text_width = self
            .galleys
            .iter()
            .map(|galley| galley.size().x)
            .fold(0.0_f32, f32::max);
        let width = (text_width + 50.0)
            .max(128.0)
            .max(self.global_trigger.width())
            .min(available.width().max(1.0));
        let wrapper_height = state
            .wrapper_height
            .max(1.0)
            .min(available.height().max(1.0));
        let height = self.full_height().min(wrapper_height).max(1.0);
        let left = self.global_trigger.left().clamp(
            available.left(),
            (available.right() - width).max(available.left()),
        );
        // The wrapper, not the content, is anchored. Short content sits at
        // the wrapper's top; it must not be stretched to the available height.
        let top = if state.bottom_anchored {
            available.bottom() - wrapper_height
        } else {
            available.top()
        };
        let menu = Rect::from_min_size(egui::pos2(left, top), egui::vec2(width, height));
        let border = BORDER.min(height * 0.2).min(width * 0.2);
        let inner = menu.shrink(border);
        let scroll_height = self.items.len() as f32 * ROW_HEIGHT + 2.0 * VIEWPORT_PADDING;
        let overflowing = scroll_height > inner.height();
        let band = if overflowing && inner.height() >= 80.0 {
            SCROLL_BUTTON_HEIGHT
        } else {
            0.0
        };
        // At the bottom the down button is absent. This is the stable endpoint,
        // including the up button, rather than the transient two-button extent.
        let max_scroll = if overflowing {
            (scroll_height - inner.height() + band).max(0.0)
        } else {
            0.0
        };
        let scroll = state.scroll.clamp(0.0, max_scroll);
        let up = (band > 0.0 && scroll > 0.0)
            .then(|| Rect::from_min_size(inner.min, egui::vec2(inner.width(), band)));
        let down = (band > 0.0 && scroll < max_scroll).then(|| {
            Rect::from_min_size(
                egui::pos2(inner.left(), inner.bottom() - band),
                egui::vec2(inner.width(), band),
            )
        });
        let viewport = Rect::from_min_max(
            egui::pos2(inner.left(), up.map_or(inner.top(), |rect| rect.bottom())),
            egui::pos2(
                inner.right(),
                down.map_or(inner.bottom(), |rect| rect.top()),
            ),
        );
        let padding = VIEWPORT_PADDING.min(viewport.width() * 0.2);
        let rows = (0..self.items.len())
            .map(|index| {
                Rect::from_min_size(
                    egui::pos2(
                        viewport.left() + padding,
                        viewport.top() + VIEWPORT_PADDING + index as f32 * ROW_HEIGHT - scroll,
                    ),
                    egui::vec2((viewport.width() - 2.0 * padding).max(1.0), ROW_HEIGHT),
                )
            })
            .collect();

        PopupGeometry {
            menu,
            viewport,
            rows,
            up,
            down,
            max_scroll,
        }
    }

    fn scroll_to(&self, state: &mut State, requested: f32) -> bool {
        let geometry = self.geometry(state);
        let next = requested.clamp(0.0, geometry.max_scroll);
        let delta = (next - state.scroll).abs();
        if delta == 0.0 {
            return false;
        }

        state.scroll = next;
        let available_height = self.available().height().max(1.0);
        if state.wrapper_height < available_height {
            let next_height = state.wrapper_height + delta;
            state.wrapper_height = next_height.min(available_height);
            if state.bottom_anchored {
                // Radix absorbs scrolling into upward wrapper growth first.
                state.scroll = (next_height - available_height).max(0.0);
            }
        }
        state.scroll = state.scroll.clamp(0.0, self.geometry(state).max_scroll);
        true
    }

    fn reveal_active(&self, state: &mut State) {
        let Some(index) = self.index(state.active) else {
            return;
        };
        if index == 0 {
            self.scroll_to(state, 0.0);
        } else if index + 1 == self.items.len() {
            self.scroll_to(state, self.geometry(state).max_scroll);
        }

        for _ in 0..3 {
            let geometry = self.geometry(state);
            let row = geometry.rows[index];
            let delta = if row.top() < geometry.viewport.top() {
                row.top() - geometry.viewport.top()
            } else if row.bottom() > geometry.viewport.bottom() {
                row.bottom() - geometry.viewport.bottom()
            } else {
                break;
            };
            if !self.scroll_to(state, state.scroll + delta) {
                break;
            }
        }
    }

    fn register_popup(&self, state: &State, selected: &Option<T>) {
        let ctx = &self.response.ctx;
        let geometry = self.geometry(state);
        // Transform the parts, not the layer: the backdrop must cover the screen.
        let transform = geometry.animation_transform(state.fade.value(self.now()));
        let menu = transform * geometry.menu;
        let viewport = (transform * geometry.viewport).intersect(self.screen);
        let layer = self.popup_layer();
        let owner = self.popup_id();
        let selected_index = self.selected_index(selected);

        let modal = egui::Modal::new(layer.id)
            .area(
                egui::Area::new(layer.id)
                    .order(layer.order)
                    .fixed_pos(menu.min)
                    .default_size(menu.size())
                    .movable(false)
                    .constrain(false)
                    .fade_in(false)
                    .sense(egui::Sense::hover()),
            )
            .frame(egui::Frame::NONE)
            .backdrop_color(egui::Color32::TRANSPARENT)
            .register(ctx, |ui| {
                ui.set_clip_rect(self.screen);
                ui.set_min_size(menu.size());
                let popup = ui.interact(menu, owner, egui::Sense::click_and_drag());
                ctx.set_input_rounding(owner, 8.0 * transform.scaling);
                popup.widget_info(|| {
                    egui::WidgetInfo::selected(egui::Role::ListBox, true, false, &self.placeholder)
                });
                ctx.memory_mut(|memory| {
                    memory.register_focus_policy(owner, layer, focus_filter(true));
                });

                for (index, rect) in geometry.rows.iter().copied().enumerate() {
                    let rect = transform * rect;
                    let mut row_ui = ui.new_child(
                        egui::UiBuilder::new()
                            .scope_id(self.ids[index].with("ui"))
                            .max_rect(rect),
                    );
                    row_ui.set_clip_rect(viewport);
                    if self.items[index].disabled {
                        row_ui.disable();
                    }
                    let response =
                        row_ui.interact(rect, self.ids[index], egui::Sense::click_and_drag());
                    response.widget_info(|| {
                        egui::WidgetInfo::selected(
                            egui::Role::ListBoxOption,
                            !self.items[index].disabled,
                            selected_index == Some(index),
                            &self.items[index].label,
                        )
                    });
                    ctx.set_input_rounding(self.ids[index], 6.0 * transform.scaling);
                    if !self.items[index].disabled {
                        ctx.memory_mut(|memory| {
                            memory.register_focus_policy(
                                self.ids[index],
                                layer,
                                focus_filter(true),
                            );
                        });
                    }
                    // Deliberately no set_input_owner: read_response must keep
                    // this row's current-pass clipped, rounded, covered hit.
                }

                for (id, rect) in [(self.up_id(), geometry.up), (self.down_id(), geometry.down)] {
                    if let Some(rect) = rect {
                        let rect = transform * rect;
                        ui.interact(rect, id, egui::Sense::click_and_drag());
                        ctx.memory_mut(|memory| {
                            memory.register_focus_policy(id, layer, focus_filter(true));
                        });
                    } else {
                        // Preserve content focus when a focused button vanishes
                        // as a consequence of our own scrolling.
                        if self.focused_id() == Some(id) {
                            ctx.memory_mut(|memory| memory.request_focus(owner));
                        }
                        ctx.remove_input_widget(id);
                    }
                }
            });

        // Only the backdrop is aliased. Its press must retain content focus,
        // but no row or scroll-button hit is ever inferred from this owner.
        ctx.set_input_owner(modal.backdrop_response.id, owner);
    }

    fn metadata(&self, selected: &Option<T>, state: &State) {
        let label = self.selected_index(selected).map_or_else(
            || {
                if selected.is_none() {
                    self.placeholder.as_str()
                } else {
                    ""
                }
            },
            |index| self.items[index].label.as_str(),
        );
        self.response.widget_info(|| {
            egui::WidgetInfo::selected(
                egui::Role::ComboBox,
                self.response.enabled(),
                selected.is_some(),
                label,
            )
        });
        self.response
            .ctx
            .accesskit_node_builder(self.response.id, |node| {
                node.set_expanded(state.open);
                node.set_value(label);
                if self.invalid {
                    node.set_invalid(egui::accesskit::Invalid::True);
                } else {
                    node.clear_invalid();
                }
            });
    }

    fn open(&self, state: &mut State, selected: &Option<T>, press: Option<Pos2>) {
        if state.open || !self.usable() {
            return;
        }
        state.previous_modal = self.top_modal();
        state.previous_modal_pass = self.pass;
        state.open = true;
        state.active = self.initial_active(selected);
        state.primary_gesture = press.map(PrimaryGesture::Trigger);
        state.keyboard = press.is_none();
        state.search.clear();
        state.stop_repeat();
        self.place(state, selected);
        self.reveal_active(state);
        state.fade.retarget(self.now(), 1.0);
        self.register_popup(state, selected);
        self.focus_active(state);
        self.response.ctx.request_repaint();
    }

    fn close(&self, state: &mut State, return_focus: bool) {
        if !state.open {
            return;
        }

        let ctx = &self.response.ctx;
        let had_scope = self.top_modal() == Some(self.popup_layer());
        let had_focus = self.owns_focus(state);
        let focused = self.focused_id();
        let previous = state.previous_modal.take();
        let previous_live = state.previous_modal_pass == self.pass;

        state.open = false;
        state.primary_gesture = None;
        state.search.clear();
        state.stop_repeat();
        state.fade.retarget(self.now(), 0.0);

        ctx.remove_input_layer(self.popup_layer());
        if had_focus && let Some(id) = focused {
            ctx.memory_mut(|memory| memory.surrender_focus(id));
        }

        if had_scope
            && self.top_modal().is_none()
            && previous_live
            && let Some(previous) = previous
        {
            ctx.memory_mut(|memory| memory.set_modal_layer(previous));
        }

        if return_focus && had_scope && had_focus && self.usable() {
            let allowed = ctx.memory(|memory| memory.allows_interaction(self.response.layer_id));
            if allowed && self.focused_id().is_none() {
                self.response.request_focus();
                state.trigger_focused = true;
            }
        }
        ctx.request_repaint();
    }

    fn authoritative_hit(&self, id: Id) -> bool {
        self.response.ctx.read_response(id).is_some_and(|response| {
            response.enabled() && response.visible() && response.contains_pointer()
        })
    }

    fn hit_row(&self) -> Option<usize> {
        self.items.iter().enumerate().find_map(|(index, item)| {
            (!item.disabled && self.authoritative_hit(self.ids[index])).then_some(index)
        })
    }

    fn over_menu(&self, state: &State, geometry: &PopupGeometry) -> bool {
        let menu = geometry.animation_transform(state.fade.value(self.now())) * geometry.menu;
        let inside = self.response.ctx.input(|input| {
            input
                .pointer
                .hover_pos()
                .is_some_and(|pos| menu.contains(pos))
        });
        inside
            && (self.authoritative_hit(self.popup_id())
                || self.hit_row().is_some()
                || (geometry.up.is_some() && self.authoritative_hit(self.up_id()))
                || (geometry.down.is_some() && self.authoritative_hit(self.down_id())))
    }

    fn commit(&self, index: usize, selected: &mut Option<T>, response: &mut Response) {
        if !self.items[index].disabled && selected.as_ref() != Some(&self.items[index].value) {
            *selected = Some(self.items[index].value.clone());
            response.mark_changed();
        }
    }

    fn navigate(&self, state: &mut State, key: egui::Key) {
        let enabled: Vec<_> = self
            .items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| (!item.disabled).then_some(index))
            .collect();
        if enabled.is_empty() {
            state.active = None;
            return;
        }
        let current = self
            .index(state.active)
            .and_then(|index| enabled.iter().position(|candidate| *candidate == index));
        let position = match key {
            egui::Key::Home => 0,
            egui::Key::End => enabled.len() - 1,
            egui::Key::ArrowUp => {
                current.map_or(enabled.len() - 1, |position| position.saturating_sub(1))
            }
            egui::Key::ArrowDown => current
                .map_or(0, |position| position.saturating_add(1))
                .min(enabled.len() - 1),
            _ => return,
        };
        state.active = Some(self.ids[enabled[position]]);
        state.stop_repeat();
        self.reveal_active(state);
    }

    fn typeahead(
        &self,
        state: &mut State,
        text: &str,
        selected: &mut Option<T>,
        response: &mut Response,
    ) {
        let now = self.now();
        state.expire_search(now);
        state.search.push_str(&text.to_lowercase());
        state.search_at = now;
        if state.search.is_empty() || self.items.is_empty() {
            return;
        }

        let mut chars = state.search.chars();
        let first = chars.next().unwrap();
        let repeated = chars.all(|character| character == first);
        let needle = if repeated {
            first.to_string()
        } else {
            state.search.clone()
        };
        let current = if state.open {
            self.index(state.active)
        } else {
            self.selected_index(selected)
        };
        let start = current.map_or(0, |index| {
            if needle.chars().count() == 1 {
                (index + 1) % self.items.len()
            } else {
                index
            }
        });
        let found = (0..self.items.len())
            .map(|offset| (start + offset) % self.items.len())
            .find(|index| {
                let item = &self.items[*index];
                !item.disabled && item.label.trim_start().to_lowercase().starts_with(&needle)
            });

        if let Some(index) = found {
            if state.open {
                state.active = Some(self.ids[index]);
                state.stop_repeat();
                self.reveal_active(state);
            } else {
                self.commit(index, selected, response);
            }
        }
    }

    fn update_repeat(&self, state: &mut State, time_opportunity: bool) -> bool {
        let geometry = self.geometry(state);
        let direction = if geometry.up.is_some() && self.authoritative_hit(self.up_id()) {
            -1
        } else if geometry.down.is_some() && self.authoritative_hit(self.down_id()) {
            1
        } else {
            0
        };
        let now = self.now();
        if direction != state.repeat_direction {
            state.repeat_direction = direction;
            state.repeat_at = if direction == 0 {
                f64::INFINITY
            } else {
                now + REPEAT_SECONDS
            };
        }

        let mut changed = false;
        if time_opportunity && direction != 0 && now >= state.repeat_at {
            let steps = ((now - state.repeat_at) / REPEAT_SECONDS).floor() + 1.0;
            changed = self.scroll_to(
                state,
                state.scroll + direction as f32 * ROW_HEIGHT * steps as f32,
            );
            state.repeat_at += steps * REPEAT_SECONDS;
            let geometry = self.geometry(state);
            if !changed
                || (direction < 0 && geometry.up.is_none())
                || (direction > 0 && geometry.down.is_none())
            {
                state.stop_repeat();
            }
        }
        if state.repeat_direction != 0 {
            self.response
                .ctx
                .request_repaint_after(Duration::from_secs_f64((state.repeat_at - now).max(0.0)));
        }
        changed
    }

    /// Handle exactly one current routed event, before background scroll input.
    ///
    /// Consume `changed()` on this returned response before advancing the cursor.
    /// The public `response` field remains the original registration response.
    pub fn handle(&mut self, routed: &egui::RoutedInput, selected: &mut Option<T>) -> Response {
        self.assert_pass();
        let mut response = self.current_response();
        if self.removed {
            return response;
        }

        let mut state = self.load();
        let trigger_focused = response.has_focus();
        if trigger_focused && !state.trigger_focused {
            state.keyboard = true;
        }
        state.trigger_focused = trigger_focused;
        state.expire_search(self.now());

        if !response.enabled()
            || !response.visible()
            || !self.response.ctx.input(|input| input.focused)
        {
            self.close(&mut state, false);
            state.reset_transient();
            self.save(state);
            return response;
        }

        if state.open && (self.top_modal() != Some(self.popup_layer()) || !self.owns_focus(&state))
        {
            // An explicit surrender, transfer, or newer host modal wins.
            self.close(&mut state, false);
            self.save(state);
            return response;
        }

        let reconciled = self.reconcile(&mut state, selected);
        let mut refresh = state.open && reconciled;
        let mut focus_active = refresh;

        match &routed.trigger {
            egui::InputTrigger::Time => {
                if state.open {
                    refresh |= self.update_repeat(&mut state, true);
                }
            }
            egui::InputTrigger::Event { event, .. } => match event {
                egui::Event::WindowFocused(false) | egui::Event::PointerGone => {
                    if matches!(event, egui::Event::WindowFocused(false)) {
                        self.close(&mut state, false);
                    } else if state.open && !state.keyboard {
                        state.active = None;
                        focus_active = true;
                    }
                    state.primary_gesture = None;
                    state.stop_repeat();
                }

                egui::Event::PointerButton {
                    pos,
                    button,
                    pressed,
                    modifiers,
                } => {
                    if state.open {
                        let geometry = self.geometry(&state);
                        let menu = geometry.animation_transform(state.fade.value(self.now()))
                            * geometry.menu;
                        if *pressed {
                            if *button == egui::PointerButton::Primary {
                                state.primary_gesture = None;
                            }
                            // The owner hit outside the menu is the bound
                            // backdrop's authoritative hit, not an expanded
                            // owner rectangle. Dismiss on press, not click.
                            if !menu.contains(*pos) && self.authoritative_hit(self.popup_id()) {
                                self.close(&mut state, true);
                            } else if self.over_menu(&state, &geometry) {
                                if *button == egui::PointerButton::Primary {
                                    state.primary_gesture = Some(PrimaryGesture::Popup);
                                }
                                state.keyboard = false;
                                state.active = self.hit_row().map(|index| self.ids[index]);
                                focus_active = true;
                                self.update_repeat(&mut state, false);
                            }
                        } else if *button == egui::PointerButton::Primary
                            && let Some(gesture) = state.primary_gesture.take()
                        {
                            let prevented = match gesture {
                                PrimaryGesture::Trigger(origin) => {
                                    (origin.x - pos.x).abs() <= 10.0
                                        && (origin.y - pos.y).abs() <= 10.0
                                }
                                PrimaryGesture::Popup => false,
                            };
                            if !prevented {
                                // Capture and routed targets are not part-hit
                                // authority. In particular, releases can be
                                // untargeted after a trigger-to-row drag.
                                if let Some(index) = self.hit_row() {
                                    self.commit(index, selected, &mut response);
                                    self.close(&mut state, true);
                                } else if !menu.contains(*pos)
                                    && self.authoritative_hit(self.popup_id())
                                {
                                    self.close(&mut state, true);
                                }
                            }
                        }
                    } else if *pressed
                        && *button == egui::PointerButton::Primary
                        && !modifiers.ctrl
                        && routed.target == Some(self.response.id)
                        && response.contains_pointer()
                    {
                        self.open(&mut state, selected, Some(*pos));
                    }
                }

                egui::Event::PointerMoved(_) if state.open => {
                    state.keyboard = false;
                    state.active = self.hit_row().map(|index| self.ids[index]);
                    focus_active = true;
                    self.update_repeat(&mut state, false);
                }

                egui::Event::MouseWheel { unit, delta, .. } if state.open => {
                    let geometry = self.geometry(&state);
                    if self.over_menu(&state, &geometry) {
                        let amount = match unit {
                            egui::MouseWheelUnit::Point => delta.y,
                            egui::MouseWheelUnit::Line => delta.y * ROW_HEIGHT,
                            egui::MouseWheelUnit::Page => delta.y * geometry.viewport.height(),
                        };
                        let requested = state.scroll - amount;
                        refresh |= self.scroll_to(&mut state, requested);
                        state.stop_repeat();
                    }
                    // The current ordered event must be consumed before the
                    // host's background scroll handler, also at an endpoint.
                    self.response.ctx.input_mut(|input| {
                        input
                            .events
                            .retain(|event| !matches!(event, egui::Event::MouseWheel { .. }));
                    });
                }

                egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } if (state.open || trigger_focused) && unmodified(*modifiers) => {
                    state.keyboard = true;
                    let was_open = state.open;
                    let searching_space = *key == egui::Key::Space && !state.search.is_empty();

                    if searching_space {
                        self.typeahead(&mut state, " ", selected, &mut response);
                        focus_active = state.open;
                    } else if !was_open {
                        if matches!(
                            key,
                            egui::Key::Enter
                                | egui::Key::Space
                                | egui::Key::ArrowUp
                                | egui::Key::ArrowDown
                        ) {
                            self.open(&mut state, selected, None);
                        }
                    } else {
                        match key {
                            egui::Key::ArrowUp
                            | egui::Key::ArrowDown
                            | egui::Key::Home
                            | egui::Key::End => {
                                self.navigate(&mut state, *key);
                                focus_active = true;
                            }
                            egui::Key::Enter | egui::Key::Space => {
                                if let Some(index) = self.index(state.active) {
                                    self.commit(index, selected, &mut response);
                                    self.close(&mut state, true);
                                }
                            }
                            egui::Key::Escape => self.close(&mut state, true),
                            egui::Key::Tab => {}
                            _ => {}
                        }
                    }

                    if matches!(
                        key,
                        egui::Key::Enter
                            | egui::Key::Space
                            | egui::Key::ArrowUp
                            | egui::Key::ArrowDown
                            | egui::Key::Home
                            | egui::Key::End
                    ) || (was_open && matches!(key, egui::Key::Escape | egui::Key::Tab))
                    {
                        self.response.ctx.input_mut(|input| {
                            input.consume_key(*modifiers, *key);
                        });
                    }
                    refresh |= state.open;
                }

                egui::Event::Text(text)
                    if (state.open || trigger_focused)
                        && text != " "
                        && !text.chars().any(char::is_control)
                        && self.response.ctx.input(|input| unmodified(input.modifiers)) =>
                {
                    state.keyboard = true;
                    self.typeahead(&mut state, text, selected, &mut response);
                    refresh |= state.open;
                    focus_active = state.open;
                }

                egui::Event::AccessKitActionRequest(request)
                    if state.open
                        && request.target_tree == egui::accesskit::TreeId::ROOT
                        && request.action == egui::accesskit::Action::Focus =>
                {
                    // The dispatcher has already applied accepted focus requests.
                    if let Some(index) = self.index(self.focused_id())
                        && !self.items[index].disabled
                        && request.target_node == self.ids[index].accesskit_id()
                    {
                        state.active = Some(self.ids[index]);
                        state.keyboard = true;
                        state.stop_repeat();
                        self.reveal_active(&mut state);
                        refresh = true;
                    }
                }

                egui::Event::AccessKitActionRequest(request)
                    if request.action == egui::accesskit::Action::Click =>
                {
                    if !state.open && routed.target == Some(self.response.id) {
                        self.open(&mut state, selected, None);
                    } else if state.open
                        && let Some(index) =
                            self.items.iter().enumerate().find_map(|(index, item)| {
                                (!item.disabled
                                    && request.target_node == self.ids[index].accesskit_id())
                                .then_some(index)
                            })
                    {
                        self.commit(index, selected, &mut response);
                        self.close(&mut state, true);
                    }
                }

                _ => {}
            },
        }

        if state.open {
            if refresh {
                self.register_popup(&state, selected);
            }
            if focus_active {
                self.focus_active(&state);
            }
        }
        state.observed_selection = self.selected_id(selected);
        self.metadata(selected, &state);
        self.save(state);
        response
    }

    /// Remove this registration during the current input batch.
    ///
    /// Call before removing/replacing a live select, or before abandoning its
    /// registration after a same-batch disable/hide. This relinquishes its
    /// popup without reclaiming focus. Do not call after draining the cursor.
    pub fn unregister(&mut self) {
        self.assert_pass();
        if self.removed {
            return;
        }
        let mut state = self.load();
        self.close(&mut state, false);
        state.reset_transient();
        state.fade = Fade::default();
        self.save(state);
        self.response.ctx.remove_input_widget(self.response.id);
        self.removed = true;
    }

    /// Submit immutable trigger and popup groups after dispatch.
    ///
    /// `painter` must be from the trigger's context/layer and captured before
    /// disabled UI scopes. Its parent opacity is applied once to each portal
    /// submission; the component owns its own disabled-group opacity.
    pub fn paint(
        &self,
        selected: &Option<T>,
        painter: &egui::Painter,
    ) -> Result<Response, compositing::Error> {
        self.assert_pass();
        assert_eq!(&self.response.ctx, painter.ctx(), "Select painter context");
        assert_eq!(
            self.response.layer_id,
            painter.layer_id(),
            "Select painter layer"
        );
        let response = self.current_response();
        if !self.removed {
            let state = self.load();
            self.metadata(selected, &state);
            paint::submit(self, selected, &response, &state, painter)?;
        }
        Ok(response)
    }
}
