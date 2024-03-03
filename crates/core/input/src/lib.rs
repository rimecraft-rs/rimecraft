use glam::Vec2;

pub mod cursor_movement;
pub mod keyboard_input;

pub struct Input<T> {
    pub movement_sideways: f32,
    pub movement_forward: f32,
    pub pressing_forward: bool,
    pub pressing_backward: bool,
    pub pressing_left: bool,
    pub pressing_right: bool,
    pub jumping: bool,
    pub sneaking: bool,
    pub child: T,
}

pub trait SlowDownTickable {
    fn tick(&mut self, slow_down: SlowDown);
}

pub enum SlowDown {
    Yes(f32),
    No,
}

impl<T> SlowDownTickable for Input<T> {
	fn tick(&mut self, _slow_down: SlowDown) {
		// Does nothing
	}
}

impl<T> Input<T> {
    pub fn new(child: T) -> Input<T> {
        Self {
            movement_sideways: 0.0,
            movement_forward: 0.0,
            pressing_forward: false,
            pressing_backward: false,
            pressing_left: false,
            pressing_right: false,
            jumping: false,
            sneaking: false,
            child
        }
    }
}

impl<T> Input<T> {
    pub fn get_movement_input(&self) -> Vec2 {
        Vec2 {
            x: self.movement_sideways,
            y: self.movement_forward,
        }
    }

    pub fn has_movement_forward(&self) -> bool {
        self.movement_forward > 1.0e-5
    }
}
