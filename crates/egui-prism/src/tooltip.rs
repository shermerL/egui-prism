//! Plain-text, noninteractive hints for explicitly identified ordinary triggers.
//!
//! Register after the trigger, offer every ordered event to `handle_all`, then
//! paint after draining the host's cursor. The slice is the explicit provider
//! scope: it coordinates peer dismissal, hover transit and the 300ms skip delay.
//! No popup widget, focus scope, input owner, timer or global registry is created.

use std::{sync::Arc, time::Duration};

use crate::{
    compositing::{self, Group, Item},
    egui::{self, Id, Pos2, Rect, Response},
    theme::Theme,
};

/// A hint borrowing its trigger, plain text and theme.
///
/// The response supplies the caller's stable trigger ID. Enablement is explicit
/// and is additionally constrained by the trigger's enablement and visibility.
#[must_use = "register the tooltip before ordered dispatch"]
pub struct Tooltip<'a, 'response> {
    theme: &'a Theme,
    response: &'response Response,
    text: &'a str,
    enabled: bool,
    delay: Duration,
}

impl<'a, 'response> Tooltip<'a, 'response> {
    /// Attach a plain-text hint to an already registered, explicitly named trigger.
    pub fn new(theme: &'a Theme, response: &'response Response, text: &'a str) -> Self {
        Self {
            theme,
            response,
            text,
            enabled: true,
            delay: Duration::ZERO,
        }
    }

    /// Enable or disable this hint in addition to its trigger's own enablement.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set the pointer opening delay. The pinned shadcn provider defaults to zero.
    /// Keyboard focus opens immediately. A provider scope skips this delay while
    /// another hint is open and for 300ms after a hint closes.
    pub fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    /// Register without allocating, drawing, or changing the trigger's input policy.
    ///
    /// `ui` must be in the trigger's context and layer, with its enclosing clip.
    /// The registration clones the response snapshot; it retains borrows only of
    /// text and theme, not of the response or UI. Call between `begin_input` and
    /// dispatch, or inside the host's `input_layout` refresh.
    pub fn register(self, ui: &egui::Ui) -> TooltipRegistration<'a> {
        let ctx = &self.response.ctx;
        assert!(
            ctx.options(|options| options.ordered_input),
            "Tooltip requires ordered input"
        );
        assert_eq!(ctx, ui.ctx(), "Tooltip registration context");
        assert_eq!(
            self.response.layer_id,
            ui.layer_id(),
            "Tooltip registration layer"
        );
        let screen = ctx.content_rect();
        assert!(
            screen.is_finite() && screen.is_positive(),
            "Tooltip requires a finite viewport"
        );
        let global = |rect| {
            ctx.layer_transform_to_global(self.response.layer_id)
                .map_or(rect, |transform| transform * rect)
        };
        let anchor = global(self.response.rect);
        let visible_anchor =
            global(self.response.interact_rect.intersect(ui.clip_rect())).intersect(screen);
        let mut description = None;
        ctx.accesskit_node_builder(self.response.id, |node| {
            description = node.description().map(str::to_owned);
        });
        let registration = TooltipRegistration {
            response: self.response.clone(),
            theme: self.theme,
            text: self.text,
            enabled: self.enabled,
            delay: self.delay,
            key: self
                .response
                .id
                .with(("egui-prism/tooltip", ctx.viewport_id())),
            viewport: ctx.viewport_id(),
            pass: ctx.cumulative_pass_nr(),
            frame: ctx.cumulative_frame_nr(),
            screen,
            visible_anchor,
            geometry: Geometry::new(ui, self.text, anchor, screen),
            description,
            removed: false,
        };
        let mut state = registration.load();
        let discontinuity = state.last_frame.is_some_and(|last| {
            registration.frame > last.saturating_add(1)
                || state.anchor != anchor
                || state.screen != screen
        });
        if discontinuity || !registration.usable() || !ctx.input(|input| input.focused) {
            registration.cancel(&mut state);
        }
        state.last_frame = Some(registration.frame);
        state.anchor = anchor;
        state.screen = screen;
        registration.metadata(&state);
        registration.schedule(&state);
        registration.save(state);
        // This requests an opportunity, not a frame or timer. In particular it
        // observes host focus changes even in an otherwise event-free frame.
        ctx.request_input_time();
        registration
    }
}

/// Pass-local tooltip authority. It does not own or register its trigger.
///
/// Use one `handle_all` slice for the viewport's live hints. Resources and the
/// registration must survive through final paint, but not into another pass.
pub struct TooltipRegistration<'a> {
    /// The original trigger response snapshot, with the caller's stable ID.
    /// Use `Context::read_response` for current event activation results.
    pub response: Response,
    theme: &'a Theme,
    text: &'a str,
    enabled: bool,
    delay: Duration,
    key: Id,
    viewport: egui::ViewportId,
    pass: u64,
    frame: u64,
    screen: Rect,
    visible_anchor: Rect,
    geometry: Geometry,
    description: Option<String>,
    removed: bool,
}

/// A snapshot, not an activation or a focus request.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TooltipOutput {
    /// Whether the hint is logically open. Closing animation is not open state.
    pub open: bool,
    /// Unanimated content body bounds in global points, including during exit.
    /// The decorative arrow and clipped animation may extend beyond this body.
    pub content_rect: Option<Rect>,
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
        egui::lerp(self.from..=self.to, ease(x))
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

// CSS animation's default `ease`, not Tailwind transition timing.
fn ease(x: f32) -> f32 {
    if x == 0.0 || x == 1.0 {
        return x;
    }
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

#[derive(Clone)]
struct State {
    last_frame: Option<u64>,
    anchor: Rect,
    screen: Rect,
    open: bool,
    pending: Option<f64>,
    skip_until: f64,
    focused: bool,
    pressed: bool,
    entered: bool,
    over: u8,
    grace: Vec<Pos2>,
    fade: Fade,
}

impl Default for State {
    fn default() -> Self {
        Self {
            last_frame: None,
            anchor: Rect::NOTHING,
            screen: Rect::NOTHING,
            open: false,
            pending: None,
            skip_until: f64::NEG_INFINITY,
            focused: false,
            pressed: false,
            entered: false,
            over: 0,
            grace: Vec::new(),
            fade: Fade::default(),
        }
    }
}

struct Geometry {
    body: Rect,
    galley: Arc<egui::Galley>,
    above: bool,
    arrow_x: f32,
}

impl Geometry {
    fn new(ui: &egui::Ui, text: &str, anchor: Rect, screen: Rect) -> Self {
        let above = (anchor.top() - screen.top() - 10.0).max(0.0);
        let below = (screen.bottom() - anchor.bottom() - 10.0).max(0.0);
        let mut font = egui::TextStyle::Body.resolve(ui.style());
        font.size = 12.0;
        // The pinned Popper wrapper has min-width: max-content. Preserve its
        // natural text width; only the final drawing and allocation are clipped.
        // CSS normal white-space collapses ASCII whitespace, but not NBSP.
        let visible_text = text
            .split([' ', '\t', '\n', '\r', '\u{000c}'])
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        let mut job = egui::text::LayoutJob::simple(
            visible_text,
            font,
            egui::Color32::PLACEHOLDER,
            f32::INFINITY,
        );
        for section in &mut job.sections {
            section.format.line_height = Some(16.0);
        }
        let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
        let size = galley.size() + egui::vec2(24.0, 12.0);
        let use_above = size.y <= above || above >= below;
        let left = (anchor.center().x - size.x * 0.5)
            .clamp(screen.left(), (screen.right() - size.x).max(screen.left()));
        let top = if use_above {
            anchor.top() - 10.0 - size.y
        } else {
            anchor.bottom() + 10.0
        }
        .clamp(screen.top(), (screen.bottom() - size.y).max(screen.top()));
        let body = Rect::from_min_size(egui::pos2(left, top), size);
        let inset = 10.0_f32.min(body.width() * 0.5);
        let arrow_x = anchor
            .center()
            .x
            .clamp(body.left() + inset, body.right() - inset);
        Self {
            body,
            galley,
            above: use_above,
            arrow_x,
        }
    }

    fn transform(&self, state: &State, now: f64) -> egui::emath::TSTransform {
        let progress = state.fade.value(now);
        let scaling = 0.95 + 0.05 * progress;
        let origin = egui::pos2(
            self.arrow_x,
            if self.above {
                self.body.bottom() + 10.0
            } else {
                self.body.top() - 10.0
            },
        );
        // The reference has slide-in classes, but no slide-out classes.
        let slide = if state.fade.to == 1.0 {
            (if self.above { 8.0 } else { -8.0 }) * (1.0 - progress)
        } else {
            0.0
        };
        egui::emath::TSTransform {
            scaling,
            translation: origin.to_vec2() * (1.0 - scaling) + egui::vec2(0.0, slide),
        }
    }
}

impl TooltipRegistration<'_> {
    fn assert_pass(&self) {
        assert_eq!(
            self.viewport,
            self.response.ctx.viewport_id(),
            "Tooltip viewport"
        );
        assert_eq!(
            self.pass,
            self.response.ctx.cumulative_pass_nr(),
            "Tooltip registrations are pass-local"
        );
    }

    fn now(&self) -> f64 {
        self.response.ctx.input(|input| input.time)
    }

    fn load(&self) -> State {
        self.response
            .ctx
            .data(|data| data.get_temp::<State>(self.key))
            .unwrap_or_default()
    }

    fn save(&self, state: State) {
        self.response
            .ctx
            .data_mut(|data| data.insert_temp(self.key, state));
    }

    fn current_response(&self) -> Response {
        self.response
            .ctx
            .read_response(self.response.id)
            .unwrap_or_else(|| self.response.clone())
    }

    fn usable(&self) -> bool {
        let response = self.current_response();
        !self.removed
            && self.enabled
            && !self.text.is_empty()
            && response.enabled()
            && response.visible()
            && self.visible_anchor.is_finite()
            && self.visible_anchor.is_positive()
            && self
                .response
                .ctx
                .memory(|memory| memory.allows_interaction(response.layer_id))
    }

    fn hit(&self, pos: Pos2) -> bool {
        self.visible_anchor.contains(pos) && self.current_response().contains_pointer()
    }

    fn close(&self, state: &mut State) {
        state.pending = None;
        state.grace.clear();
        if state.open {
            state.open = false;
            state.skip_until = self.now() + 0.3;
            state.fade.retarget(self.now(), 0.0);
            self.response.ctx.request_repaint();
        }
    }

    fn open(&self, state: &mut State) {
        state.pending = None;
        if !state.open {
            state.open = true;
            state.fade.retarget(self.now(), 1.0);
            self.response.ctx.request_repaint();
        }
    }

    fn cancel(&self, state: &mut State) {
        self.close(state);
        state.fade = Fade::default();
        state.skip_until = f64::NEG_INFINITY;
        state.pressed = false;
        state.entered = false;
        state.over = 0;
        // Re-enabling or reappearing must not synthesize an old focus edge.
        state.focused = self.current_response().has_focus();
    }

    fn schedule(&self, state: &State) {
        if let Some(at) = state.pending {
            self.response
                .ctx
                .request_repaint_after(Duration::from_secs_f64((at - self.now()).max(0.0)));
        }
    }

    fn metadata(&self, state: &State) {
        let described = state.open && self.usable();
        self.response
            .ctx
            .accesskit_node_builder(self.response.id, |node| {
                if described {
                    if let Some(base) = &self.description {
                        node.set_description(format!("{base}\n{}", self.text));
                    } else {
                        node.set_description(self.text);
                    }
                } else if let Some(base) = &self.description {
                    node.set_description(base.clone());
                } else {
                    node.clear_description();
                }
            });
    }

    /// Observe one event for the complete, explicitly supplied provider scope.
    ///
    /// Offer every event, including untargeted Time, releases, blur and events for
    /// other widgets. Call before ordinary trigger activation and host scrolling.
    /// Nothing is consumed: dispatch the same event to its ordinary target too.
    /// The slice must contain distinct trigger IDs in one context and viewport.
    pub fn handle_all(registrations: &mut [Self], routed: &egui::RoutedInput) {
        let Some(first) = registrations.first() else {
            return;
        };
        let now = first.now();
        for (i, registration) in registrations.iter().enumerate() {
            registration.assert_pass();
            assert_eq!(
                &registration.response.ctx, &first.response.ctx,
                "Tooltip provider context"
            );
            assert_eq!(
                registration.viewport, first.viewport,
                "Tooltip provider viewport"
            );
            assert!(
                registrations[..i]
                    .iter()
                    .all(|other| other.response.id != registration.response.id),
                "duplicate Tooltip trigger"
            );
        }
        let mut states: Vec<_> = registrations.iter().map(Self::load).collect();
        let instant = states
            .iter()
            .any(|state| state.open || now < state.skip_until);
        let mut opening = vec![false; states.len()];
        let mut hovering = vec![false; states.len()];
        // egui's synthetic pointer events do not carry a pointer type. A raw
        // touch-bearing batch must not open a mouse-hover tooltip.
        let touch = first.response.ctx.input(|input| {
            input
                .raw
                .events
                .iter()
                .any(|event| matches!(event, egui::Event::Touch { .. }))
        });

        for (index, (registration, state)) in registrations.iter().zip(&mut states).enumerate() {
            if !registration.usable() || !registration.response.ctx.input(|input| input.focused) {
                registration.cancel(state);
                continue;
            }
            let response = registration.current_response();
            let event = match &routed.trigger {
                egui::InputTrigger::Event { event, .. } => Some(event),
                egui::InputTrigger::Time => None,
            };
            let own_press = matches!(event, Some(egui::Event::PointerButton { pos, pressed: true, .. }) if registration.hit(*pos));
            if own_press {
                state.pressed = true;
            }
            let focused = response.has_focus();
            if focused && !state.focused && !state.pressed {
                opening[index] = true;
            } else if !focused && state.focused {
                registration.close(state);
            }
            state.focused = focused;

            match event {
                Some(egui::Event::PointerMoved(pos)) if !touch => {
                    let trigger = registration.hit(*pos);
                    let content_rect =
                        registration.geometry.transform(state, now) * registration.geometry.body;
                    let content = state.open && content_rect.contains(*pos);
                    if !trigger {
                        state.entered = false;
                        state.pending = None;
                    }
                    if state.open
                        && ((state.over == 1 && !trigger) || (state.over == 2 && !content))
                    {
                        let (from, to) = if state.over == 1 {
                            (registration.visible_anchor, content_rect)
                        } else {
                            (content_rect, registration.visible_anchor)
                        };
                        state.grace = corridor(*pos, from, to);
                    }
                    if trigger || content {
                        state.grace.clear();
                    } else if !state.grace.is_empty() && !inside(*pos, &state.grace) {
                        registration.close(state);
                    }
                    hovering[index] = trigger && !state.entered;
                    state.over = if trigger {
                        1
                    } else if content {
                        2
                    } else {
                        0
                    };
                }
                Some(egui::Event::PointerButton { pos, pressed, .. }) => {
                    if *pressed {
                        let content = state.open
                            && (registration.geometry.transform(state, now)
                                * registration.geometry.body)
                                .contains(*pos);
                        // Radix trigger pointerdown closes an open hint; outside
                        // pointerdown dismisses without blocking the click.
                        if own_press {
                            if state.open {
                                registration.close(state);
                            }
                        } else if !content {
                            registration.close(state);
                        }
                        opening[index] = false;
                    } else {
                        state.pressed = false;
                    }
                }
                Some(egui::Event::PointerGone) => {
                    registration.close(state);
                    state.entered = false;
                    state.pressed = false;
                    state.over = 0;
                    opening[index] = false;
                }
                Some(egui::Event::WindowFocused(false)) => {
                    registration.cancel(state);
                    opening[index] = false;
                }
                Some(egui::Event::Key {
                    key: egui::Key::Escape,
                    pressed: true,
                    ..
                }) => {
                    registration.close(state);
                    opening[index] = false;
                }
                // Ordinary button click semantics. Custom trigger activation
                // policies can additionally call dismiss from their own handler.
                Some(egui::Event::Key { key, pressed, .. })
                    if focused
                        && ((*key == egui::Key::Enter && *pressed)
                            || (*key == egui::Key::Space && !pressed)) =>
                {
                    registration.close(state);
                    opening[index] = false;
                }
                Some(egui::Event::AccessKitActionRequest(request))
                    if routed.target == Some(response.id)
                        && request.action == egui::accesskit::Action::Click =>
                {
                    registration.close(state);
                    opening[index] = false;
                }
                None => {
                    if let Some(at) = state.pending {
                        let hit = registration
                            .response
                            .ctx
                            .input(|input| input.pointer.hover_pos())
                            .is_some_and(|pos| registration.hit(pos));
                        if !hit {
                            state.pending = None;
                        } else if now >= at {
                            opening[index] = true;
                        }
                    }
                }
                _ => {}
            }
            if routed.target == Some(response.id) && response.clicked() {
                registration.close(state);
                opening[index] = false;
            }
        }

        let in_transit = states.iter().any(|state| !state.grace.is_empty());
        for (index, (registration, state)) in registrations.iter().zip(&mut states).enumerate() {
            if hovering[index] && !in_transit {
                state.entered = true;
                if instant || registration.delay.is_zero() {
                    opening[index] = true;
                } else {
                    state.pending = Some(now + registration.delay.as_secs_f64());
                }
            }
        }
        for index in 0..states.len() {
            if opening[index] && registrations[index].usable() && !states[index].open {
                for other in 0..states.len() {
                    if other != index {
                        registrations[other].close(&mut states[other]);
                    }
                }
                registrations[index].open(&mut states[index]);
            }
        }
        let skip_until = states
            .iter()
            .map(|state| state.skip_until)
            .fold(f64::NEG_INFINITY, f64::max);
        for (registration, mut state) in registrations.iter().zip(states) {
            if !registration.removed {
                state.skip_until = skip_until;
            }
            registration.metadata(&state);
            registration.schedule(&state);
            registration.save(state);
        }
    }

    /// Dismiss without touching trigger input, focus, or activation.
    ///
    /// Use for custom activation and actual ancestor scrolling. A wheel event
    /// alone is not a DOM scroll event and is deliberately not consumed or treated
    /// as proof that the trigger's ancestor scrolled.
    pub fn dismiss(&mut self) {
        self.assert_pass();
        let mut state = self.load();
        self.close(&mut state);
        self.metadata(&state);
        self.save(state);
    }

    /// Cancel before same-batch removal, replacement, hide or disable.
    ///
    /// This removes neither the borrowed trigger nor its focus policy. The host
    /// retires its own widget geometry. Re-register replacements before advancing
    /// the cursor. A missing whole frame also cancels stale state on reappearance.
    pub fn unregister(&mut self) {
        self.assert_pass();
        let mut state = self.load();
        self.cancel(&mut state);
        self.removed = true;
        self.metadata(&state);
        self.save(state);
    }

    /// Inspect current logical state and unanimated global body bounds.
    pub fn output(&self) -> TooltipOutput {
        self.assert_pass();
        let state = self.load();
        let live = self.usable();
        TooltipOutput {
            open: live && state.open,
            content_rect: (live && (state.open || state.fade.value(self.now()) > 0.0))
                .then_some(self.geometry.body),
        }
    }

    /// Paint only the hint after dispatch, using a clean trigger-layer painter.
    ///
    /// The portal escapes the trigger clip and transform. Parent opacity is
    /// applied once to the composited body, arrow and text, never per primitive.
    /// No input geometry is created. The host still paints the ordinary trigger.
    pub fn paint(&self, painter: &egui::Painter) -> Result<TooltipOutput, compositing::Error> {
        self.assert_pass();
        assert_eq!(&self.response.ctx, painter.ctx(), "Tooltip painter context");
        assert_eq!(
            self.response.layer_id,
            painter.layer_id(),
            "Tooltip painter layer"
        );
        let state = self.load();
        self.metadata(&state);
        let output = self.output();
        let now = self.now();
        let opacity = state.fade.value(now);
        if output.content_rect.is_none() {
            return Ok(output);
        }
        if state.fade.moving(now) {
            self.response.ctx.request_repaint();
        }
        if opacity <= 0.0 {
            return Ok(output);
        }

        let body = self.geometry.body;
        let colors = self.theme.colors.background;
        let arrow_center = egui::pos2(
            self.geometry.arrow_x,
            if self.geometry.above {
                body.bottom() + 2.0
            } else {
                body.top() - 2.0
            },
        );
        let arrow = rounded_diamond(arrow_center, colors.foreground);
        let bounds = body.union(arrow.visual_bounding_rect()).expand(1.0);
        let transform = self.geometry.transform(&state, now);
        let mut items = Vec::new();
        let mut add = |clip: Rect, shape: egui::Shape| {
            let clip = (transform * clip).intersect(self.screen);
            if clip.is_positive() {
                let mut clipped = egui::epaint::ClippedShape::new(clip, shape);
                clipped.transform_after_tessellation = transform;
                items.push(Item::Shape(clipped));
            }
        };
        add(bounds, arrow);
        add(bounds, egui::Shape::rect_filled(body, 8, colors.foreground));
        let text_clip = body.shrink2(egui::vec2(12.0, 6.0));
        add(
            text_clip,
            egui::Shape::galley(
                body.min + egui::vec2(12.0, 6.0) - self.geometry.galley.rect.min.to_vec2(),
                self.geometry.galley.clone(),
                colors.background,
            ),
        );
        let visible_bounds = (transform * bounds).intersect(self.screen);
        if !visible_bounds.is_positive() {
            return Ok(output);
        }
        let group = Group::new(visible_bounds, opacity, items)?;
        let mut portal = painter.clone();
        portal.set_layer_id(egui::LayerId::new(
            egui::Order::Tooltip,
            self.key.with("drawing"),
        ));
        portal.set_clip_rect(self.screen);
        compositing::paint(&portal, group)?;
        Ok(output)
    }
}

fn rounded_diamond(center: Pos2, color: egui::Color32) -> egui::Shape {
    let mut points = Vec::with_capacity(28);
    for (corner, start) in [
        (egui::vec2(3.0, -3.0), -std::f32::consts::FRAC_PI_2),
        (egui::vec2(3.0, 3.0), 0.0),
        (egui::vec2(-3.0, 3.0), std::f32::consts::FRAC_PI_2),
        (egui::vec2(-3.0, -3.0), std::f32::consts::PI),
    ] {
        for step in 0..=6 {
            let angle = start + step as f32 * std::f32::consts::FRAC_PI_2 / 6.0;
            let p = corner + egui::vec2(angle.cos(), angle.sin()) * 2.0;
            let k = std::f32::consts::FRAC_1_SQRT_2;
            points.push(center + egui::vec2(p.x - p.y, p.x + p.y) * k);
        }
    }
    egui::Shape::convex_polygon(points, color, egui::Stroke::NONE)
}

// Radix's nearest exit side, two inward-padded exit points, and convex hull of
// those points with the opposite rectangle. This is not a timeout or a union
// of expanded rectangles: leaving the corridor closes on that pointer move.
fn corridor(point: Pos2, from: Rect, to: Rect) -> Vec<Pos2> {
    let distances = [
        (from.left() - point.x).abs(),
        (from.right() - point.x).abs(),
        (from.top() - point.y).abs(),
        (from.bottom() - point.y).abs(),
    ];
    let side = (0..4)
        .min_by(|a, b| distances[*a].total_cmp(&distances[*b]))
        .unwrap();
    let offsets = match side {
        0 => [egui::vec2(5.0, -5.0), egui::vec2(5.0, 5.0)],
        1 => [egui::vec2(-5.0, -5.0), egui::vec2(-5.0, 5.0)],
        2 => [egui::vec2(-5.0, 5.0), egui::vec2(5.0, 5.0)],
        _ => [egui::vec2(-5.0, -5.0), egui::vec2(5.0, -5.0)],
    };
    let mut points = vec![
        point + offsets[0],
        point + offsets[1],
        to.left_top(),
        to.right_top(),
        to.right_bottom(),
        to.left_bottom(),
    ];
    points.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
    points.dedup();
    let mut hull = Vec::new();
    for half in [points.clone(), points.into_iter().rev().collect()] {
        let mut edge: Vec<Pos2> = Vec::new();
        for p in half {
            while edge.len() >= 2 {
                let r = edge[edge.len() - 2];
                let q = edge[edge.len() - 1];
                if (q.x - r.x) * (p.y - r.y) >= (q.y - r.y) * (p.x - r.x) {
                    edge.pop();
                } else {
                    break;
                }
            }
            edge.push(p);
        }
        edge.pop();
        hull.extend(edge);
    }
    hull
}

fn inside(point: Pos2, polygon: &[Pos2]) -> bool {
    let mut result = false;
    let mut previous = polygon[polygon.len() - 1];
    for &current in polygon {
        if (current.y > point.y) != (previous.y > point.y)
            && point.x
                < (previous.x - current.x) * (point.y - current.y) / (previous.y - current.y)
                    + current.x
        {
            result = !result;
        }
        previous = current;
    }
    result
}
