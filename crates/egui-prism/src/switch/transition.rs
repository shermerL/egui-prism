use crate::egui;

// Per-property CSS transition state, including the reversing shortening factor.
#[derive(Clone)]
struct Transition<const N: usize> {
    from: [f32; N],
    to: [f32; N],
    reversing_start: [f32; N],
    shortening: f32,
    start: f64,
    duration: f64,
}

pub(super) fn animate<const N: usize>(
    ctx: &egui::Context,
    id: egui::Id,
    target: [f32; N],
) -> [f32; N] {
    let now = ctx.input(|input| input.time);
    let mut state = ctx
        .data_mut(|data| data.get_temp::<Transition<N>>(id))
        .unwrap_or(Transition {
            from: target,
            to: target,
            reversing_start: target,
            shortening: 1.0,
            start: now,
            duration: 0.15,
        });
    let fraction = ease(((now - state.start) / state.duration).clamp(0.0, 1.0) as f32);
    let current = std::array::from_fn(|i| egui::lerp(state.from[i]..=state.to[i], fraction));
    if state.to != target {
        let shortening = if target == state.reversing_start {
            (fraction * state.shortening + 1.0 - state.shortening)
                .abs()
                .clamp(0.0, 1.0)
        } else {
            1.0
        };
        state = Transition {
            from: current,
            to: target,
            reversing_start: state.to,
            shortening,
            start: now,
            duration: (0.15 * f64::from(shortening)).max(f64::EPSILON),
        };
    }
    if now < state.start + state.duration && state.from != state.to {
        ctx.request_repaint();
    }
    ctx.data_mut(|data| data.insert_temp(id, state));
    current
}

fn ease(x: f32) -> f32 {
    if x == 0.0 || x == 1.0 {
        return x;
    }
    // Invert cubic-bezier(0.4, 0, 0.2, 1)'s x coordinate.
    let (mut low, mut high) = (0.0_f32, 1.0_f32);
    for _ in 0..16 {
        let t = (low + high) * 0.5;
        let bx = 1.2 * (1.0 - t).powi(2) * t + 0.6 * (1.0 - t) * t * t + t * t * t;
        if bx < x {
            low = t;
        } else {
            high = t;
        }
    }
    let t = (low + high) * 0.5;
    t * t * (3.0 - 2.0 * t)
}

// CSS interpolates these colors in premultiplied Oklab, including the dark
// input's alpha. The standard Oklab matrices map linear sRGB through LMS.
pub(super) fn color_components(color: egui::Color32) -> [f32; 4] {
    let [r, g, b, a] = egui::Rgba::from(color).to_rgba_unmultiplied();
    let l = (0.41222146 * r + 0.53633255 * g + 0.051445995 * b).cbrt();
    let m = (0.2119035 * r + 0.6806995 * g + 0.10739696 * b).cbrt();
    let s = (0.08830246 * r + 0.28171885 * g + 0.6299787 * b).cbrt();
    [
        (0.21045426 * l + 0.7936178 * m - 0.004072047 * s) * a,
        (1.9779985 * l - 2.4285922 * m + 0.4505937 * s) * a,
        (0.025904037 * l + 0.78277177 * m - 0.80867577 * s) * a,
        a,
    ]
}

pub(super) fn color([lightness, a, b, alpha]: [f32; 4]) -> egui::Color32 {
    if alpha == 0.0 {
        return egui::Color32::TRANSPARENT;
    }
    let (lightness, a, b) = (lightness / alpha, a / alpha, b / alpha);
    let l = (lightness + 0.39633778 * a + 0.21580376 * b).powi(3);
    let m = (lightness - 0.105561346 * a - 0.06385417 * b).powi(3);
    let s = (lightness - 0.08948418 * a - 1.2914855 * b).powi(3);
    egui::Rgba::from_rgba_unmultiplied(
        (4.0767417 * l - 3.3077116 * m + 0.23096994 * s).clamp(0.0, 1.0),
        (-1.268438 * l + 2.6097574 * m - 0.34131938 * s).clamp(0.0, 1.0),
        (-0.0041960863 * l - 0.7034186 * m + 1.7076147 * s).clamp(0.0, 1.0),
        alpha,
    )
    .into()
}
