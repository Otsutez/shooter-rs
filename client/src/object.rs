use raylib::camera::Camera3D;
use raylib::color::Color;
use raylib::drawing::{RaylibDraw3D, RaylibDrawHandle, RaylibMode3DExt};
use raylib::math::{Vector2, Vector3};

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
