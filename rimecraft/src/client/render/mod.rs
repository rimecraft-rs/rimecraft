use super::util::math::ArgbHelper;
use glam::{Mat3, Mat4, Vec3, Vec4};
use glium::vertex::AttributeType;
use std::borrow::Cow;

pub type VertexFormatElement = (Cow<'static, str>, usize, i32, AttributeType, bool);

/// An trait that consumes vertices in a certain [`VertexFormat`].
///
/// The vertex elements must be specified in the same order as defined in the format the vertices being consumed are in.
pub trait VertexConsume {
    fn vertex(&mut self, x: f64, y: f64, z: f64);
    fn color(&mut self, red: u32, green: u32, blue: u32, alpha: u32);
    fn texture(&mut self, u: f32, v: f32);
    fn overlay(&mut self, u: i32, v: i32);
    fn light(&mut self, u: i32, v: i32);
    fn normal(&mut self, x: f32, y: f32, z: f32);
    fn next(&mut self);

    fn vertex_all(
        &mut self,
        x: f32,
        y: f32,
        z: f32,
        red: f32,
        green: f32,
        blue: f32,
        alpha: f32,
        u: f32,
        v: f32,
        overlay: i32,
        light: i32,
        normal_x: f32,
        normal_y: f32,
        normal_z: f32,
    ) {
        self.vertex(x as f64, y as f64, z as f64);
        self.color_f32(red, green, blue, alpha);
        self.texture(u, v);
        self.overlay_uv(overlay);
        self.light_uv(light);
        self.normal(normal_x, normal_y, normal_z);
        self.next()
    }

    fn fixed_color(&mut self, red: u32, green: u32, blue: u32, alpha: u32);
    fn unfix_color(&mut self);

    fn color_f32(&mut self, red: f32, green: f32, blue: f32, alpha: f32) {
        self.color(
            (red * 255.0) as u32,
            (green * 255.0) as u32,
            (blue * 255.0) as u32,
            (alpha * 255.0) as u32,
        )
    }

    fn color_argb(&mut self, argb: u32) {
        let h = ArgbHelper(argb);
        self.color(h.red(), h.green(), h.blue(), h.alpha())
    }

    fn light_uv(&mut self, uv: i32) {
        self.light(
            uv & (LightmapTextureManager::MAX_BLOCK_LIGHT_COORDINATE as i32 | 0xFF0F),
            uv >> 16 & (LightmapTextureManager::MAX_BLOCK_LIGHT_COORDINATE as i32 | 0xFF0F),
        )
    }

    fn overlay_uv(&mut self, uv: i32) {
        self.overlay(uv & 0xFFFF, uv >> 16 & 0xFFFF)
    }

    fn vertex_with_matrix(&mut self, matrix: Mat4, x: f32, y: f32, z: f32) {
        let vec4 = matrix * Vec4::new(x, y, z, 1.0);
        self.vertex(vec4.x as f64, vec4.y as f64, vec4.z as f64)
    }

    fn normal_with_matrix(&mut self, matrix: Mat3, x: f32, y: f32, z: f32) {
        let vec3 = matrix * Vec3::new(x, y, z);
        self.normal(vec3.x, vec3.y, vec3.z)
    }
}

pub trait BufVertexConsume: VertexConsume {
    fn current_element(&self) -> VertexFormatElement;
    fn next_element(&mut self);

    fn put_u8(&mut self, index: usize, value: u8);
    fn put_i16(&mut self, index: usize, value: i16);
    fn put_f32(&mut self, index: usize, value: f32);

    fn pack_u8(f: f32) -> u8 {
        (((crate::util::math::clamp_f32(f, -1.0, 1.0) * 127.0) as i32) & 0xFF) as u8
    }

    fn super_vertex(&mut self, x: f64, y: f64, z: f64) -> bool {
        let e = self.current_element().3;
        if !matches!(
            e,
            AttributeType::F16
                | AttributeType::F16F16
                | AttributeType::F16F16F16
                | AttributeType::F16F16F16F16
                | AttributeType::F16x2x2
                | AttributeType::F16x2x3
                | AttributeType::F16x2x4
                | AttributeType::F16x3x2
                | AttributeType::F16x3x3
                | AttributeType::F16x3x4
                | AttributeType::F16x4x2
                | AttributeType::F16x4x3
                | AttributeType::F16x4x4
        ) || e.get_num_components() != 3
        {
            return false;
        }

        self.put_f32(0, x as f32);
        self.put_f32(4, y as f32);
        self.put_f32(8, z as f32);
        self.next_element();
        true
    }

    fn color(&mut self, red: u32, green: u32, blue: u32, alpha: u32) -> bool {
        let e = self.current_element().3;
        if !matches!(
            e,
            AttributeType::U8
                | AttributeType::U8U8
                | AttributeType::U8U8U8
                | AttributeType::U8U8U8U8
        ) || e.get_num_components() != 4
        {
            return false;
        }

        self.put_u8(0, red as u8);
        self.put_u8(1, green as u8);
        self.put_u8(2, blue as u8);
        self.put_u8(3, alpha as u8);
        self.next_element();
        true
    }
}

pub struct FixedColorVertexConsumer {
    pub color_fixed: bool,
    pub fixed_red: u32,
    pub fixed_green: u32,
    pub fixed_blue: u32,
    pub fixed_alpha: u32,
}

impl FixedColorVertexConsumer {
    pub fn fixed_color(&mut self, red: u32, green: u32, blue: u32, alpha: u32) {
        self.fixed_red = red;
        self.fixed_green = green;
        self.fixed_blue = blue;
        self.fixed_alpha = alpha;
        self.color_fixed = true;
    }

    pub fn unfix_color(&mut self) {
        self.color_fixed = false;
    }
}

pub struct LightmapTextureManager;

impl LightmapTextureManager {
    pub const MAX_BLOCK_LIGHT_COORDINATE: u32 = 240;
}
