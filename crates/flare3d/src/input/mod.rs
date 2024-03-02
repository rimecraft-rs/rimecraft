use glam::Vec2;

pub mod cursor_movement;

pub struct Input {
	movement_sideways: f32,
	movement_forward: f32,

	pressing_forward: bool,
	pressing_backward: bool,
	pressing_left: bool,
	pressing_right: bool,
	jumping: bool,
	sneaking: bool
}

pub trait SlowDownTickable {
	fn tick(slow_down: SlowDown) {

	}
}

pub enum SlowDown {
	Yes(f32),
	No
}

impl Input {
	pub fn get_movement_input(&self) -> Vec2 {
		Vec2 { x: self.movement_sideways, y: self.movement_forward }
	}

	pub fn has_movement_forward(&self) -> bool {
		self.movement_forward > 1.0e-5
	}
}