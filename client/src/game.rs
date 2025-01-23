/*
We use the State design pattern to implement our Game struct.
https://doc.rust-lang.org/book/ch17-03-oo-design-patterns.html
*/

use crate::button::Button;
use crate::input_box::InputBox;
use crate::map::Map;
use crate::object::Drawable3D;
use crate::player::Player;
use raylib::prelude::*;
use std::net::TcpStream;

pub struct Game {
    state: Option<Box<dyn GameState>>,
}

impl Game {
    pub const SCREEN_WIDTH: i32 = 1024;
    pub const SCREEN_HEIGHT: i32 = 768;
    pub const FONT_SIZE: i32 = 20;

    pub fn new() -> Game {
        let (rl, thread) = raylib::init()
            .msaa_4x()
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
    rl: RaylibHandle,
    thread: RaylibThread,
    input_box: InputBox,
    play_button: Button,
    quit_button: Button,
    map: Map,
    camera: Camera3D,
}

impl LobbyState {
    fn new(rl: RaylibHandle, thread: RaylibThread) -> Self {
        let x = Game::SCREEN_WIDTH / 2 - Button::WIDTH / 2;
        let play_y = Game::SCREEN_HEIGHT / 2 - Button::HEIGHT / 2;
        let quit_y = Game::SCREEN_HEIGHT / 2 + Button::HEIGHT / 2 + Button::SPACING;

        // Camera
        let pos = Vector3::new(30.0, 20.0, 30.0);
        let target = Vector3::zero();
        let up = Vector3::up();
        let fovy = 60.0;
        let camera = Camera3D::perspective(pos, target, up, fovy);

        LobbyState {
            input_box: InputBox::default(),
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
            map: Map::default(),
            camera,
        }
    }
}

impl GameState for LobbyState {
    fn run(mut self: Box<Self>) -> Option<Box<dyn GameState>> {
        loop {
            if self.rl.window_should_close() {
                // Return None to signal game over, no new state
                break None;
            }
            // Update
            self.rl
                .update_camera(&mut self.camera, CameraMode::CAMERA_ORBITAL);

            self.input_box.update(&mut self.rl);
            self.play_button.update(&self.rl);
            self.quit_button.update(&self.rl);

            // Draw
            let mut d = self.rl.begin_drawing(&self.thread);

            d.clear_background(Color::SKYBLUE);

            self.map.draw(&mut d, &self.camera);

            self.input_box.draw(&mut d);
            self.play_button.draw(&mut d);
            self.quit_button.draw(&mut d);

            drop(d);

            // Check if button is clicked
            if self.play_button.is_clicked() {
                // Check if can connect to server
                // let ip = self.input_box.get_text();
                let stream = match TcpStream::connect("127.0.0.1:1234") {
                    Ok(s) => s,
                    Err(_) => {
                        self.play_button.toggle_clicked();
                        continue;
                    }
                };

                // Can connect to server, try building WaitState
                if let Some(s) = WaitState::try_build(self.rl, self.thread, stream, self.map) {
                    break Some(Box::new(s));
                } else {
                    break None;
                }
            }

            if self.quit_button.is_clicked() {
                break None;
            }
        }
    }
}

struct WaitState {
    rl: RaylibHandle,
    thread: RaylibThread,
    stream: TcpStream,
    player: Player,
    map: Map,
}

impl WaitState {
    fn try_build(
        rl: RaylibHandle,
        thread: RaylibThread,
        mut stream: TcpStream,
        map: Map,
    ) -> Option<Self> {
        // Read player position from stream
        let mut player = Player::default();
        match player.read_pos(&mut stream) {
            Ok(_) => Some(WaitState {
                rl,
                thread,
                stream,
                player,
                map,
            }),
            Err(_) => None,
        }
    }
}

impl GameState for WaitState {
    fn run(mut self: Box<Self>) -> Option<Box<dyn GameState>> {
        // self.rl.disable_cursor();
        loop {
            if self.rl.window_should_close() {
                break None;
            }

            // Update
            self.player.update(&self.rl);
            self.player
                .write_pos(&mut self.stream)
                .expect("Sending player pos to server");

            // Draw
            let mut d = self.rl.begin_drawing(&self.thread);
            d.clear_background(Color::SKYBLUE);
            self.map.draw(&mut d, self.player.get_camera());
            d.draw_text("Waiting for enemy...", 200, 200, 50, Color::BLACK);
        }
    }
}
