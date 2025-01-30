use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::drawing::{RaylibDraw3D, RaylibDrawHandle, RaylibMode3DExt};
use raylib::ffi::{rlPopMatrix, rlPushMatrix, rlRotatef, rlTranslatef};
use raylib::math::{BoundingBox, Vector2, Vector3};

use crate::player::find_angle;

// ----------------------------------------------------------------------------
// Traits for 3D objecs
// ----------------------------------------------------------------------------

pub trait Drawable3D {
    fn draw(&self, d: &mut RaylibDrawHandle, camera: &Camera3D);
}

pub trait Movable {
    fn move_to(&mut self, new_pos: Vector3);
}

// ----------------------------------------------------------------------------
// 3D objects
// ----------------------------------------------------------------------------

pub struct Plane {
    center_pos: Vector3,
    size: Vector2,
    color: Color,
}

impl Plane {
    pub fn new(pos: Vector3, size: Vector2, color: Color) -> Self {
        Plane {
            center_pos: pos,
            size,
            color,
        }
    }
}

impl Drawable3D for Plane {
    fn draw(&self, d: &mut RaylibDrawHandle, camera: &Camera3D) {
        let mut d = d.begin_mode3D(camera);
        d.draw_plane(self.center_pos, self.size, self.color);
    }
}

pub struct Cuboid {
    pos: Vector3,
    size: Vector3,
    color: Color,
}

impl Cuboid {
    pub fn new(pos: Vector3, size: Vector3, color: Color) -> Self {
        Cuboid { pos, size, color }
    }

    pub fn get_bounding_box(&self) -> BoundingBox {
        let half_length = self.size.z / 2.0;
        let half_width = self.size.x / 2.0;
        let half_height = self.size.y / 2.0;

        let min = Vector3::new(
            self.pos.x - half_width,
            self.pos.y - half_height,
            self.pos.z - half_length,
        );
        let max = Vector3::new(
            self.pos.x + half_width,
            self.pos.y + half_height,
            self.pos.z + half_length,
        );
        BoundingBox::new(min, max)
    }

    pub fn draw_target(&self, d: &mut RaylibDrawHandle, camera: &Camera3D, target: Vector3) {
        let mut direction = target - self.pos;
        let forward = Vector3::forward();

        direction.y = 0.0;

        let cross_product = forward.cross(direction);
        let mut angle = f32::atan2(cross_product.length(), forward.dot(direction)) * 180.0
            / std::f32::consts::PI;

        if cross_product.y < 0.0 {
            angle = 360.0 - angle;
        }

        let mut d = d.begin_mode3D(camera);
        unsafe {
            rlPushMatrix();
            rlTranslatef(self.pos.x, self.pos.y, self.pos.z);
            rlPushMatrix();
            rlRotatef(angle, 0.0, 1.0, 0.0);
            d.draw_cube_v(Vector3::zero(), self.size, self.color);
            d.draw_cube_wires(
                Vector3::zero(),
                self.size.x,
                self.size.y,
                self.size.z,
                Color::BLACK,
            );
            rlPopMatrix();
            rlPopMatrix();
        }
    }
}

impl Drawable3D for Cuboid {
    fn draw(&self, d: &mut RaylibDrawHandle, camera: &Camera3D) {
        let mut d = d.begin_mode3D(camera);
        d.draw_cube_v(self.pos, self.size, self.color);
        d.draw_cube_wires(
            self.pos,
            self.size.x,
            self.size.y,
            self.size.z,
            Color::BLACK,
        );
    }
}

impl Movable for Cuboid {
    fn move_to(&mut self, new_pos: Vector3) {
        self.pos = new_pos;
    }
}
