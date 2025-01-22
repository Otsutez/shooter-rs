/*
Input Box logic is taken from one of raylib's example
https://github.com/raysan5/raylib/blob/master/examples/text/text_input_box.c
*/

use crate::button::Button;
use crate::game::Game;
use raylib::color::Color;
use raylib::drawing::{RaylibDraw, RaylibDrawHandle};
use raylib::math::Rectangle;
use raylib::RaylibHandle;

pub struct InputBox {
    rect: Rectangle,
    text: String,
}

impl InputBox {
    const WIDTH: i32 = 400;
    const HEIGHT: i32 = 50;
    const MAX_INPUT_CHAR: usize = 20;
    const FONT_SIZE: i32 = 40;

    pub fn update(&mut self, rl: &mut RaylibHandle) {
        if self.rect.check_collision_point_rec(rl.get_mouse_position()) {
            rl.set_mouse_cursor(raylib::ffi::MouseCursor::MOUSE_CURSOR_IBEAM);

            while let Some(key) = rl.get_char_pressed() {
                if self.text.len() < Self::MAX_INPUT_CHAR {
                    self.text.push(key);
                }
            }

            if rl.is_key_pressed(raylib::ffi::KeyboardKey::KEY_BACKSPACE) {
                if self.text.len() > 0 {
                    self.text.pop();
                }
            }
        } else {
            rl.set_mouse_cursor(raylib::ffi::MouseCursor::MOUSE_CURSOR_DEFAULT);
        }
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        let mouse_on_text = self.rect.check_collision_point_rec(d.get_mouse_position());
        let mut color = Color::DARKGRAY;

        if mouse_on_text {
            color = Color::RED;
        }

        d.draw_rectangle_rec(self.rect, Color::RAYWHITE);

        d.draw_rectangle_lines(
            self.rect.x as i32,
            self.rect.y as i32,
            self.rect.width as i32,
            self.rect.height as i32,
            color,
        );

        d.draw_text(
            &self.text,
            self.rect.x as i32 + 5,
            self.rect.y as i32 + 8,
            Self::FONT_SIZE,
            Color::MAROON,
        );

        if mouse_on_text {
            d.draw_text(
                "_",
                self.rect.x as i32 + 8 + d.measure_text(&self.text, Self::FONT_SIZE),
                self.rect.y as i32 + 12,
                Self::FONT_SIZE,
                Color::MAROON,
            );
        }
    }

    pub fn get_text(&self) -> &str {
        return &self.text;
    }
}

impl Default for InputBox {
    fn default() -> Self {
        let x = Game::SCREEN_WIDTH / 2 - InputBox::WIDTH / 2;
        let y = Game::SCREEN_HEIGHT / 2 - Button::HEIGHT / 2 - Button::SPACING - InputBox::HEIGHT;
        InputBox {
            rect: Rectangle {
                x: x as f32,
                y: y as f32,
                width: InputBox::WIDTH as f32,
                height: InputBox::HEIGHT as f32,
            },
            text: String::new(),
        }
    }
}
