//! Immutable drawing construction. Input registration and mutation stay in the
//! parent module; this module never creates widgets or input layers.

use std::hash::Hash;

use super::{SelectRegistration, State, transition};
use crate::{
    compositing::{self, Group, Item},
    egui,
};

fn item(clip: egui::Rect, shape: egui::Shape) -> Item {
    Item::Shape(egui::epaint::ClippedShape::new(clip, shape))
}

fn chevron(center: egui::Pos2, up: bool, color: egui::Color32) -> egui::Shape {
    let direction = if up { -1.0 } else { 1.0 };
    egui::Shape::line(
        vec![
            center + egui::vec2(-4.0, -2.0 * direction),
            center + egui::vec2(0.0, 2.0 * direction),
            center + egui::vec2(4.0, -2.0 * direction),
        ],
        egui::Stroke::new(4.0 / 3.0, color),
    )
}

fn text(
    clip: egui::Rect,
    x: f32,
    center_y: f32,
    galley: &std::sync::Arc<egui::Galley>,
    color: egui::Color32,
) -> Item {
    item(
        clip,
        egui::Shape::galley(
            egui::pos2(x, center_y - galley.size().y * 0.5) - galley.rect.min.to_vec2(),
            galley.clone(),
            color,
        ),
    )
}

fn transformed(
    group: Group,
    transform: egui::emath::TSTransform,
    screen: egui::Rect,
) -> Result<Group, compositing::Error> {
    let mut items = Vec::with_capacity(group.items().len());
    for child in group.items() {
        match child {
            Item::Group(child) => {
                items.push(Item::Group(transformed(child.clone(), transform, screen)?));
            }
            Item::Shape(shape) => {
                let mut shape = shape.clone();
                shape.clip_rect = (transform * shape.clip_rect).intersect(screen);
                shape.transform_after_tessellation = transform;
                items.push(Item::Shape(shape));
            }
        }
    }
    Group::new(transform * group.bounds(), group.opacity(), items)
}

pub(super) fn submit<T: Clone + Eq + Hash>(
    registration: &SelectRegistration<'_, T>,
    selected: &Option<T>,
    response: &egui::Response,
    state: &State,
    painter: &egui::Painter,
) -> Result<(), compositing::Error> {
    let ctx = &response.ctx;
    let colors = registration.theme.colors;
    let rect = response.rect;
    let bounds = rect.expand(5.0);
    let clip = registration.clip.intersect(bounds);

    if response.visible() && clip.is_positive() && clip.is_finite() {
        let focus_visible = response.enabled() && response.has_focus() && state.keyboard;
        let [focus] = transition::animate(
            ctx,
            registration.key.with("focus-transition"),
            [if focus_visible { 1.0 } else { 0.0 }],
        );
        let fill = if registration.theme.dark {
            colors
                .input
                .gamma_multiply(if response.hovered() { 0.5 } else { 0.3 })
        } else {
            egui::Color32::TRANSPARENT
        };
        let fill = transition::color(transition::animate(
            ctx,
            registration.key.with("fill-transition"),
            transition::color_components(fill),
        ));
        let border = if registration.invalid {
            colors.destructive.background
        } else if focus_visible {
            colors.ring
        } else {
            colors.input
        };
        let border = transition::color(transition::animate(
            ctx,
            registration.key.with("border-transition"),
            transition::color_components(border),
        ));
        let ring = if registration.invalid {
            colors
                .destructive
                .background
                .gamma_multiply(if registration.theme.dark { 0.4 } else { 0.2 })
        } else {
            colors.ring.gamma_multiply(0.5)
        };
        let ring = transition::color(transition::animate(
            ctx,
            registration.key.with("ring-transition"),
            transition::color_components(ring),
        ));

        let mut items = vec![item(
            clip,
            egui::epaint::Shadow {
                offset: [0, 1],
                blur: 2,
                spread: 0,
                color: egui::Color32::from_black_alpha(13),
            }
            .as_shape(rect, 8)
            .into(),
        )];
        if focus > 0.0 {
            items.push(item(
                clip,
                egui::Shape::rect_stroke(
                    rect.expand(1.5),
                    8,
                    egui::Stroke::new(3.0, ring.gamma_multiply(focus)),
                    egui::StrokeKind::Middle,
                ),
            ));
        }
        items.push(item(clip, egui::Shape::rect_filled(rect, 8, fill)));
        items.push(item(
            clip,
            egui::Shape::rect_stroke(
                rect,
                8,
                egui::Stroke::new(1.0, border),
                egui::StrokeKind::Inside,
            ),
        ));

        let label_clip = clip.intersect(egui::Rect::from_min_max(
            egui::pos2(rect.left() + 13.0, rect.top() + 1.0),
            egui::pos2(rect.right() - 37.0, rect.bottom() - 1.0),
        ));
        if label_clip.is_positive() {
            if let Some(index) = registration.selected_index(selected) {
                items.push(text(
                    label_clip,
                    rect.left() + 13.0,
                    rect.center().y,
                    &registration.galleys[index],
                    colors.background.foreground,
                ));
            } else if selected.is_none() {
                items.push(text(
                    label_clip,
                    rect.left() + 13.0,
                    rect.center().y,
                    &registration.placeholder_galley,
                    colors.muted.foreground,
                ));
            }
        }
        items.push(item(
            clip,
            chevron(
                egui::pos2(rect.right() - 21.0, rect.center().y),
                false,
                colors.muted.foreground.gamma_multiply(0.5),
            ),
        ));

        let trigger = Group::new(bounds, if response.enabled() { 1.0 } else { 0.5 }, items)?;
        compositing::paint(&painter.with_clip_rect(clip), trigger)?;
    }

    let now = registration.now();
    let progress = state.fade.value(now);
    if state.fade.moving(now) {
        ctx.request_repaint();
    }
    if progress <= 0.0 || !response.visible() || !response.enabled() {
        return Ok(());
    }
    if state.open
        && (registration.top_modal() != Some(registration.popup_layer())
            || !registration.owns_focus(state))
    {
        return Ok(());
    }

    let geometry = registration.geometry(state);
    let menu = geometry.menu;
    let outer = menu.expand(12.0);
    let clip = registration.screen.intersect(outer);
    let mut items = Vec::new();

    for (shadow, inset) in [
        (
            egui::epaint::Shadow {
                offset: [0, 4],
                blur: 6,
                spread: 0,
                color: egui::Color32::from_black_alpha(26),
            },
            1.0,
        ),
        (
            egui::epaint::Shadow {
                offset: [0, 2],
                blur: 4,
                spread: 0,
                color: egui::Color32::from_black_alpha(26),
            },
            2.0,
        ),
    ] {
        items.push(item(
            clip,
            shadow
                .as_shape(menu.shrink(inset), (8.0 - inset) as u8)
                .into(),
        ));
    }
    items.push(item(
        clip,
        egui::Shape::rect_filled(menu, 8, colors.surface.background),
    ));
    items.push(item(
        clip,
        egui::Shape::rect_stroke(
            menu,
            8,
            egui::Stroke::new(1.0, colors.border),
            egui::StrokeKind::Inside,
        ),
    ));

    let selected_index = registration.selected_index(selected);
    for (index, row) in geometry.rows.iter().copied().enumerate() {
        let row_clip = row
            .intersect(geometry.viewport)
            .intersect(registration.screen);
        if !row_clip.is_positive() {
            continue;
        }
        let disabled = registration.items[index].disabled;
        let active = !disabled && state.active == Some(registration.ids[index]);
        let foreground = if active {
            colors.accent.foreground
        } else {
            colors.surface.foreground
        };
        let mut row_items = Vec::new();
        if active {
            row_items.push(item(
                row_clip,
                egui::Shape::rect_filled(row, 6, colors.accent.background),
            ));
        }
        let label_clip = row_clip.intersect(egui::Rect::from_min_max(
            egui::pos2(row.left() + 8.0, row.top()),
            egui::pos2(row.right() - 32.0, row.bottom()),
        ));
        if label_clip.is_positive() {
            row_items.push(text(
                label_clip,
                row.left() + 8.0,
                row.center().y,
                &registration.galleys[index],
                foreground,
            ));
        }
        if selected_index == Some(index) {
            let center = egui::pos2(row.right() - 15.0, row.center().y);
            row_items.push(item(
                row_clip,
                egui::Shape::line(
                    vec![
                        center + egui::vec2(-16.0 / 3.0, 0.0),
                        center + egui::vec2(-2.0, 10.0 / 3.0),
                        center + egui::vec2(16.0 / 3.0, -4.0),
                    ],
                    egui::Stroke::new(4.0 / 3.0, foreground),
                ),
            ));
        }
        items.push(Item::Group(Group::new(
            row,
            if disabled { 0.5 } else { 1.0 },
            row_items,
        )?));
    }

    for (rect, up) in [(geometry.up, true), (geometry.down, false)] {
        if let Some(rect) = rect {
            items.push(item(
                rect.intersect(registration.screen),
                chevron(rect.center(), up, colors.surface.foreground),
            ));
        }
    }

    let popup = Group::new(outer, progress, items)?;
    let scale = 0.95 + 0.05 * progress;
    let origin = menu.center();
    let transform = egui::emath::TSTransform {
        scaling: scale,
        translation: origin.to_vec2() * (1.0 - scale),
    };
    let popup = transformed(popup, transform, registration.screen)?;

    // The portal escapes the trigger's clip and local transform, retaining the
    // clean host painter's inherited opacity exactly once.
    let mut portal = painter.clone();
    portal.set_layer_id(registration.popup_layer());
    portal.set_clip_rect(registration.screen);
    compositing::paint(&portal, popup)
}
