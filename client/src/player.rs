use raylib::camera::Camera3D;
use raylib::math::{Vector2, Vector3};
use raylib::RaylibHandle;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerPacket {
    x: f32,
    y: f32,
}

impl PlayerPacket {
    pub fn get_pos(&self) -> Vector2 {
        Vector2::new(self.x, self.y)
    }
}

pub struct Player {
    camera: Camera3D,
}

impl Player {
    const FOV: f32 = 60.0;
    const CAMERA_HEIGHT: f32 = 3.2;

    pub fn new(pos: Vector2) -> Self {
        let pos = Vector3::new(pos.x, Self::CAMERA_HEIGHT, pos.y);
        let camera = Camera3D::perspective(pos, Vector3::forward(), Vector3::up(), Self::FOV);
        Player { camera }
    }

    pub fn get_camera(&self) -> &Camera3D {
        &self.camera
    }

    pub fn update(&mut self, rl: &RaylibHandle) {
        rl.update_camera(
            &mut self.camera,
            raylib::ffi::CameraMode::CAMERA_FIRST_PERSON,
        );
    }
}
