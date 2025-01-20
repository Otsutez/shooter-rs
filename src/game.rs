use crate::config;
use raylib::prelude::*;

pub struct Game {
    is_running: bool,
    rl: RaylibHandle,
    thread: RaylibThread,
}

impl Game {
    pub fn new() -> Game {
        let (rl, thread) = raylib::init()
            .size(config::SCREEN_WIDTH, config::SCREEN_HEIGHT)
            .title("Shooter-rs")
            .build();

        Game {
            is_running: true,
            rl,
            thread,
        }
    }

    pub fn run(&mut self) {
        while !self.rl.window_should_close() {
            let mut d = self.rl.begin_drawing(&self.thread);

            d.clear_background(Color::WHITE);
            d.draw_text("Hello, world!", 12, 12, 20, Color::BLACK);
        }
    }
}
