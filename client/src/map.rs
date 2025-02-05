use raylib::{
    color::Color,
    math::{Vector2, Vector3},
};
use std::fs::read_to_string;

use crate::object::{Cuboid, Drawable3D, Plane};

pub struct Map {
    plane: Plane,
    pub objects: Vec<Cuboid>,
}

impl Map {
    const WIDTH: f32 = 40.0;
    const LENGTH: f32 = 40.0;
    const UNIT: f32 = 2.0;
    const WALL_HEIGHT: f32 = Map::UNIT * 2.0;
}

impl Default for Map {
    fn default() -> Self {
        // Construct plane
        let plane = Plane::new(
            Vector3::zero(),
            Vector2::new(Map::WIDTH, Map::LENGTH),
            Color::GRAY,
        );

        let mut objects: Vec<Cuboid> = Vec::new();

        // Construct walls
        let wall_offset = Map::LENGTH / 2.0 + Map::UNIT / 2.0;
        objects.push(Cuboid::new(
            Vector3::new(wall_offset, Map::UNIT, 0.0),
            Vector3::new(Map::UNIT, Map::WALL_HEIGHT, Map::WIDTH),
            Color::DARKGRAY,
        ));
        objects.push(Cuboid::new(
            Vector3::new(-wall_offset, Map::UNIT, 0.0),
            Vector3::new(Map::UNIT, Map::WALL_HEIGHT, Map::WIDTH),
            Color::DARKGRAY,
        ));
        objects.push(Cuboid::new(
            Vector3::new(0.0, Map::UNIT, wall_offset),
            Vector3::new(Map::LENGTH, Map::WALL_HEIGHT, Map::UNIT),
            Color::DARKGRAY,
        ));
        objects.push(Cuboid::new(
            Vector3::new(0.0, Map::UNIT, -wall_offset),
            Vector3::new(Map::LENGTH, Map::WALL_HEIGHT, Map::UNIT),
            Color::DARKGRAY,
        ));

        // Construct cuboids
        let maps = read_to_string("./resources/map.txt").unwrap();
        let maps: Vec<Vec<char>> = maps.lines().map(|line| line.chars().collect()).collect();

        let half = Map::WIDTH / 2.0;

        for i in 0..maps.len() {
            for j in 0..maps[i].len() {
                let cube_height = maps[i][j].to_digit(10).unwrap();
                if cube_height != 0 {
                    let size = Vector3::new(Map::UNIT, Map::UNIT * cube_height as f32, Map::UNIT);
                    let pos = Vector3::new(
                        j as f32 * Map::UNIT - half + Map::UNIT / 2.0,
                        size.y / 2.0,
                        i as f32 * Map::UNIT - half + Map::UNIT / 2.0,
                    );
                    objects.push(Cuboid::new(pos, size, Color::RED));
                }
            }
        }

        Map { plane, objects }
    }
}

impl Drawable3D for Map {
    fn draw(&self, d: &mut raylib::prelude::RaylibDrawHandle, camera: &raylib::prelude::Camera3D) {
        self.plane.draw(d, camera);
        self.objects.iter().for_each(|obj| obj.draw(d, camera));
    }
}
