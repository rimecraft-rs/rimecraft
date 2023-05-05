pub fn lerp_f32_i32(delta: f32, start: i32, end: i32) -> i32 {
    start + (delta * (end - start) as f32).floor() as i32
}

pub fn lerp_f32_u32(delta: f32, start: u32, end: u32) -> u32 {
    start + (delta * (end - start) as f32).floor() as u32
}

pub fn clamp_f32(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        min
    } else {
        min_f32(value, max)
    }
}

pub fn min_f32(a: f32, b: f32) -> f32 {
    if a <= b {
        a
    } else {
        b
    }
}
