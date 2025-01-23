use raylib::camera::Camera3D;
use raylib::ffi::KeyboardKey;
use raylib::math::{Quaternion, Vector3};
use raylib::RaylibHandle;
use std::io::prelude::*;
use std::net::TcpStream;

pub struct Player {
    camera: Camera3D,
    velocity: Vector3,
}

impl Player {
    const FOV: f32 = 60.0;
    const CAMERA_HEIGHT: f32 = 3.2;
    const CAMERA_MOUSE_SENSITIVITY: f32 = 0.0075;
    const SPEED: f32 = 60.0;

    pub fn get_camera(&self) -> &Camera3D {
        &self.camera
    }

    pub fn update(&mut self, rl: &RaylibHandle) {
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

        // TODO: Check collision

        // Change camera position
        self.camera.position += displacement;
        self.camera.target += displacement;
    }

    pub fn write_pos(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        stream.write(&self.camera.position.x.to_be_bytes())?;
        stream.write(&self.camera.position.z.to_be_bytes())?;
        Ok(())
    }

    pub fn read_pos(&mut self, stream: &mut TcpStream) -> std::io::Result<()> {
        let mut x: [u8; 4] = [0; 4];
        let mut y: [u8; 4] = [0; 4];
        stream.read(&mut x)?;
        stream.read(&mut y)?;
        self.camera.position.x = f32::from_be_bytes(x);
        self.camera.position.z = f32::from_be_bytes(y);
        Ok(())
    }
}

impl Default for Player {
    fn default() -> Self {
        let pos = Vector3::new(0.0, Self::CAMERA_HEIGHT, 0.0);
        let camera = Camera3D::perspective(pos, Vector3::forward(), Vector3::up(), Self::FOV);
        Player {
            camera,
            velocity: Vector3::zero(),
        }
    }
}

fn find_angle(vec_1: Vector3, vec_2: Vector3) -> f32 {
    (vec_1.dot(vec_2) / (vec_1.length() * vec_2.length())).acos()
}
