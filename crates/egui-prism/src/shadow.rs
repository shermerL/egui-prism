//! CSS outer shadow with the rounded border box excluded.

use crate::egui;

pub(crate) fn outer(ctx: &egui::Context, rect: egui::Rect, radius: u8) -> egui::Shape {
    use egui::epaint::{Mesh, Primitive, RoundedRect, Vertex, tessellator::path};

    let mut shadow = Mesh::default();
    let shape: egui::Shape = egui::epaint::Shadow {
        offset: [0, 1],
        blur: 2,
        spread: 0,
        color: egui::Color32::from_black_alpha(13),
    }
    .as_shape(rect, radius)
    .into();
    for primitive in ctx.tessellate(
        vec![egui::epaint::ClippedShape::new(rect.expand(5.0), shape)],
        ctx.pixels_per_point(),
    ) {
        let Primitive::Mesh(part) = primitive.primitive else {
            unreachable!("a shadow cannot emit a callback");
        };
        shadow.append(part);
    }
    let mut hole = Vec::new();
    path::rounded_rectangle(&mut hole, RoundedRect::new(rect, f32::from(radius)));

    // CSS outer shadows exclude the unshifted rounded border-box, not its fill.
    // Keep cut colors premultiplied and unrounded until the final mesh vertices.
    let mut mesh = Mesh::default();
    for triangle in shadow.indices.chunks_exact(3) {
        let mut remaining: Vec<_> = triangle
            .iter()
            .map(|&index| {
                let vertex = shadow.vertices[index as usize];
                (vertex.pos, vertex.color.to_array().map(f32::from))
            })
            .collect();
        for (&a, &b) in hole.iter().zip(hole.iter().cycle().skip(1)) {
            if remaining.len() < 3 {
                break;
            }
            let edge = b - a;
            let side = |p: egui::Pos2| edge.x * (p.y - a.y) - edge.y * (p.x - a.x);
            let winding = side(rect.center()).signum();
            let mut inside = Vec::new();
            let mut outside = Vec::new();
            for (&from, &to) in remaining.iter().zip(remaining.iter().cycle().skip(1)) {
                let from_distance = side(from.0) * winding;
                let to_distance = side(to.0) * winding;
                if (from_distance >= 0.0) != (to_distance >= 0.0) {
                    let t = from_distance / (from_distance - to_distance);
                    let cut = (
                        from.0 + (to.0 - from.0) * t,
                        std::array::from_fn(|channel| {
                            from.1[channel] + (to.1[channel] - from.1[channel]) * t
                        }),
                    );
                    inside.push(cut);
                    outside.push(cut);
                }
                if to_distance >= 0.0 {
                    inside.push(to);
                } else {
                    outside.push(to);
                }
            }

            // Each emitted piece is outside this edge and inside all previous edges.
            if outside.len() >= 3 {
                let start = mesh.vertices.len() as u32;
                mesh.vertices.extend(outside.iter().map(|&(pos, color)| {
                    let [r, g, b, a] = color.map(|channel| channel.round() as u8);
                    Vertex::untextured(pos, egui::Color32::from_rgba_premultiplied(r, g, b, a))
                }));
                for index in 1..outside.len() - 1 {
                    mesh.add_triangle(start, start + index as u32, start + index as u32 + 1);
                }
            }
            remaining = inside;
        }
    }
    egui::Shape::mesh(mesh)
}
