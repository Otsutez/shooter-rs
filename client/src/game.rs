/*
We use the State design pattern to implement our Game struct.
https://doc.rust-lang.org/book/ch17-03-oo-design-patterns.html
*/

use crate::button::Button;
use crate::input_box::InputBox;
use crate::map::Map;
use crate::object::Drawable3D;
use crate::player::Player;
use game_channel::{Channel, Packet};
use raylib::prelude::*;
use std::net::TcpStream;

pub struct Game {
    state: Option<Box<dyn GameState>>,
}

impl Game {
    pub const SCREEN_WIDTH: i32 = 1280;
    pub const SCREEN_HEIGHT: i32 = 720;
    pub const FONT_SIZE: i32 = 20;

    pub fn new() -> Game {
        let (mut rl, thread) = raylib::init()
            .msaa_4x()
            .size(Self::SCREEN_WIDTH, Self::SCREEN_HEIGHT)
            .title("Shooter-rs")
            .build();

        // Set exit key to nothing
        rl.set_exit_key(None);

        // Set FPS
        rl.set_target_fps(60);

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
        let pos = Vector3::new(20.0, 13.0, 20.0);
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
                if let Ok(stream) = TcpStream::connect("127.0.0.1:1234") {
                    let mut player = Player::default();
                    // stream
                    //     .set_nonblocking(true)
                    //     .expect("Set non blocking failed");

                    let mut channel = Channel::with_stream(stream);
                    if let Ok(_) = player.read_stats(&mut channel) {
                        break Some(Box::new(WaitState::new(
                            self.rl,
                            self.thread,
                            channel,
                            player,
                            self.map,
                            self.camera,
                        )));
                    }
                }
                self.play_button.toggle_clicked();
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
    channel: Channel<TcpStream>,
    player: Player,
    map: Map,
    camera: Camera3D,
}

impl WaitState {
    fn new(
        rl: RaylibHandle,
        thread: RaylibThread,
        channel: Channel<TcpStream>,
        player: Player,
        map: Map,
        camera: Camera3D,
    ) -> Self {
        WaitState {
            rl,
            thread,
            channel,
            player,
            map,
            camera,
        }
    }
}

impl GameState for WaitState {
    fn run(mut self: Box<Self>) -> Option<Box<dyn GameState>> {
        let text = "Waiting for enemy...";
        let text_width = self.rl.measure_text(text, 50);
        let text_x = Game::SCREEN_WIDTH / 2 - text_width / 2;

        self.channel
            .stream
            .set_nonblocking(true)
            .expect("Set non-blocking failed");

        let mut enemy = Player::default();

        loop {
            if self.rl.window_should_close() {
                break None;
            }

            // Update
            self.rl
                .update_camera(&mut self.camera, CameraMode::CAMERA_ORBITAL);

            // Draw
            let mut d = self.rl.begin_drawing(&self.thread);
            d.clear_background(Color::SKYBLUE);
            self.map.draw(&mut d, &self.camera);
            self.player.draw(&mut d, &self.camera);
            d.draw_text(text, text_x, 100, 50, Color::BLACK);
            drop(d);

            // Check if enemy position sent
            if let Ok(_) = enemy.read_stats(&mut self.channel) {
                break Some(Box::new(CountDownState::new(
                    self.rl,
                    self.thread,
                    self.channel,
                    self.player,
                    enemy,
                    self.map,
                    self.camera,
                )));
            }
        }
    }
}

struct CountDownState {
    rl: RaylibHandle,
    thread: RaylibThread,
    channel: Channel<TcpStream>,
    player: Player,
    enemy: Player,
    map: Map,
    camera: Camera3D,
}

impl CountDownState {
    fn new(
        rl: RaylibHandle,
        thread: RaylibThread,
        channel: Channel<TcpStream>,
        player: Player,
        enemy: Player,
        map: Map,
        camera: Camera3D,
    ) -> Self {
        CountDownState {
            rl,
            thread,
            channel,
            player,
            enemy,
            map,
            camera,
        }
    }
}

impl GameState for CountDownState {
    fn run(mut self: Box<Self>) -> Option<Box<dyn GameState>> {
        let mut text = String::from("3");
        let text_width = self.rl.measure_text(&text, 50);
        let text_x = Game::SCREEN_WIDTH / 2 - text_width / 2;
        loop {
            if self.rl.window_should_close() {
                break None;
            }

            // Receive time from server
            if let Ok(Packet::Time(time)) = self.channel.receive() {
                text = time.to_string();
            }

            if &text == "0" {
                break Some(Box::new(PlayState::new(
                    self.rl,
                    self.thread,
                    self.channel,
                    self.player,
                    self.enemy,
                    self.map,
                )));
            }

            // Update
            self.rl
                .update_camera(&mut self.camera, CameraMode::CAMERA_ORBITAL);

            // Draw
            let mut d = self.rl.begin_drawing(&self.thread);
            d.clear_background(Color::SKYBLUE);
            self.map.draw(&mut d, &self.camera);
            self.player.draw(&mut d, &self.camera);
            self.enemy.draw(&mut d, &self.camera);
            d.draw_text(&text, text_x, 100, 50, Color::BLACK);
        }
    }
}

struct PlayState {
    rl: RaylibHandle,
    thread: RaylibThread,
    channel: Channel<TcpStream>,
    player: Player,
    enemy: Player,
    map: Map,
}

impl PlayState {
    fn new(
        rl: RaylibHandle,
        thread: RaylibThread,
        channel: Channel<TcpStream>,
        player: Player,
        enemy: Player,
        map: Map,
    ) -> Self {
        PlayState {
            rl,
            thread,
            channel,
            player,
            enemy,
            map,
        }
    }
}

impl GameState for PlayState {
    fn run(mut self: Box<Self>) -> Option<Box<dyn GameState>> {
        self.rl.disable_cursor();

        loop {
            if self.rl.window_should_close() {
                break None;
            }

            // Allow player to free or lock mouse cursor
            if self.rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
                self.rl.enable_cursor();
            }
            if self
                .rl
                .is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
            {
                self.rl.disable_cursor();
            }

            // Receive enemy position
            let _ = self.enemy.read_stats(&mut self.channel);

            // Update player position
            self.player.update(&self.rl, &self.map.objects);

            // Send next player position
            self.player
                .write_stats(&mut self.channel)
                .expect("Send player position failed");

            // Draw
            let player_camera = self.player.get_camera();
            let mut d = self.rl.begin_drawing(&self.thread);
            d.clear_background(Color::SKYBLUE);
            self.player.draw_gun(&mut d, player_camera);
            self.enemy.draw(&mut d, player_camera);
            self.map.draw(&mut d, player_camera);

            // Player and enemy position debugging
            let (x, z) = self.player.get_pos();
            let (x2, z2) = self.enemy.get_pos();
            d.draw_text(&format!("Self x: {}", x), 0, 0, 20, Color::RED);
            d.draw_text(&format!("Self z: {}", z), 0, 20, 20, Color::RED);
            d.draw_text(&format!("Enemy x: {}", x2), 0, 40, 20, Color::RED);
            d.draw_text(&format!("Enemy z: {}", z2), 0, 60, 20, Color::RED);
        }
    }
}
