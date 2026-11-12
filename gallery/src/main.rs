//! Native acceptance gallery for the component library.

mod native;

mod badges;
mod buttons;
mod checkboxes;
mod compositing;
mod dialogs;
mod empty_states;
mod input;
mod interaction;
mod selects;
mod sliders;
mod switches;
mod text_inputs;
mod tooltips;

use egui_prism::{egui, theme::Theme};
use input::{Control, Layout, LocalUi, PageLayout, Phase};
use interaction::InteractionCase;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Case {
    Theme,
    Buttons,
    Badges,
    Empty,
    Inputs,
    Checkboxes,
    Sliders,
    Switches,
    Selects,
    Tooltips,
    Dialogs,
    Interaction,
    Layout,
    Compositing,
}
impl Case {
    fn name(self) -> &'static str {
        match self {
            Self::Theme => "Theme",
            Self::Buttons => "Buttons",
            Self::Badges => "Badges",
            Self::Empty => "Empty states",
            Self::Inputs => "Inputs",
            Self::Checkboxes => "Checkboxes",
            Self::Sliders => "Sliders",
            Self::Switches => "Switches",
            Self::Selects => "Selects",
            Self::Tooltips => "Tooltips",
            Self::Dialogs => "Dialogs",
            Self::Interaction => "Interaction",
            Self::Layout => "Layout",
            Self::Compositing => "Compositing",
        }
    }
}

struct Gallery {
    compositor: egui_prism_wgpu::Host,
    compositing: compositing::CompositingCase,
    dark: bool,
    case: Case,
    narrow: bool,
    interaction: InteractionCase,
    buttons: buttons::ButtonCase,
    inputs: text_inputs::InputCase,
    checkboxes: checkboxes::CheckboxCase,
    sliders: sliders::SliderCase,
    switches: switches::SwitchCase,
    selects: selects::SelectCase,
    tooltips: tooltips::TooltipCase,
    dialogs: dialogs::DialogCase,
    empty: empty_states::EmptyCase,
}

impl Gallery {
    fn new(
        context: &egui::Context,
        device: &wgpu::Device,
    ) -> Result<Self, egui_prism::compositing::Error> {
        context.options_mut(|options| options.ordered_input = true);
        let compositor = egui_prism_wgpu::Host::new(context, device)?;
        let automatic = std::env::args().any(|arg| arg == "--compositing-check");
        egui_prism::theme::install_fonts(context, egui::FontDefinitions::default());
        let gallery = Self {
            compositor,
            compositing: compositing::CompositingCase::new(context, automatic),
            dark: false,
            case: if automatic {
                Case::Compositing
            } else {
                Case::Theme
            },
            narrow: false,
            interaction: InteractionCase::default(),
            buttons: buttons::ButtonCase::new(context),
            inputs: text_inputs::InputCase::default(),
            checkboxes: checkboxes::CheckboxCase::default(),
            sliders: sliders::SliderCase::default(),
            switches: switches::SwitchCase::default(),
            selects: selects::SelectCase::default(),
            tooltips: tooltips::TooltipCase::default(),
            dialogs: dialogs::DialogCase::default(),
            empty: empty_states::EmptyCase::new(context),
        };
        gallery.apply_theme(context);
        Ok(gallery)
    }

    fn theme(&self) -> Theme {
        if self.dark {
            Theme::dark()
        } else {
            Theme::light()
        }
    }

    fn apply_theme(&self, context: &egui::Context) {
        let theme = self.theme();
        context.set_theme(if self.dark {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        });
        context.all_styles_mut(|style| theme.apply(style));
    }

    fn ui(&mut self, ui: &mut egui::Ui, device: &wgpu::Device) {
        let context = ui.ctx().clone();
        self.compositor.check().expect("compositing host failure");

        // Lifecycle work is not part of registration or targeted widget replay.
        self.compositing.begin_update();
        let restart = self.compositing.take_restart_request();

        // Registrations borrow these immutable resources, never Gallery values.
        // Both themes and every option variant survive all same-batch rebuilds.
        let select_resources = selects::Resources::new();
        let mut select_live = selects::Live::default();
        let tooltip_resources = tooltips::Resources::new();
        let mut tooltip_live = tooltips::Live::default();
        context.begin_input();
        let root = LocalUi::named(ui, ui.make_persistent_id("gallery-root"));
        let mut page = self.layout(
            &mut root.open(&context),
            Phase::Register,
            sliders::Live::default(),
            &select_resources,
            &mut select_live,
            (&tooltip_resources, &mut tooltip_live),
        );

        let dialog_themes = [Theme::light(), Theme::dark()];
        let mut dialog_live = (self.case == Case::Dialogs).then(|| {
            self.dialogs
                .register(&context, &dialog_themes[usize::from(self.dark)])
        });
        let mut focus = context.memory(|memory| memory.focused());
        while let Some(event) = context.next_input_event() {
            // Navigation has already run. Observe loss now, not just at the end
            // of the batch: focus may leave and return before Space is released.
            page.cancel_interactions();
            let old_page = (self.case, self.dark, self.narrow);

            // Includes untargeted releases, Time, and blur. Popup wheel input
            // must be consumed before any background scroll shell sees it.
            if let Some(live) = &mut dialog_live {
                self.dialogs
                    .handle(live, &dialog_themes[usize::from(self.dark)], &event);
            }
            let select_changed = self.selects.handle_all(&mut select_live, &event);
            // Hints observe every event but never consume ordinary activation.
            tooltip_live.handle(&event);
            let scrolled = input::scroll_opportunity(&event) && page.body.scroll_event(&context);

            let handled = event.target.and_then(|id| {
                let entry = page.entry(id)?;
                let control = entry.control.clone();
                if matches!(control, Control::Select(selects::Control::Instance)) {
                    // Already handled by the all-event Select dispatch above.
                    return None;
                }
                let response = if let Some(button) = &entry.button {
                    let response = button.handle(&event);
                    if response.clicked() {
                        match control {
                            Control::Button(control) => self.buttons.activated(control),
                            Control::Empty(control) => self.empty.activated(control),
                            _ => {}
                        }
                    }
                    response
                } else if let Some(registration) = &entry.checkbox {
                    let Control::Checkbox(control) = control else {
                        unreachable!("Checkbox registration control");
                    };
                    self.checkboxes.handle(control, registration, &event)
                } else if let Some(registration) = &entry.switch {
                    let Control::Switch(control) = control else {
                        unreachable!("Switch registration control");
                    };
                    self.switches.handle(control, registration, &event)
                } else if let Control::Page(page) = control {
                    let response = context.read_response(id).expect("registered page tab");
                    if response.clicked() {
                        self.case = page;
                    }
                    response
                } else if let Control::Slider(slider) = control {
                    self.sliders.handle(
                        &mut entry.open(&context),
                        slider,
                        &page.body.slider_live,
                        &event,
                    )
                } else {
                    self.control(&mut entry.open(&context), &control).response
                };
                Some((control, response))
            });

            page.cancel_interactions();

            let global = old_page != (self.case, self.dark, self.narrow)
                || handled.as_ref().is_some_and(|(control, response)| {
                    matches!(control, Control::Zoom(_)) && response.clicked()
                });
            let next_focus = context.memory(|memory| memory.focused());
            let focus_changed = focus != next_focus;
            focus = next_focus;
            if self.dark != old_page.1 {
                self.apply_theme(&context);
                context.request_repaint();
            }

            if global {
                if let Some(live) = dialog_live.take() {
                    live.unregister();
                }
                select_live.unregister();
                tooltip_live.unregister();
                context.input_layout(|| {
                    // Re-capture the root style after a theme change.
                    let mut root_ui = root.open(&context);
                    root_ui.set_style(context.global_style());
                    let next = self.layout(
                        &mut root_ui,
                        Phase::Register,
                        sliders::Live::default(),
                        &select_resources,
                        &mut select_live,
                        (&tooltip_resources, &mut tooltip_live),
                    );
                    page.retire(&context, &next);
                    page = next;
                    dialog_live = (self.case == Case::Dialogs).then(|| {
                        self.dialogs
                            .register(&context, &dialog_themes[usize::from(self.dark)])
                    });
                });
            } else if scrolled
                || handled
                    .as_ref()
                    .is_some_and(|(control, response)| control.changes_layout(response))
            {
                select_live.unregister();
                tooltip_live.unregister();
                context.input_layout(|| {
                    let next = self.body(
                        &mut page.body_ui.open(&context),
                        Phase::Register,
                        sliders::Live::default(),
                        &select_resources,
                        &mut select_live,
                        (&tooltip_resources, &mut tooltip_live),
                    );
                    page.body.retire(&context, &next);
                    page.body = next;
                });
            } else if select_changed
                || handled
                    .as_ref()
                    .is_some_and(|(_, response)| response.changed() || response.clicked())
                || focus_changed
            {
                context.input_layout(|| {
                    page.body
                        .refresh_statuses(&context, |ui, control| self.control(ui, control));
                });
            }
            if let Some(live) = &mut dialog_live {
                self.dialogs
                    .sync(live, &dialog_themes[usize::from(self.dark)]);
            }
        }

        page.cancel_interactions();

        // Keep the revoked-host probe in the final graphics, not in a discarded
        // registration/event list. Restart and screenshot lifecycle run once.
        if restart {
            self.restart_compositor(&context, device);
        }

        let mut root_ui = root.open(&context);
        root_ui.set_style(context.global_style());
        let _ = self.layout(
            &mut root_ui,
            Phase::Paint,
            page.body.slider_live,
            &select_resources,
            &mut select_live,
            (&tooltip_resources, &mut tooltip_live),
        );
        if let Some(live) = dialog_live {
            self.dialogs
                .paint(live, &dialog_themes[usize::from(self.dark)]);
        }
    }

    fn restart_compositor(&mut self, context: &egui::Context, device: &wgpu::Device) {
        let rect = egui::Rect::from_min_size(egui::pos2(4.0, 4.0), egui::vec2(8.0, 8.0));
        let painter = egui::Painter::new(
            context.clone(),
            egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::unique("revoked-compositor"),
            ),
            rect,
        );
        let group = egui_prism::compositing::Group::new(
            rect,
            1.0,
            vec![egui_prism::compositing::Item::Shape(
                egui::epaint::ClippedShape::new(
                    rect,
                    egui::Shape::rect_filled(rect, 0, egui::Color32::RED),
                ),
            )],
        )
        .expect("stale callback probe");
        egui_prism::compositing::paint(&painter, group).expect("old host");
        self.compositor.invalidate();
        self.compositor = egui_prism_wgpu::Host::new(context, device).expect("restart compositor");
    }

    fn widget(&mut self, ui: &mut egui::Ui, frame: &mut Layout, control: Control) {
        frame.widget(ui, control.clone(), |ui| self.control(ui, &control));
    }

    fn layout<'a>(
        &mut self,
        ui: &mut egui::Ui,
        phase: Phase,
        slider_live: sliders::Live,
        select_resources: &'a selects::Resources,
        select_live: &mut selects::Live<'a>,
        tooltip: (&'a tooltips::Resources, &mut tooltips::Live<'a>),
    ) -> PageLayout {
        let theme = self.theme();
        let automatic = self.compositing.automatic();
        let panel = if automatic {
            egui::CentralPanel::default()
        } else {
            egui::CentralPanel::default().frame(
                egui::Frame::central_panel(ui.style()).inner_margin(theme.metrics.page_margin),
            )
        };
        panel
            .show(ui, |ui| {
                let mut chrome = Layout::new(phase);
                if !automatic {
                    chrome.track(ui.heading("egui-prism"));
                    chrome.label(ui, "Parity correction in progress: reference tokens and Geist fonts; components await individual acceptance.");
                    ui.horizontal_wrapped(|ui| {
                        for case in [
                            Case::Theme,
                            Case::Compositing,
                            Case::Buttons,
                            Case::Badges,
                            Case::Empty,
                            Case::Inputs,
                            Case::Checkboxes,
                            Case::Sliders,
                            Case::Switches,
                            Case::Selects,
                            Case::Tooltips,
                            Case::Dialogs,
                            Case::Interaction,
                            Case::Layout,
                        ] {
                            // Let the parent horizontal layout wrap the whole tab.
                            let response = ui.selectable_label(self.case == case, case.name());
                            chrome.bind_page(case, response);
                        }
                    });
                    ui.horizontal_wrapped(|ui| {
                        self.widget(ui, &mut chrome, Control::Dark(false));
                        self.widget(ui, &mut chrome, Control::Dark(true));
                        self.widget(ui, &mut chrome, Control::Narrow);
                        chrome.label(ui, "Scale:");
                        self.widget(ui, &mut chrome, Control::Zoom(1.0));
                        self.widget(ui, &mut chrome, Control::Zoom(1.5));
                    });
                    chrome.track(ui.separator());
                }

                let body_ui = LocalUi::named(ui, ui.make_persistent_id("gallery-body"));
                let mut child = body_ui.open(ui.ctx());
                let body = self.body(&mut child, phase, slider_live, select_resources, select_live, tooltip);
                ui.advance_cursor_after_rect(child.min_rect());
                PageLayout {
                    chrome,
                    body,
                    body_ui,
                }
            })
            .inner
    }

    fn body<'a>(
        &mut self,
        ui: &mut egui::Ui,
        phase: Phase,
        mut slider_live: sliders::Live,
        select_resources: &'a selects::Resources,
        select_live: &mut selects::Live<'a>,
        tooltip: (&'a tooltips::Resources, &mut tooltips::Live<'a>),
    ) -> Layout {
        let theme = self.theme();
        let mut frame = Layout::new(phase);
        if self.compositing.automatic() {
            self.compositing.show(ui, &theme, &mut frame);
            return frame;
        }

        frame.scroll(ui, false, |ui, frame| {
            if self.narrow {
                ui.set_max_width(ui.available_width().min(320.0));
            }
            match self.case {
                Case::Compositing => self.compositing.show(ui, &theme, frame),
                Case::Theme => {
                    frame.track(ui.heading("Semantic palette"));
                    show_palette(ui, &theme, frame);
                    self.widget(ui, frame, Control::Selected);
                    frame.label(ui, "Opaque disabled tokens are paired colors. Standard egui disabled controls use opacity.");
                }
                Case::Buttons => self.buttons.show(ui, &theme, frame),
                Case::Badges => badges::show(ui, &theme, frame),
                Case::Empty => self.empty.show(ui, &theme, frame),
                Case::Inputs => self.inputs.show(ui, &theme, frame),
                Case::Checkboxes => self.checkboxes.show(ui, &theme, frame),
                Case::Sliders => self.sliders.layout(ui, &theme, |case, ui, control| {
                    frame.widget(ui, Control::Slider(control), |ui| {
                        match phase {
                            Phase::Register => case.register_control(ui, &theme, control, &mut slider_live),
                            Phase::Paint => case.paint_control(ui, control, &mut slider_live),
                        }.into()
                    })
                }),
                Case::Switches => self.switches.show(ui, &theme, frame),
                Case::Selects => self.selects.show(ui, select_resources, self.dark, frame, select_live),
                Case::Tooltips => self.tooltips.show(ui, tooltip.0, self.dark, frame, tooltip.1),
                Case::Dialogs => self.dialogs.show(ui, &theme, frame),
                Case::Interaction | Case::Layout => {
                    frame.track(ui.heading("Harness controls (egui)"));
                    frame.label(ui, "Activate with the pointer or Tab and Enter. Counts change once per activation and remain still while idle.");
                    ui.horizontal_wrapped(|ui| {
                        for control in [
                            interaction::Control::Disabled,
                            interaction::Control::Reversed,
                        ] {
                            self.widget(ui, frame, Control::Interaction(control));
                        }
                    });
                    if self.case == Case::Layout {
                        frame.label(ui, "Long labels use the same values and stable IDs as the interaction case. Try narrow content and 150% scale, then scroll to the second instance.");
                    }
                    self.interaction.show_ordered(ui, self.case == Case::Layout, frame);
                }
            }

            frame.track(ui.separator());
            frame.track(ui.small(format!(
                "Development v{} · {} · {}",
                env!("CARGO_PKG_VERSION"),
                if cfg!(target_os = "macos") {
                    "wgpu Metal"
                } else if cfg!(target_os = "windows") {
                    "wgpu DX12"
                } else {
                    "wgpu Vulkan"
                },
                if cfg!(debug_assertions) { "debug" } else { "release" }
            )));
            frame.track(ui.small(format!(
                "Build: {}",
                option_env!("EGUI_PRISM_BUILD_ID").unwrap_or("local development")
            )));
            ui.horizontal_wrapped(|ui| {
                for (label, url) in [
                    ("Source", "https://github.com/shermerL/egui-prism"),
                    (
                        "MIT license",
                        "https://github.com/shermerL/egui-prism/blob/main/LICENSE",
                    ),
                    (
                        "Third-party notices",
                        "https://github.com/shermerL/egui-prism/blob/main/THIRD_PARTY_NOTICES.md",
                    ),
                ] {
                    self.widget(ui, frame, Control::Link(label, url));
                }
            });
        });
        frame.slider_live = slider_live;
        frame
    }
}

fn show_palette(ui: &mut egui::Ui, theme: &Theme, frame: &mut Layout) {
    let pairs = [
        ("Page", theme.colors.background),
        ("Surface", theme.colors.surface),
        ("Muted", theme.colors.muted),
        ("Primary", theme.colors.primary),
        ("Selected", theme.colors.selection),
        ("Destructive", theme.colors.destructive),
        ("Disabled token", theme.colors.disabled),
        ("Focus / pressed", theme.colors.focus),
    ];
    let font = egui::TextStyle::Body.resolve(ui.style());
    let card_width = pairs
        .iter()
        .map(|(name, pair)| {
            ui.painter()
                .layout_no_wrap((*name).into(), font.clone(), pair.foreground)
                .size()
                .x
        })
        .fold(0.0_f32, f32::max)
        + theme.metrics.container_margin.sum().x
        + 2.0;
    let gap = theme.metrics.item_spacing.x;
    let columns = (((ui.available_width() + gap) / (card_width + gap)) as usize).clamp(1, 4);
    egui::Grid::new("semantic-palette")
        .num_columns(columns)
        .show(ui, |ui| {
            for (index, (name, pair)) in pairs.into_iter().enumerate() {
                egui::Frame::new()
                    .fill(pair.background)
                    .stroke(egui::Stroke::new(1.0, theme.colors.border))
                    .corner_radius(theme.metrics.corner_radius)
                    .inner_margin(theme.metrics.container_margin)
                    .show(ui, |ui| {
                        frame.track(
                            ui.add(
                                egui::Label::new(egui::RichText::new(name).color(pair.foreground))
                                    .wrap_mode(egui::TextWrapMode::Extend),
                            ),
                        );
                    });
                if (index + 1) % columns == 0 {
                    ui.end_row();
                }
            }
        });
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    native::run()
}
