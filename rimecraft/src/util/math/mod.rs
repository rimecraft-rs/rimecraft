pub fn lerp_f32_i32(delta: f32, start: i32, end: i32) -> i32 {
    start + (delta * (end - start) as f32).floor() as i32
}

pub fn lerp_f32_u32(delta: f32, start: u32, end: u32) -> u32 {
    start + (delta * (end - start) as f32).floor() as u32
}
