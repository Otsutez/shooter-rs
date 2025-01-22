use crate::game::Game;
use raylib::{
    color::Color,
    drawing::{RaylibDraw, RaylibDrawHandle},
    ffi::MouseButton,
    math::Rectangle,
    RaylibHandle,
};

pub struct Button {
    rect: Rectangle,
    text: String,
    state: Option<Box<dyn ButtonState>>,
    clicked: bool,
}

impl Button {
    pub const WIDTH: i32 = 150;
    pub const HEIGHT: i32 = 50;
    pub const ROUNDNESS: f32 = 0.4;
    pub const SEGMENTS: i32 = 20;
    pub const SPACING: i32 = 20;
    pub const LINE_THICKNESS: f32 = 5.0;

    pub fn new(rect: Rectangle, text: String) -> Self {
        Button {
            rect,
            text,
            state: Some(Box::new(Idle {})),
            clicked: false,
        }
    }

    pub fn update(&mut self, rl: &RaylibHandle) {
        if let Some(s) = self.state.take() {
            self.state = Some(s.update(rl, self));
        }
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        self.state.as_ref().unwrap().draw(d, self);
    }

    pub fn is_clicked(&self) -> bool {
        self.clicked
    }

    pub fn toggle_clicked(&mut self) {
        self.clicked = !self.clicked;
    }
}

struct Idle;
struct Hover;
struct MidClick;
struct Clicked;

trait ButtonState {
    fn update(self: Box<Self>, rl: &RaylibHandle, button: &mut Button) -> Box<dyn ButtonState>;
    fn draw(&self, d: &mut RaylibDrawHandle, button: &Button);
}

impl ButtonState for Idle {
    fn update(self: Box<Self>, rl: &RaylibHandle, button: &mut Button) -> Box<dyn ButtonState> {
        let mouse_point = rl.get_mouse_position();

        if button.rect.check_collision_point_rec(mouse_point) {
            Box::new(Hover)
        } else {
            Box::new(Idle)
        }
    }

    fn draw(&self, d: &mut RaylibDrawHandle, button: &Button) {
        d.draw_rectangle_rounded(
            button.rect,
            Button::ROUNDNESS,
            Button::SEGMENTS,
            Color::GOLD.fade(0.6),
        );

        d.draw_rectangle_rounded_lines(
            button.rect,
            Button::ROUNDNESS,
            Button::SEGMENTS,
            Button::LINE_THICKNESS,
            Color::GOLD,
        );

        let text_size = d.measure_text(&button.text, Game::FONT_SIZE);
        let text_x = button.rect.x as i32 + button.rect.width as i32 / 2 - text_size / 2;
        let text_y = button.rect.y as i32 + button.rect.height as i32 / 2 - Game::FONT_SIZE / 2;
        d.draw_text(&button.text, text_x, text_y, 20, Color::BLACK);
    }
}

impl ButtonState for Hover {
    fn update(self: Box<Self>, rl: &RaylibHandle, button: &mut Button) -> Box<dyn ButtonState> {
        let mouse_point = rl.get_mouse_position();

        if button.rect.check_collision_point_rec(mouse_point) {
            if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
                Box::new(MidClick)
            } else {
                Box::new(Hover)
            }
        } else {
            Box::new(Idle)
        }
    }

    fn draw(&self, d: &mut RaylibDrawHandle, button: &Button) {
        d.draw_rectangle_rounded(
            button.rect,
            Button::ROUNDNESS,
            Button::SEGMENTS,
            Color::YELLOW.fade(0.6),
        );

        d.draw_rectangle_rounded_lines(
            button.rect,
            Button::ROUNDNESS,
            Button::SEGMENTS,
            Button::LINE_THICKNESS,
            Color::YELLOW,
        );

        let text_size = d.measure_text(&button.text, 20);
        let text_x = button.rect.x as i32 + button.rect.width as i32 / 2 - text_size / 2;
        let text_y = button.rect.y as i32 + button.rect.height as i32 / 2 - 10;
        d.draw_text(&button.text, text_x, text_y, 20, Color::BLACK);
    }
}

impl ButtonState for MidClick {
    fn update(self: Box<Self>, rl: &RaylibHandle, button: &mut Button) -> Box<dyn ButtonState> {
        let mouse_point = rl.get_mouse_position();

        if rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
            if button.rect.check_collision_point_rec(mouse_point) {
                button.clicked = true;
                Box::new(Hover)
            } else {
                Box::new(Idle)
            }
        } else {
            Box::new(MidClick)
        }
    }

    fn draw(&self, d: &mut RaylibDrawHandle, button: &Button) {
        d.draw_rectangle_rounded(
            button.rect,
            Button::ROUNDNESS,
            Button::SEGMENTS,
            Color::ORANGE.fade(0.6),
        );

        d.draw_rectangle_rounded_lines(
            button.rect,
            Button::ROUNDNESS,
            Button::SEGMENTS,
            Button::LINE_THICKNESS,
            Color::ORANGE,
        );

        let text_size = d.measure_text(&button.text, 20);
        let text_x = button.rect.x as i32 + button.rect.width as i32 / 2 - text_size / 2;
        let text_y = button.rect.y as i32 + button.rect.height as i32 / 2 - 10;
        d.draw_text(&button.text, text_x, text_y, 20, Color::BLACK);
    }
}
