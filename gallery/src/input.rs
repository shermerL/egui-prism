//! Pass-local routing for the Gallery's explicit controls and scroll shells.
use crate::{
    Case, Gallery, buttons, checkboxes, compositing, dialogs, egui, interaction, selects, sliders,
    spinners, switches, text_inputs, tooltips,
};
use egui_prism::checkbox::{Checkbox, CheckboxRegistration};
use egui_prism::switch::{Switch, SwitchRegistration};
use std::{collections::HashSet, sync::Arc};

#[derive(Clone, Debug)]
pub(super) enum Control {
    Page(Case),
    Dark(bool),
    Narrow,
    Zoom(f32),
    Selected,
    Button(buttons::Control),
    Input(text_inputs::Control),
    Checkbox(checkboxes::Control),
    Slider(sliders::Control),
    Switch(switches::Control),
    Spinner(spinners::Control),
    Select(selects::Control),
    Tooltip(tooltips::Control),
    Dialog(dialogs::Control),
    Interaction(interaction::Control),
    Compositing(compositing::Control),
    Link(&'static str, &'static str),
}

impl Control {
    pub fn is_status(&self) -> bool {
        matches!(
            self,
            Self::Button(buttons::Control::PreviewCount | buttons::Control::Count(_))
                | Self::Input(text_inputs::Control::Count(_))
                | Self::Checkbox(checkboxes::Control::Count(_))
                | Self::Slider(sliders::Control::Value(_) | sliders::Control::Focus(_))
                | Self::Switch(switches::Control::Count(_))
                | Self::Select(selects::Control::Count(_) | selects::Control::OutsideCount)
                | Self::Tooltip(tooltips::Control::Count(_) | tooltips::Control::OutsideCount)
                | Self::Dialog(dialogs::Control::Count(_) | dialogs::Control::OutsideCount)
                | Self::Interaction(
                    interaction::Control::Count(_) | interaction::Control::Focus(_)
                )
        )
    }

    pub fn changes_layout(&self, response: &egui::Response) -> bool {
        match self {
            Self::Button(
                buttons::Control::Disabled | buttons::Control::Invalid | buttons::Control::Reversed,
            )
            | Self::Input(
                text_inputs::Control::Disabled
                | text_inputs::Control::Error
                | text_inputs::Control::Reversed
                | text_inputs::Control::Editor(_),
            )
            | Self::Checkbox(
                checkboxes::Control::Disabled
                | checkboxes::Control::Reversed
                | checkboxes::Control::Long
                | checkboxes::Control::Invalid
                | checkboxes::Control::Compositing,
            )
            | Self::Slider(
                sliders::Control::Disabled
                | sliders::Control::Reversed
                | sliders::Control::Long
                | sliders::Control::Tinted
                | sliders::Control::Width(_),
            )
            | Self::Switch(
                switches::Control::Disabled
                | switches::Control::Reversed
                | switches::Control::Long
                | switches::Control::Small
                | switches::Control::Invalid
                | switches::Control::Compositing,
            )
            | Self::Select(
                selects::Control::Small
                | selects::Control::Disabled
                | selects::Control::Invalid
                | selects::Control::Long
                | selects::Control::Reversed
                | selects::Control::Empty
                | selects::Control::AllDisabled
                | selects::Control::Tinted,
            )
            | Self::Tooltip(
                tooltips::Control::Disabled
                | tooltips::Control::Hidden
                | tooltips::Control::Long
                | tooltips::Control::Reverse
                | tooltips::Control::Delayed,
            )
            | Self::Spinner(spinners::Control::Hidden | spinners::Control::Reduced)
            | Self::Dialog(
                dialogs::Control::Long
                | dialogs::Control::Reverse
                | dialogs::Control::KeepOpen
                | dialogs::Control::Tinted,
            )
            | Self::Interaction(
                interaction::Control::Disabled
                | interaction::Control::Reversed
                | interaction::Control::Text(_),
            ) => response.changed(),
            Self::Input(text_inputs::Control::Inspector)
            | Self::Select(selects::Control::Reset) => response.clicked(),
            _ => false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Phase {
    Register,
    Paint,
}

pub(super) struct Draw {
    pub response: egui::Response,
    pub ids: Vec<egui::Id>,
    pub button: Option<egui_prism::button::ButtonOutput>,
}
impl From<egui::Response> for Draw {
    fn from(response: egui::Response) -> Self {
        Self {
            response,
            ids: Vec::new(),
            button: None,
        }
    }
}
impl From<sliders::Draw> for Draw {
    fn from(draw: sliders::Draw) -> Self {
        Self {
            response: draw.response,
            ids: draw.ids,
            button: None,
        }
    }
}
impl From<egui_prism::button::ButtonOutput> for Draw {
    fn from(button: egui_prism::button::ButtonOutput) -> Self {
        Self {
            response: button.response.clone(),
            ids: Vec::new(),
            button: Some(button),
        }
    }
}

/// Everything needed to reopen one local UI, not a retained widget tree.
///
/// Registration, targeted handling and painting all open the same explicitly
/// named UI. In particular, none reconstruct a widget's automatic response ID.
#[derive(Clone)]
pub(super) struct LocalUi {
    id: egui::Id,
    rect: egui::Rect,
    clip: egui::Rect,
    layer: egui::LayerId,
    layout: egui::Layout,
    style: Arc<egui::Style>,
    enabled: bool,
    visible: bool,
    opacity: f32,
    in_scroll: bool,
}
impl LocalUi {
    pub fn capture(ui: &egui::Ui) -> Self {
        Self {
            id: ui.scope_id(),
            rect: ui.available_rect_before_wrap(),
            clip: ui.clip_rect(),
            layer: ui.layer_id(),
            layout: *ui.layout(),
            style: ui.style().clone(),
            enabled: ui.is_enabled(),
            visible: ui.is_visible(),
            opacity: ui.opacity(),
            in_scroll: ui
                .stack()
                .iter()
                .any(|stack| stack.kind() == Some(egui::UiKind::ScrollArea)),
        }
    }

    pub fn named(ui: &egui::Ui, id: egui::Id) -> Self {
        Self {
            id,
            ..Self::capture(ui)
        }
    }

    pub fn open(&self, context: &egui::Context) -> egui::Ui {
        let mut ui = egui::Ui::new(
            context.clone(),
            self.id,
            egui::UiBuilder::new()
                .max_rect(self.rect)
                .layer_id(self.layer)
                .layout(self.layout)
                .style(self.style.clone())
                .ui_stack_info(if self.in_scroll {
                    egui::UiStackInfo::new(egui::UiKind::ScrollArea)
                } else {
                    egui::UiStackInfo::default()
                }),
        );
        ui.set_clip_rect(self.clip);
        if !self.enabled {
            ui.disable();
        }
        if !self.visible {
            ui.set_invisible();
        }
        ui.set_opacity(self.opacity);
        ui
    }
}

pub(super) struct Entry {
    pub control: Control,
    pub response: egui::Response,
    pub button: Option<egui_prism::button::ButtonOutput>,
    pub switch: Option<SwitchRegistration>,
    pub checkbox: Option<CheckboxRegistration>,
    local: Option<LocalUi>,
    ids: HashSet<egui::Id>,
}

impl Entry {
    pub fn open(&self, context: &egui::Context) -> egui::Ui {
        self.local.as_ref().expect("local control UI").open(context)
    }
}

struct Scroll {
    id: egui::Id,
    local: LocalUi,
    horizontal: bool,
    size: egui::Vec2,
    offset: egui::Vec2,
}

pub(super) struct Layout {
    pub phase: Phase,
    pub entries: Vec<Entry>,
    pub ids: HashSet<egui::Id>,
    scrolls: Vec<Scroll>,
    pub slider_live: sliders::Live,
}
impl Layout {
    pub fn new(phase: Phase) -> Self {
        Self {
            phase,
            entries: Vec::new(),
            ids: HashSet::new(),
            scrolls: Vec::new(),
            slider_live: sliders::Live::default(),
        }
    }

    pub fn track(&mut self, response: egui::Response) -> egui::Response {
        self.ids.insert(response.id);
        response
    }

    pub fn label(
        &mut self,
        ui: &mut egui::Ui,
        text: impl Into<egui::WidgetText>,
    ) -> egui::Response {
        self.track(ui.label(text))
    }

    pub fn bind_page(&mut self, page: Case, response: egui::Response) {
        let id = response.id;
        self.ids.insert(id);
        self.entries.push(Entry {
            control: Control::Page(page),
            response,
            button: None,
            switch: None,
            checkbox: None,
            local: None,
            ids: HashSet::from([id]),
        });
    }

    pub fn widget(
        &mut self,
        ui: &mut egui::Ui,
        control: Control,
        draw: impl FnOnce(&mut egui::Ui) -> Draw,
    ) -> egui::Response {
        // The semantic key, rather than the current position in the layout,
        // names the control. Instance scopes still distinguish repeated cases.
        let id = ui.make_persistent_id(("gallery-control", format!("{control:?}")));
        let slot = ui.scope_builder(egui::UiBuilder::new().scope_id(id.with("slot")), |slot| {
            let local = LocalUi::named(slot, id);
            let mut child = local.open(slot.ctx());
            let painter = (self.phase == Phase::Paint).then(|| child.painter().clone());
            let mut output = draw(&mut child);
            if let Some(painter) = painter
                && let Some(button) = output.button.take()
            {
                button.paint(&painter).expect("button compositor");
            }
            slot.advance_cursor_after_rect(child.min_rect());
            (local, output)
        });
        self.ids.insert(slot.response.id);
        let (local, mut output) = slot.inner;
        let mut ids: HashSet<_> = output.ids.drain(..).collect();
        ids.insert(output.response.id);
        ids.insert(id);
        self.ids.extend(ids.iter().copied());

        let response = output.response.clone();
        self.entries.push(Entry {
            control,
            response: output.response,
            button: output.button,
            switch: None,
            checkbox: None,
            local: Some(local),
            ids,
        });
        response
    }

    pub fn switch_widget<'a>(
        &mut self,
        ui: &mut egui::Ui,
        control: switches::Control,
        build: impl FnOnce(&mut egui::Ui) -> Switch<'a, 'a>,
    ) {
        let phase = self.phase;
        let mut registration = None;
        self.widget(ui, Control::Switch(control), |ui| {
            let painter = (phase == Phase::Paint).then(|| ui.painter().clone());
            let switch = build(ui);
            if let Some(painter) = painter {
                switch
                    .show(ui)
                    .paint(&painter)
                    .expect("switch compositor")
                    .into()
            } else {
                let registered = switch.register(ui);
                let response = registered.response.clone();
                registration = Some(registered);
                response.into()
            }
        });
        self.entries.last_mut().expect("switch entry").switch = registration;
    }

    pub fn checkbox_widget<'a>(
        &mut self,
        ui: &mut egui::Ui,
        control: checkboxes::Control,
        build: impl FnOnce(&mut egui::Ui) -> Checkbox<'a, 'a>,
    ) {
        let phase = self.phase;
        let mut registration = None;
        self.widget(ui, Control::Checkbox(control), |ui| {
            let painter = (phase == Phase::Paint).then(|| ui.painter().clone());
            let checkbox = build(ui);
            if let Some(painter) = painter {
                checkbox
                    .show(ui)
                    .paint(&painter)
                    .expect("checkbox compositor")
                    .into()
            } else {
                let registered = checkbox.register(ui);
                let response = registered.response.clone();
                registration = Some(registered);
                response.into()
            }
        });
        self.entries.last_mut().expect("checkbox entry").checkbox = registration;
    }

    pub fn scroll(
        &mut self,
        ui: &mut egui::Ui,
        horizontal: bool,
        contents: impl FnOnce(&mut egui::Ui, &mut Self),
    ) {
        let id = ui.make_persistent_id(("gallery-scroll", horizontal));
        let slot = ui.scope_builder(egui::UiBuilder::new().scope_id(id.with("slot")), |slot| {
            let local = LocalUi::named(slot, id);
            let mut child = local.open(slot.ctx());
            let output = scroll_area(horizontal).show(&mut child, |ui| contents(ui, self));
            slot.advance_cursor_after_rect(child.min_rect());
            self.ids.insert(output.id);
            // Inner shells are appended while the content closure runs.
            self.scrolls.push(Scroll {
                id: output.id,
                local,
                horizontal,
                size: output.content_size,
                offset: output.state.unclamped_offset(),
            });
        });
        self.ids.insert(slot.response.id);
        self.ids.insert(id);
    }

    /// Offer only shells an input opportunity, in inner-first consumption order.
    pub fn scroll_event(&mut self, context: &egui::Context) -> bool {
        let mut moved = false;
        for scroll in &mut self.scrolls {
            let mut ui = scroll.local.open(context);
            let output = scroll_area(scroll.horizontal).show(&mut ui, |ui| {
                ui.set_min_size(scroll.size);
            });
            let next = output.state.unclamped_offset();
            moved |= next != scroll.offset;
            scroll.offset = next;
        }
        moved
    }

    pub fn cancel_interactions(&self) {
        for entry in &self.entries {
            if let Some(button) = &entry.button {
                button.cancel_unfocused();
            }
            if let Some(checkbox) = &entry.checkbox {
                checkbox.cancel_unfocused();
            }
            if let Some(switch) = &entry.switch {
                switch.cancel_unfocused();
            }
        }
    }

    pub fn refresh_statuses(
        &mut self,
        context: &egui::Context,
        mut draw: impl FnMut(&mut egui::Ui, &Control) -> Draw,
    ) {
        for entry in &mut self.entries {
            if !entry.control.is_status() {
                continue;
            }
            let local = entry.local.as_ref().expect("status label UI");
            let output = draw(&mut local.open(context), &entry.control);
            let mut ids: HashSet<_> = output.ids.into_iter().collect();
            ids.insert(output.response.id);
            ids.insert(local.id);
            for id in entry.ids.difference(&ids) {
                context.remove_input_widget(*id);
                self.ids.remove(id);
            }
            self.ids.extend(ids.iter().copied());
            entry.ids = ids;
            entry.response = output.response;
            entry.button = output.button;
        }
    }

    pub fn retire(&self, context: &egui::Context, replacement: &Self) {
        for scroll in &self.scrolls {
            if !replacement.scrolls.iter().any(|next| next.id == scroll.id) {
                // Replace the stock scrollbar/background hit geometry with an
                // empty clip without relying on their private widget IDs.
                let mut ui = scroll.local.open(context);
                ui.set_clip_rect(egui::Rect::NOTHING);
                scroll_area(scroll.horizontal).show(&mut ui, |ui| ui.set_min_size(scroll.size));
            }
        }
        for id in self.ids.difference(&replacement.ids) {
            context.remove_input_widget(*id);
        }
        // Removal is also a cancellation boundary, including an instance that
        // disappears and is restored later in this same input batch.
        self.cancel_interactions();
    }
}

pub(super) struct PageLayout {
    pub chrome: Layout,
    pub body: Layout,
    pub body_ui: LocalUi,
}
impl PageLayout {
    pub fn entry(&self, id: egui::Id) -> Option<&Entry> {
        self.chrome
            .entries
            .iter()
            .chain(&self.body.entries)
            .find(|entry| entry.response.id == id)
    }

    pub fn cancel_interactions(&self) {
        self.chrome.cancel_interactions();
        self.body.cancel_interactions();
    }

    pub fn retire(&self, context: &egui::Context, replacement: &Self) {
        self.chrome.retire(context, &replacement.chrome);
        self.body.retire(context, &replacement.body);
    }
}

pub(super) fn scroll_opportunity(event: &egui::RoutedInput) -> bool {
    matches!(
        &event.trigger,
        egui::InputTrigger::Time
            | egui::InputTrigger::Event {
                event: egui::Event::MouseWheel { .. }
                    | egui::Event::PointerMoved(_)
                    | egui::Event::PointerButton { .. }
                    | egui::Event::PointerGone
                    | egui::Event::Touch { .. }
                    | egui::Event::WindowFocused(_),
                ..
            }
    )
}

fn scroll_area(horizontal: bool) -> egui::ScrollArea {
    if horizontal {
        egui::ScrollArea::horizontal().id_salt("scroll")
    } else {
        egui::ScrollArea::vertical().id_salt("scroll")
    }
}

impl Gallery {
    pub(super) fn control(&mut self, ui: &mut egui::Ui, control: &Control) -> Draw {
        let theme = self.theme();
        match control {
            Control::Page(_) => unreachable!("page tabs use the registered response"),
            Control::Dark(dark) => ui
                .selectable_value(&mut self.dark, *dark, if *dark { "Dark" } else { "Light" })
                .into(),
            Control::Narrow => ui.checkbox(&mut self.narrow, "320pt content").into(),
            Control::Zoom(scale) => {
                let response = ui.selectable_label(
                    ui.ctx().zoom_factor() == *scale,
                    if *scale == 1.0 { "100%" } else { "150%" },
                );
                if response.clicked() {
                    ui.ctx().set_zoom_factor(*scale);
                }
                response.into()
            }
            Control::Selected => ui.selectable_label(true, "Selected foreground").into(),
            Control::Button(control) => self.buttons.control(ui, &theme, *control),
            Control::Input(control) => self.inputs.control(ui, &theme, *control),
            Control::Checkbox(control) => self.checkboxes.control(ui, *control),
            Control::Slider(control) => self.sliders.stock(ui, *control).into(),
            Control::Switch(control) => self.switches.control(ui, *control),
            Control::Spinner(control) => self.spinners.control(ui, &theme, *control),
            Control::Select(control) => self.selects.control(ui, *control),
            Control::Tooltip(control) => self.tooltips.control(ui, *control),
            Control::Dialog(control) => self.dialogs.control(ui, *control),
            Control::Interaction(control) => {
                self.interaction
                    .control(ui, *control, self.case == Case::Layout)
            }
            Control::Compositing(control) => self.compositing.control(ui, *control),
            Control::Link(label, url) => ui.hyperlink_to(*label, *url).into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout(ui: &mut egui::Ui) -> Layout {
        let mut frame = Layout::new(Phase::Register);
        let mut local =
            ui.new_child(egui::UiBuilder::new().scope_id(egui::Id::unique("scroll-test")));
        frame.scroll(&mut local, false, |ui, frame| {
            ui.add_space(76.0);
            frame.widget(ui, Control::Selected, |ui| ui.button("Action").into());
            ui.add_space(360.0);
        });
        frame
    }

    #[test]
    fn local_shell_moves_hit_geometry_before_the_next_event() {
        let context = egui::Context::default();
        context.options_mut(|options| options.ordered_input = true);
        let input = |time, events| egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(200.0, 120.0),
            )),
            time: Some(time),
            events,
            ..Default::default()
        };
        let mut original = egui::Rect::NOTHING;
        context
            .run_ui(input(0.0, vec![]), |ui| {
                context.begin_input();
                original = layout(ui).entries[0].response.rect;
                while context.next_input_event().is_some() {}
            })
            .drop_without_applying_deltas();
        let pos = original.center() - egui::vec2(0.0, 40.0);
        let wheel = |phase, delta| egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: egui::vec2(0.0, delta),
            phase,
            source: egui::MouseWheelSource::Trackpad,
            modifiers: egui::Modifiers::NONE,
        };
        let pointer = |pressed| egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        let mut activations = 0;
        context
            .run_ui(
                input(
                    0.02,
                    vec![
                        egui::Event::PointerMoved(pos),
                        wheel(egui::TouchPhase::Start, 0.0),
                        wheel(egui::TouchPhase::Move, -40.0),
                        pointer(true),
                        pointer(false),
                    ],
                ),
                |ui| {
                    context.begin_input();
                    let mut frame = layout(ui);
                    while let Some(event) = context.next_input_event() {
                        if scroll_opportunity(&event) && frame.scroll_event(&context) {
                            context.input_layout(|| {
                                let next = layout(ui);
                                frame.retire(&context, &next);
                                frame = next;
                            });
                        }
                        let id = frame.entries[0].response.id;
                        if event.target == Some(id) && context.read_response(id).unwrap().clicked()
                        {
                            activations += 1;
                        }
                    }
                    assert!(frame.entries[0].response.rect.contains(pos));
                },
            )
            .drop_without_applying_deltas();
        assert_eq!(activations, 1);
    }
}
