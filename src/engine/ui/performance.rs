use egui::{Color32, Context, Id, LayerId, Order, Painter, Rangef, Rect, Stroke, Vec2, Window};
use std::{collections::VecDeque, time::Instant};

const BACKLOG: usize = 100;
const HEIGHT: f32 = 100.0;
const AVG: usize = 25;

pub struct Stats {
    fps: VecDeque<f32>,
    timings: VecDeque<f32>,
    last: Instant,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            fps: VecDeque::with_capacity(BACKLOG),
            timings: VecDeque::with_capacity(BACKLOG),
            last: Instant::now(),
        }
    }

    fn push(&mut self, fps: f32, timing: f32) {
        if self.fps.len() >= BACKLOG {
            self.fps.pop_back();
            self.fps.push_front(fps);
        } else {
            self.fps.push_front(fps);
        }

        if self.timings.len() >= BACKLOG {
            self.timings.pop_back();
            self.timings.push_front(timing);
        } else {
            self.timings.push_front(timing);
        }
    }

    pub fn record(&mut self) {
        let now = Instant::now();

        let delta = self.last.elapsed().as_secs_f32();

        self.push(1.0 / delta, delta * 1000f32);

        self.last = now;
    }

    pub fn avg_fps(&self, n: usize) -> f32 {
        self.fps.iter().take(n).sum::<f32>() / n as f32
    }

    pub fn avg_timing(&self, n: usize) -> f32 {
        self.timings.iter().take(n).sum::<f32>() / n as f32
    }

    pub fn render(&mut self, context: &Context) {
        let window = Window::new("Performance")
            .default_width(300.0)
            .default_height(400.0)
            .min_width(300.0)
            .min_height(400.0)
            .resizable([true, true])
            .scroll(false);

        window.show(context, |ui| {
            ui.label(format!(
                "FPS {:.3} / {:.3}ms",
                self.avg_fps(AVG),
                self.avg_timing(AVG)
            ));

            let width = ui.available_width();

            let pos = ui.next_widget_position();

            ui.add_space(HEIGHT);

            let painter = Painter::new(
                context.clone(),
                LayerId::new(Order::Foreground, Id::new(0)),
                Rect::from_min_size(pos, Vec2::new(width, HEIGHT)),
            );

            let x_delta = width / BACKLOG as f32 / 2f32;

            let mut x = pos.x;

            for n in &self.fps {
                painter.vline(
                    x,
                    Rangef::new(pos.y + HEIGHT - n.min(HEIGHT), pos.y + HEIGHT),
                    Stroke::new(x_delta * 2f32, Color32::WHITE),
                );

                x += x_delta;
            }
        });
    }
}

impl Default for Stats {
    fn default() -> Self {
        Self::new()
    }
}
