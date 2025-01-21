/*
We use the State design pattern to implement our Game struct.
https://doc.rust-lang.org/book/ch17-03-oo-design-patterns.html
*/

use crate::button::Button;
use raylib::prelude::*;

pub struct Game {
    state: Option<Box<dyn GameState>>,
}

impl Game {
    const SCREEN_WIDTH: i32 = 1024;
    const SCREEN_HEIGHT: i32 = 768;

    pub fn new() -> Game {
        let (rl, thread) = raylib::init()
            .size(Self::SCREEN_WIDTH, Self::SCREEN_HEIGHT)
            .title("Shooter-rs")
            .build();

        Game {
            state: Some(Box::new(LobbyState::new(rl, thread))),
        }
    }

    pub fn run(&mut self) {
        while let Some(s) = self.state.take() {
            self.state = s.run();
        }
    }
}

// ----------------------------------------------------------------------------
// Game States
// ----------------------------------------------------------------------------

trait GameState {
    fn run(self: Box<Self>) -> Option<Box<dyn GameState>>;
}

struct LobbyState {
    play_button: Button,
    quit_button: Button,
    rl: RaylibHandle,
    thread: RaylibThread,
}

impl LobbyState {
    fn new(rl: RaylibHandle, thread: RaylibThread) -> Self {
        let x = Game::SCREEN_WIDTH / 2 - Button::WIDTH / 2;
        let play_y = Game::SCREEN_HEIGHT / 2 - Button::HEIGHT - Button::SPACING / 2;
        let quit_y = Game::SCREEN_HEIGHT / 2 + Button::SPACING / 2;
        LobbyState {
            play_button: Button::new(
                Rectangle {
                    x: x as f32,
                    y: play_y as f32,
                    width: Button::WIDTH as f32,
                    height: Button::HEIGHT as f32,
                },
                String::from("PLAY"),
            ),
            quit_button: Button::new(
                Rectangle {
                    x: x as f32,
                    y: quit_y as f32,
                    width: Button::WIDTH as f32,
                    height: Button::HEIGHT as f32,
                },
                String::from("QUIT"),
            ),
            rl,
            thread,
        }
    }
}
// struct PlayState;

impl GameState for LobbyState {
    fn run(mut self: Box<Self>) -> Option<Box<dyn GameState>> {
        while !self.rl.window_should_close() {
            // Update
            self.play_button.update(&self.rl);
            self.quit_button.update(&self.rl);

            // draw
            let mut d = self.rl.begin_drawing(&self.thread);
            self.play_button.draw(&mut d);
            self.quit_button.draw(&mut d);
        }

        None
    }
}
// impl GameState for PlayState {}
