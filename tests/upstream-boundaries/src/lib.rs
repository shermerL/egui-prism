//! Fixed unmodified-egui comparison, not a proof that no public adapter can work.

#[cfg(test)]
mod tests {
    use egui::{Context, Event, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};

    struct Observation {
        actions: [usize; 2],
        callbacks: usize,
        rectangles: [Rect; 2],
    }

    impl Default for Observation {
        fn default() -> Self {
            Self {
                actions: [0; 2],
                callbacks: 0,
                rectangles: [Rect::NOTHING; 2],
            }
        }
    }

    fn frame(context: &Context, state: &mut Observation, events: Vec<Event>, time: f64) {
        context
            .run_ui(
                RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(400., 200.))),
                    events,
                    time: Some(time),
                    ..Default::default()
                },
                |ui| {
                    state.callbacks += 1;
                    for i in 0..2 {
                        let response = ui.button(format!("Action {i}"));
                        state.rectangles[i] = response.rect;
                        if response.clicked() {
                            state.actions[i] += 1;
                        }
                    }
                },
            )
            .drop_without_applying_deltas();
    }

    fn click(position: Pos2) -> Vec<Event> {
        vec![
            Event::PointerMoved(position),
            Event::PointerButton {
                pos: position,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
            Event::PointerButton {
                pos: position,
                button: PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            },
        ]
    }

    fn observe(split: bool) -> Observation {
        let context = Context::default();
        let mut state = Observation::default();
        // Establish previous-pass geometry before comparing the same event order.
        frame(&context, &mut state, vec![], 0.);
        let first = click(state.rectangles[0].center());
        let second = click(state.rectangles[1].center());
        state.callbacks = 0;
        if split {
            // Even at identical time, two public frames execute the UI closure twice.
            frame(&context, &mut state, first, 0.1);
            frame(&context, &mut state, second, 0.1);
        } else {
            frame(
                &context,
                &mut state,
                first.into_iter().chain(second).collect(),
                0.1,
            );
        }
        state
    }

    #[test]
    fn ordinary_batch_and_split_public_frames_have_different_observables() {
        let batch = observe(false);
        let split = observe(true);
        assert_eq!(batch.actions, [0, 1]);
        assert_eq!(batch.callbacks, 1);
        assert_eq!(split.actions, [1, 1]);
        assert_eq!(split.callbacks, 2);
        // This rejects the claim that splitting is a transparent drop-in. It
        // does not reject a deliberately redesigned, phase-aware public adapter.
    }
}
