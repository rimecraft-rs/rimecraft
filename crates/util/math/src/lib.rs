pub fn lerp(factor: f32, start: f32, end: f32) -> f32 {
	start + (end - start) * factor
}

pub fn get_lerp_factor(value: f32, start: f32, end: f32) -> f32 {
	(value - start) / (end - start)
}

pub fn map(value: f32, old_start: f32, old_end: f32, new_start: f32, new_end: f32) -> f32 {
	lerp(get_lerp_factor(value, old_start, old_end), new_start, new_end)
}
