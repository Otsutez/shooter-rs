use crate::game::Game;
use crate::gun::Pistol;
use crate::object::{Cuboid, Drawable3D, Movable};
use game_channel::{Channel, ChannelVector2, Packet};
use raylib::audio::Sound;
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::ffi::{KeyboardKey, MouseButton};
use raylib::math::{BoundingBox, RayCollision};
use raylib::math::{Quaternion, Ray, Vector2, Vector3};
use raylib::prelude::RaylibDraw;
use raylib::RaylibHandle;
use std::net::TcpStream;

pub struct Player {
    camera: Camera3D,
    velocity: Vector3,
    body: Cuboid,
    pistol: Pistol,
    health: u8,
}

impl Default for Player {
    fn default() -> Self {
        let camera_pos = Vector3::new(0.0, Self::CAMERA_HEIGHT, 0.0);
        let camera =
            Camera3D::perspective(camera_pos, Vector3::forward(), Vector3::up(), Self::FOV);
        let body_pos = Vector3::new(0.0, Self::PLAYER_HEIGHT_HALF, 0.0);
        let body_size = Vector3::new(
            Player::PLAYER_UNIT,
            Player::PLAYER_HEIGHT,
            Player::PLAYER_UNIT,
        );
        let body = Cuboid::new(body_pos, body_size, Color::GREEN);
        let pistol = Pistol::new();

        Player {
            camera,
            velocity: Vector3::zero(),
            body,
            pistol,
            health: 100,
        }
    }
}

impl Player {
    const FOV: f32 = 60.0;
    const CAMERA_HEIGHT: f32 = 3.2;
    const CAMERA_MOUSE_SENSITIVITY: f32 = 0.0015;
    const SPEED: f32 = 90.0;
    const PLAYER_HEIGHT: f32 = 3.5;
    const PLAYER_HEIGHT_HALF: f32 = Self::PLAYER_HEIGHT / 2.0;
    const PLAYER_UNIT: f32 = 1.0;

    pub fn get_camera(&self) -> &Camera3D {
        &self.camera
    }

    pub fn update(&mut self, rl: &RaylibHandle, objects: &Vec<Cuboid>, rays: &mut Option<Ray>) {
        // --------------------------------------------------------------------
        // Player turning
        // Some Logic taken from https://github.com/raysan5/raylib/blob/master/src/rcamera.h
        // --------------------------------------------------------------------

        let mouse_delta = rl.get_mouse_delta();
        let up = self.camera.up;
        let mut forward = self.camera.target - self.camera.position;
        let right = forward.cross(up).normalized();
        let yaw_angle = -mouse_delta.x * Self::CAMERA_MOUSE_SENSITIVITY;
        let mut pitch_angle = -mouse_delta.y * Self::CAMERA_MOUSE_SENSITIVITY;
        let forward_copy = forward.normalized();

        // Rotate forward vector around up axis to rotate camera left/right
        forward.rotate(Quaternion::from_axis_angle(up, yaw_angle));

        // Clamp view up
        let mut max_angle_up = find_angle(up, forward);
        max_angle_up -= 0.001;
        if pitch_angle > max_angle_up {
            pitch_angle = max_angle_up
        };

        // Clamp view down
        let mut max_angle_down = find_angle(-up, forward);
        max_angle_down *= -1.0;
        max_angle_down += 0.001;
        if pitch_angle < max_angle_down {
            pitch_angle = max_angle_down
        }

        // Rotate forward vector around right axis to rotate camera up/down
        forward.rotate(Quaternion::from_axis_angle(right, pitch_angle));

        // Move target relative to position
        self.camera.target = self.camera.position + forward;

        // ----------------------------------------------------------------
        // Player Movement
        // Inspiration from https://gist.github.com/jakubtomsu/9cae5298f86d2b9d2aed48641a1a3dbd
        // ----------------------------------------------------------------

        let dt = rl.get_frame_time();

        // Remove y component from forward to restrict movement to the ground
        forward.y = 0.0;
        forward.normalize();

        if rl.is_key_down(KeyboardKey::KEY_W) {
            self.velocity += forward * dt * Self::SPEED;
        }
        if rl.is_key_down(KeyboardKey::KEY_S) {
            self.velocity -= forward * dt * Self::SPEED;
        }
        if rl.is_key_down(KeyboardKey::KEY_D) {
            self.velocity += right * dt * Self::SPEED;
        }
        if rl.is_key_down(KeyboardKey::KEY_A) {
            self.velocity -= right * dt * Self::SPEED;
        }

        // Damping
        self.velocity *= 0.85;

        // Displacement
        let displacement = self.velocity * dt;

        // Check collision
        let new_displacement = self.displacement_after_collision(displacement, objects);

        // Change camera position
        self.camera.position += new_displacement;
        self.camera.target += new_displacement;

        // Move body to match camera
        self.move_body();

        // ----------------------------------------------------------------
        // Shooting
        // ----------------------------------------------------------------
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            // let mut offset =
            //     forward_copy * Pistol::BARREL_Z_OFFSET + right * -Pistol::BARREL_X_OFFSET;
            // offset.y = Pistol::BARREL_Y_OFFSET;
            rays.replace(Ray {
                // position: self.camera.position + offset,
                position: self.camera.position,
                direction: forward_copy,
            });
        }
    }

    fn displacement_after_collision(
        &self,
        mut displacement: Vector3,
        objects: &Vec<Cuboid>,
    ) -> Vector3 {
        // Check horizontal collision
        let new_x_pos = Vector3::new(
            self.camera.position.x + displacement.x,
            self.camera.position.y,
            self.camera.position.z,
        );

        for obj in objects {
            if Self::get_bounding_box(new_x_pos).check_collision_boxes(obj.get_bounding_box()) {
                displacement.x = 0.0;
                break;
            }
        }

        // Check vertical collision
        let new_z_pos = Vector3::new(
            self.camera.position.x,
            self.camera.position.y,
            self.camera.position.z + displacement.z,
        );

        for obj in objects {
            if Self::get_bounding_box(new_z_pos).check_collision_boxes(obj.get_bounding_box()) {
                displacement.z = 0.0;
                break;
            }
        }

        displacement
    }

    fn get_bounding_box(pos: Vector3) -> BoundingBox {
        let half = Player::PLAYER_UNIT / 2.0;
        let min = Vector3::new(pos.x - half, 0.0, pos.z - half);
        let max = Vector3::new(pos.x + half, Player::PLAYER_HEIGHT, pos.z + half);
        BoundingBox::new(min, max)
    }

    pub fn write_stats(&self, channel: &mut Channel<TcpStream>) -> Result<(), ()> {
        channel
            .send(Packet::Player {
                pos: ChannelVector2::from(self.camera.position),
                target: ChannelVector2::from(self.camera.target),
            })
            .map_err(|_| ())
    }

    pub fn read_stats(&mut self, channel: &mut Channel<TcpStream>) -> Result<(), ()> {
        if let Ok(Packet::Player { pos, target }) = channel.receive() {
            self.camera.position.x = pos.x;
            self.camera.position.z = pos.z;
            self.camera.target.x = target.x;
            self.camera.target.z = target.z;
            self.move_body();
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn write_health(&self, channel: &mut Channel<TcpStream>) -> Result<(), ()> {
        channel.send(Packet::Health(self.health)).map_err(|_| ())
    }

    pub fn read_health(
        &mut self,
        channel: &mut Channel<TcpStream>,
        ouch: &Sound,
    ) -> Result<(), ()> {
        if let Ok(Packet::Health(health)) = channel.receive() {
            if health < self.health {
                ouch.play();
            }
            self.health = health;
            Ok(())
        } else {
            Err(())
        }
    }

    fn move_body(&mut self) {
        self.body.move_to(Vector3::new(
            self.camera.position.x,
            Self::PLAYER_HEIGHT_HALF,
            self.camera.position.z,
        ));
    }

    pub fn get_pos(&self) -> (f32, f32) {
        (self.camera.position.x, self.camera.position.z)
    }

    pub fn draw_gun(&self, d: &mut raylib::prelude::RaylibDrawHandle, camera: &Camera3D) {
        let mut direction = self.camera.target - self.camera.position;
        let forward = Vector3::forward();

        direction.y = 0.0;

        let cross_product = forward.cross(direction);
        let mut angle = f32::atan2(cross_product.length(), forward.dot(direction)) * 180.0
            / std::f32::consts::PI;

        if cross_product.y < 0.0 {
            angle = 360.0 - angle;
        }
        self.pistol
            .draw_target(d, camera, self.camera.position, angle);
    }

    pub fn draw_health_bar(&self, d: &mut raylib::prelude::RaylibDrawHandle) {
        let rect_size = Vector2::new(250.0, 50.0);
        let health_size = Vector2::new(250.0 * self.health as f32 / 100.0, 50.0);
        let health_pos = Vector2::new(20.0, Game::SCREEN_HEIGHT as f32 - 20.0 - 50.0);
        d.draw_rectangle_v(health_pos, rect_size, Color::WHITE);
        d.draw_rectangle_v(health_pos, health_size, Color::LIGHTGREEN);
        d.draw_rectangle_lines(
            health_pos.x as i32,
            health_pos.y as i32,
            rect_size.x as i32,
            rect_size.y as i32,
            Color::BLACK,
        );
    }

    pub fn collision(&self, ray: Ray) -> RayCollision {
        self.body.get_bounding_box().get_ray_collision_box(ray)
    }

    pub fn decrease_health(&mut self) {
        self.health -= 10;
    }
}

pub fn find_angle(vec_1: Vector3, vec_2: Vector3) -> f32 {
    (vec_1.dot(vec_2) / (vec_1.length() * vec_2.length())).acos()
}

impl Drawable3D for Player {
    fn draw(&self, d: &mut raylib::prelude::RaylibDrawHandle, camera: &Camera3D) {
        let mut direction = self.camera.target - self.camera.position;
        let forward = Vector3::forward();

        direction.y = 0.0;

        let cross_product = forward.cross(direction);
        let mut angle = f32::atan2(cross_product.length(), forward.dot(direction)) * 180.0
            / std::f32::consts::PI;

        if cross_product.y < 0.0 {
            angle = 360.0 - angle;
        }
        self.body.draw_target(d, camera, angle);
        self.pistol
            .draw_target(d, camera, self.camera.position, angle);

        // Health
        // Find distance between camera and player
        let distance = (self.camera.position - camera.position).length();
        let rect_size = Vector2::new(1000.0, 200.0) / distance;
        let health_size = Vector2::new(rect_size.x * self.health as f32 / 100.0, rect_size.y);
        let mut health_pos =
            d.get_world_to_screen(self.camera.position + Vector3::new(0.0, 1.0, 0.0), camera);
        health_pos.x -= rect_size.x / 2.0;
        d.draw_rectangle_v(health_pos, rect_size, Color::WHITE);
        d.draw_rectangle_v(health_pos, health_size, Color::LIGHTGREEN);
        d.draw_rectangle_lines(
            health_pos.x as i32,
            health_pos.y as i32,
            rect_size.x as i32,
            rect_size.y as i32,
            Color::BLACK,
        );
    }
}
