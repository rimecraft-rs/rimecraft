use std::{cmp, f32::NAN};

pub fn f32_max(a: f32, b: f32) -> f32 {
	if a == NAN && b == NAN {
		NAN
	} else if a == NAN {
		b
	} else if b == NAN {
		a
	} else {
		if a > b { a } else { b }
	}
}

pub fn f32_min(a: f32, b: f32) -> f32 {
	if a == NAN && b == NAN {
		NAN
	} else if a == NAN {
		b
	} else if b == NAN {
		a
	} else {
		if a < b { a } else { b }
	}
}

pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
	f32_max(min, f32_min(max, value))
}

pub fn _lerp(factor: f32, start: f32, end: f32) -> f32 {
	start + (end - start) * factor
}

pub fn lerp(factor: f32, start: f32, end: f32, clamps: bool) -> f32 {
	let lerp = _lerp(factor, start, end);
	if clamps { clamp(lerp, start, end) } else { lerp }
}

pub fn _get_lerp_factor(value: f32, start: f32, end: f32) -> f32 {
	(value - start) / (end - start)
}

pub fn get_lerp_factor(value: f32, start: f32, end: f32, clamps: bool) -> f32 {
	let factor = _get_lerp_factor(value, start, end);
	if clamps { clamp(factor, 0.0, 1.0) } else { factor }
}

pub fn _map(value: f32, old_start: f32, old_end: f32, new_start: f32, new_end: f32) -> f32 {
	_lerp(_get_lerp_factor(value, old_start, old_end), new_start, new_end)
}

pub fn map(value: f32, old_start: f32, old_end: f32, new_start: f32, new_end: f32, clamps: bool) -> f32 {
	lerp(get_lerp_factor(value, old_start, old_end, clamps), new_start, new_end, clamps)
}
