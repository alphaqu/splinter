use std::mem::swap;
use std::time::Instant;

use eframe::egui::{Color32, Context, Id};

mod spinner;

pub use spinner::*;
use crate::ui::ProgressStatus;

pub struct ProgressSystem(pub Option<ProgressStatus>);

pub struct Progress {
    /// Current progress of the spinner.
    /// If this is `None` that means this spinner is Indeterminate.
    /// If it holds a value, the spinner is a Determinate spinner.
    pub progress: Option<f32>,
    /// The speed factor. Higher values give a faster spinner.
    pub speed: f32,
    /// Color of the running spinner
    pub color: Color32,
    /// Color of the spinner base
    pub track_color: Color32,
}

impl Progress {
    pub fn tick(&mut self, ctx: &Context, id: Id) -> [f32; 2] {
        ctx.data_mut(|v| {
            let state = v.get_temp_mut_or(
                id,
                ProgressState::Idle {
                    spin_start: Instant::now(),
                },
            );
            let [mut start, mut end] = state.tick(self.speed, self.progress);
            if end < start {
                swap(&mut start, &mut end);
            }
            [start, end]
        })
    }
}

#[derive(Clone)]
enum ProgressState {
    Idle {
        spin_start: Instant,
    },
    Transition {
        transition_start: Instant,
        extra_loop: bool,
        from: [f32; 2],
        to: Box<ProgressState>,
    },
    Deter {
        last_update: Instant,
        old_progress: f32,
        progress: f32,
    },
}

impl ProgressState {
    pub fn tick(&mut self, speed: f32, progress: Option<f32>) -> [f32; 2] {
        // If there is a next state, create a transition to that state from the current state.
        if let Some(mut next_state) = self.update(progress) {
            let from = self.calc_points(speed);
            let to = next_state.calc_points(speed);

            let from_length = from[1] - from[0];
            let to_length = (to[1] - to[0]);

            let start = from[0] % 1.0;
            let mut start_end = (1.0 - start) + to[0];

            let end_speed = (from_length - to_length) - start_end.abs();

            *self = ProgressState::Transition {
                transition_start: Instant::now(),
                extra_loop: end_speed >= 0.0,
                from,
                to: Box::new(next_state),
            };
        }

        self.calc_points(speed)
    }

    pub fn update(&mut self, progress: Option<f32>) -> Option<ProgressState> {
        match self {
            ProgressState::Idle { .. } => progress.map(|v| ProgressState::Deter {
                last_update: Instant::now(),
                old_progress: 0.0,
                progress: v,
            }),
            ProgressState::Deter {
                progress: old_progress,
                ..
            } => match progress {
                None => Some(ProgressState::Idle {
                    spin_start: Instant::now(),
                }),
                Some(value) => {
                    *old_progress = value;
                    None
                }
            },
            ProgressState::Transition { to, .. } => to.update(progress),
        }
    }

    pub fn calc_points(&mut self, speed: f32) -> [f32; 2] {
        match self {
            ProgressState::Idle { spin_start } => {
                let time = (spin_start.elapsed().as_secs_f32()) * speed;

                let pos = time % 1.0;
                let start_wave = pos;
                let end_wave = if pos > 0.25 { (pos - 0.25) / 0.75 } else { 0.0 };
                [
                    (time + (ease_out(start_wave))),
                    (time + (ease_out(end_wave))),
                ]
                //let pos = (time * 1.5) % 2.0;

                //let start_wave = if pos < 1.5 { pos / 1.5 } else { 1.0 };
                //let end_wave = if pos > 0.5 { (pos - 0.5) / 1.5 } else { 0.0 };

                //[
                //    (start_wave),
                //    (end_wave),
                //]
            }
            ProgressState::Transition {
                transition_start,
                extra_loop,
                from,
                to,
            } => {
                // Position of the transition
                let position =
                    ease_out((transition_start.elapsed().as_secs_f32() * speed).clamp(0.0, 1.0));

                // Calculate from
                let [from_start_raw, from_end] = *from;
                let from_start = from_start_raw % 1.0;
                let from_length = from_end - from_start_raw;

                // Calculate to
                let [to_start_raw, to_end] = to.calc_points(speed);
                let to_length = to_end - to_start_raw;

                // Calculate intended end location
                let mut start_end = (1.0 - from_start) + to_start_raw;

                // Add extra loop
                if *extra_loop {
                    start_end += 1.0;
                }

                let start_angle = from_start + lerp(0.0, start_end, position);
                let end_angle = from_start + lerp(from_length, to_length + start_end, position);

                [start_angle, end_angle]
            }
            ProgressState::Deter {
                last_update,
                old_progress,
                progress,
            } => {
                let value = ((last_update.elapsed().as_secs_f32() * 10.0) * speed).clamp(0.0, 1.0);
                let delta = (*progress - *old_progress) * value;
                let value = *old_progress + delta;

                *old_progress = value;
                *last_update = Instant::now();

                [0.0, value]
            }
        }
    }
}

fn ease_in_out(x: f32) -> f32 {
    if x < 0.5 {
        ease_in(x * 2.0) / 2.0
    } else {
        (ease_out(((x - 0.5) * 2.0)) / 2.0) + 0.5
    }
}

fn ease_in(x: f32) -> f32 {
    1.0 - (1.0 - x.powf(2.0)).sqrt()
}

fn ease_out(x: f32) -> f32 {
    (1.0 - (x - 1.0).powf(2.0)).sqrt()
}

fn lerp(a: f32, b: f32, f: f32) -> f32 {
    a + f * (b - a)
}
