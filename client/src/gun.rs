use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::drawing::RaylibMode3DExt;
use raylib::ffi::{rlPopMatrix, rlPushMatrix, rlRotatef, rlTranslatef};
use raylib::math::{Ray, Vector3};
use raylib::prelude::RaylibDraw3D;
use raylib::prelude::RaylibDrawHandle;

pub struct Pistol {
    barrel_size: Vector3,
    grip_size: Vector3,
    color: Color,
}

impl Pistol {
    pub const BARREL_X_OFFSET: f32 = -0.6;
    pub const BARREL_Y_OFFSET: f32 = -0.7;
    pub const BARREL_Z_OFFSET: f32 = 0.9;
    const BARREL_WIDTH: f32 = 0.2;
    const BARREL_HEIGHT: f32 = 0.2;
    const BARREL_LENGTH: f32 = 0.8;

    const GRIP_X_OFFSET: f32 = 0.0;
    const GRIP_Y_OFFSET: f32 = -0.3;
    const GRIP_Z_OFFSET: f32 = -0.3;
    const GRIP_WIDTH: f32 = 0.2;
    const GRIP_HEIGHT: f32 = 0.4;
    const GRIP_LENGTH: f32 = 0.2;

    pub fn new() -> Self {
        let barrel_size =
            Vector3::new(Self::BARREL_WIDTH, Self::BARREL_HEIGHT, Self::BARREL_LENGTH);
        let grip_size = Vector3::new(Self::GRIP_WIDTH, Self::GRIP_HEIGHT, Self::GRIP_LENGTH);
        Pistol {
            barrel_size,
            grip_size,
            color: Color::DARKSLATEBLUE,
        }
    }

    pub fn draw_target(
        &self,
        d: &mut RaylibDrawHandle,
        camera: &Camera3D,
        pos: Vector3,
        angle: f32,
    ) {
        let mut d = d.begin_mode3D(camera);
        unsafe {
            rlPushMatrix();
            // Translate to player position
            rlTranslatef(pos.x, pos.y, pos.z);
            // Rotate around player
            rlRotatef(angle, 0.0, 1.0, 0.0);

            // Offset to barrel position
            rlTranslatef(
                Self::BARREL_X_OFFSET,
                Self::BARREL_Y_OFFSET,
                Self::BARREL_Z_OFFSET,
            );
            d.draw_cube_v(Vector3::zero(), self.barrel_size, self.color);
            d.draw_cube_wires(
                Vector3::zero(),
                self.barrel_size.x,
                self.barrel_size.y,
                self.barrel_size.z,
                Color::BLACK,
            );

            // Offset to grip position
            rlTranslatef(
                Self::GRIP_X_OFFSET,
                Self::GRIP_Y_OFFSET,
                Self::GRIP_Z_OFFSET,
            );
            d.draw_cube_v(Vector3::zero(), self.grip_size, self.color);
            d.draw_cube_wires(
                Vector3::zero(),
                self.grip_size.x,
                self.grip_size.y,
                self.grip_size.z,
                Color::BLACK,
            );
            rlPopMatrix();
        }
    }
}
