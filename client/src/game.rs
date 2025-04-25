/*
We use the State design pattern to implement our Game struct.
https://doc.rust-lang.org/book/ch17-03-oo-design-patterns.html
*/

use crate::button::Button;
use crate::input_box::InputBox;
use crate::map::Map;
use crate::object::Drawable3D;
use crate::player::Player;
use game_channel::{Channel, Packet, Winner};
use raylib::audio::RaylibAudio;
use raylib::core::texture::Image;
use raylib::prelude::*;
use std::net::{Shutdown, TcpStream};

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
            state: Some(Box::new(LobbyState::new(rl, thread, Winner::None))),
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
    winner: Winner,
}

impl LobbyState {
    fn new(rl: RaylibHandle, thread: RaylibThread, winner: Winner) -> Self {
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
            winner,
        }
    }
}

impl GameState for LobbyState {
    fn run(mut self: Box<Self>) -> Option<Box<dyn GameState>> {
        let text = match self.winner {
            Winner::Player => "YOU WON",
            Winner::Enemy => "YOU LOSE",
            Winner::None => "",
        };
        let text_width = self.rl.measure_text(text, 60);
        let text_x = Game::SCREEN_WIDTH / 2 - text_width / 2;

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

            // Draw winner
            d.draw_text(text, text_x, 100, 60, Color::BLACK);

            drop(d);

            // Check if button is clicked
            if self.play_button.is_clicked() {
                // Check if can connect to server
                let ip = self.input_box.get_text();
                if let Ok(stream) = TcpStream::connect(ip) {
                    let mut player = Player::default();

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

        self.channel
            .stream
            .set_nodelay(true)
            .expect("Set no-delay failed");

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
        // Enable audio
        let audio = RaylibAudio::init_audio_device().expect("Enabling audio failed");

        self.rl.disable_cursor();
        let mut ray: Option<Ray> = None;

        // Load crosshair texture
        let image = Image::load_image("./resources/crosshair003.png").expect("Load image failed");
        let texture = self
            .rl
            .load_texture_from_image(&self.thread, &image)
            .expect("Load texture failed");
        let half_width = texture.width / 2;
        let half_height = texture.height / 2;

        let fx_ouch_sound = audio
            .new_sound("./resources/ouch.mp3")
            .expect("Load ouch sound effects failed");

        let fx_gun_sound = audio
            .new_sound("./resources/gunshot.wav")
            .expect("Load sound from wave failed");

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
                .is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT)
            {
                self.rl.disable_cursor();
            }

            // Receive enemy position
            let _ = self.enemy.read_stats(&mut self.channel);

            // Receive player health
            let _ = self.player.read_health(&mut self.channel, &fx_ouch_sound);

            // Update player
            self.player.update(&self.rl, &self.map.objects, &mut ray);

            // Handle shooting
            if let Some(r) = ray {
                fx_gun_sound.play();
                let collision = self.enemy.collision(r);
                if collision.hit {
                    self.enemy.decrease_health();
                }
                ray = None;
            }

            // Send next player position
            self.player
                .write_stats(&mut self.channel)
                .expect("Send player position failed");

            // Send enemy health
            self.enemy
                .write_health(&mut self.channel)
                .expect("Send enemy health failed");

            // Draw
            let player_camera = self.player.get_camera();
            let mut d = self.rl.begin_drawing(&self.thread);
            d.clear_background(Color::SKYBLUE);
            self.player.draw_gun(&mut d, player_camera);
            self.enemy.draw(&mut d, player_camera);
            self.map.draw(&mut d, player_camera);

            // Draw health bar
            self.player.draw_health_bar(&mut d);

            // Draw Crosshair
            d.draw_texture(
                &texture,
                Game::SCREEN_WIDTH / 2 - half_width,
                Game::SCREEN_HEIGHT / 2 - half_height,
                Color::WHITE,
            );

            // Player and enemy position debugging
            let (x, z) = self.player.get_pos();
            let (x2, z2) = self.enemy.get_pos();
            d.draw_text(&format!("Self x: {}", x), 10, 10, 20, Color::RED);
            d.draw_text(&format!("Self z: {}", z), 10, 30, 20, Color::RED);
            d.draw_text(&format!("Enemy x: {}", x2), 10, 50, 20, Color::RED);
            d.draw_text(&format!("Enemy z: {}", z2), 10, 70, 20, Color::RED);
            drop(d);

            // Check if game over
            if self.player.get_health() == 0 {
                self.rl.enable_cursor();
                self.channel
                    .stream
                    .shutdown(Shutdown::Both)
                    .expect("Failed to shutdown TCP stream");
                break Some(Box::new(LobbyState::new(
                    self.rl,
                    self.thread,
                    Winner::Enemy,
                )));
            } else if self.enemy.get_health() == 0 {
                self.rl.enable_cursor();
                self.channel
                    .stream
                    .shutdown(Shutdown::Both)
                    .expect("Failed to shutdown TCP stream");
                break Some(Box::new(LobbyState::new(
                    self.rl,
                    self.thread,
                    Winner::Player,
                )));
            }
        }
    }
}
