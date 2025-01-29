use crate::object::{Cuboid, Drawable3D, Movable};
use game_channel::{Channel, Packet};
use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::ffi::KeyboardKey;
use raylib::math::BoundingBox;
use raylib::math::{Quaternion, Vector3};
use raylib::RaylibHandle;
use std::net::TcpStream;

pub struct Player {
    camera: Camera3D,
    velocity: Vector3,
    body: Cuboid,
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

        Player {
            camera,
            velocity: Vector3::zero(),
            body,
        }
    }
}

impl Player {
    const FOV: f32 = 60.0;
    const CAMERA_HEIGHT: f32 = 3.2;
    const CAMERA_MOUSE_SENSITIVITY: f32 = 0.0045;
    const SPEED: f32 = 90.0;
    const PLAYER_HEIGHT: f32 = 3.5;
    const PLAYER_HEIGHT_HALF: f32 = Self::PLAYER_HEIGHT / 2.0;
    const PLAYER_UNIT: f32 = 1.0;

    pub fn get_camera(&self) -> &Camera3D {
        &self.camera
    }

    pub fn update(&mut self, rl: &RaylibHandle, objects: &Vec<Cuboid>) {
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

    pub fn write_pos(&self, channel: &mut Channel<TcpStream>) -> Result<(), ()> {
        channel
            .send(Packet::PlayerPos {
                x: self.camera.position.x,
                z: self.camera.position.z,
            })
            .map_err(|_| ())
    }

    pub fn read_pos(&mut self, channel: &mut Channel<TcpStream>) -> Result<(), ()> {
        if let Ok(Packet::PlayerPos { x, z }) = channel.receive() {
            self.camera.position.x = x;
            self.camera.position.z = z;
            self.move_body();
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
}

fn find_angle(vec_1: Vector3, vec_2: Vector3) -> f32 {
    (vec_1.dot(vec_2) / (vec_1.length() * vec_2.length())).acos()
}

impl Drawable3D for Player {
    fn draw(&self, d: &mut raylib::prelude::RaylibDrawHandle, camera: &Camera3D) {
        self.body.draw(d, camera);
    }
}
