use super::{blaze3d::systems::VertexSorter, util::math::ArgbHelper};
use bytes::BytesMut;
use glam::{Mat3, Mat4, Vec3, Vec4};
use glium::{program::OutputPrimitives, vertex::AttributeType, VertexFormat};
use log::debug;
use std::borrow::{Borrow, Cow};

pub type VertexFormatElement = (Cow<'static, str>, usize, i32, AttributeType, bool);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum VertexFormatDrawMode {
    Lines,
    LineStrip,
    DebugLines,
    DebugLineStrip,
    Triangles,
    TriangleStrip,
    TriangleFan,
    Quads,
}

impl VertexFormatDrawMode {
    pub fn get(&self) -> OutputPrimitives {
        (*self).into()
    }

    pub fn first_vertex_count(&self) -> usize {
        match self.get() {
            OutputPrimitives::Points => unreachable!(),
            OutputPrimitives::Lines => 2,
            OutputPrimitives::Triangles => 3,
            OutputPrimitives::Quads => 4,
        }
    }

    pub fn additional_vertex_count(&self) -> usize {
        match self {
            VertexFormatDrawMode::Lines | VertexFormatDrawMode::DebugLines => 2,
            VertexFormatDrawMode::LineStrip
            | VertexFormatDrawMode::DebugLineStrip
            | VertexFormatDrawMode::TriangleStrip
            | VertexFormatDrawMode::TriangleFan => 1,
            VertexFormatDrawMode::Triangles => 3,
            VertexFormatDrawMode::Quads => 4,
        }
    }

    pub fn share_vertices(&self) -> bool {
        matches!(
            self,
            Self::LineStrip | Self::DebugLineStrip | Self::TriangleStrip | Self::TriangleFan
        )
    }

    pub fn index_count(&self, count: usize) -> usize {
        match self {
            VertexFormatDrawMode::LineStrip
            | VertexFormatDrawMode::DebugLines
            | VertexFormatDrawMode::DebugLineStrip
            | VertexFormatDrawMode::Triangles
            | VertexFormatDrawMode::TriangleStrip
            | VertexFormatDrawMode::TriangleFan => count,
            VertexFormatDrawMode::Lines | VertexFormatDrawMode::Quads => count / 4 * 6,
        }
    }
}

impl Into<OutputPrimitives> for VertexFormatDrawMode {
    fn into(self) -> OutputPrimitives {
        match self {
            VertexFormatDrawMode::Lines
            | VertexFormatDrawMode::LineStrip
            | VertexFormatDrawMode::DebugLines
            | VertexFormatDrawMode::DebugLineStrip => OutputPrimitives::Lines,
            VertexFormatDrawMode::Triangles
            | VertexFormatDrawMode::TriangleStrip
            | VertexFormatDrawMode::TriangleFan => OutputPrimitives::Triangles,
            VertexFormatDrawMode::Quads => OutputPrimitives::Quads,
        }
    }
}

pub struct BufferBuilder {
    buffer: BytesMut,
    built_buf_count: usize,
    batch_offset: usize,
    element_offset: usize,
    vertex_count: usize,
    current_element: Option<usize>,
    format: Option<VertexFormat>,
    draw_mode: Option<VertexFormatDrawMode>,
    /// Whether this builder is aware of the vertex format and can skip checks for the current target target element while building a vertex in `vertex`.
    can_skip_element_checks: bool,
    has_overlay: bool,
    building: bool,
    sorting_primitive_centers: Option<Vec<Vec3>>,
    sorter: Option<VertexSorter>,
    has_no_vertex_buffer: bool,
}

impl BufferBuilder {
    pub fn new(init_capacity: usize) -> Self {
        Self {
            buffer: BytesMut::with_capacity(init_capacity * 6),
            built_buf_count: Default::default(),
            batch_offset: Default::default(),
            element_offset: Default::default(),
            vertex_count: Default::default(),
            current_element: Default::default(),
            format: Default::default(),
            draw_mode: Default::default(),
            can_skip_element_checks: Default::default(),
            has_overlay: Default::default(),
            building: Default::default(),
            sorting_primitive_centers: Default::default(),
            sorter: Default::default(),
            has_no_vertex_buffer: Default::default(),
        }
    }

    pub fn get_format(&self) -> &VertexFormat {
        match &self.format {
            Some(e) => e,
            _ => unreachable!(),
        }
    }

    fn grow_default(&mut self) {
        let e: &[(Cow<'static, str>, usize, i32, AttributeType, bool)] = self.get_format().borrow();
        self.grow(e.into_iter().map(|t| t.3.get_size_bytes() as i32).sum())
    }

    fn grow(&mut self, size: i32) {
        if self.element_offset as i32 + size <= self.buffer.capacity() as i32 {
            return;
        }
        let i = self.buffer.capacity();
        let more = Self::round_buffer_size(size) as usize;
        debug!(
            "Needed to grow BufferBuilder buffer: Old size {} bytes, new size {} bytes.",
            i,
            i + more
        );
        self.buffer.reserve(more);
    }

    fn round_buffer_size(amount: i32) -> i32 {
        let j;
        let mut i = 0x200000;
        if amount == 0 {
            return i;
        }
        if amount < 0 {
            i *= -1;
        }
        j = amount % i;
        if j == 0 {
            amount
        } else {
            amount + i - j
        }
    }

    pub fn set_sorter(&mut self, sorter: VertexSorter) {
        if self.draw_mode != Some(VertexFormatDrawMode::Quads) {
            return;
        }
        self.sorter = Some(sorter);
        if self.sorting_primitive_centers.is_none() {}
    }

    fn build_primitive_centers(&self) -> Vec<Vec3> {
        todo!()
    }
}

/// An trait that consumes vertices in a certain [`VertexFormat`].
///
/// The vertex elements must be specified in the same order as defined in the format the vertices being consumed are in.
pub trait VertexConsume {
    fn vertex(&mut self, x: f64, y: f64, z: f64) -> bool;
    fn color(&mut self, red: u32, green: u32, blue: u32, alpha: u32) -> bool;
    fn texture(&mut self, u: f32, v: f32) -> bool;
    fn overlay(&mut self, u: i32, v: i32) -> bool;
    fn light(&mut self, u: i32, v: i32) -> bool;
    fn normal(&mut self, x: f32, y: f32, z: f32) -> bool;
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
    ) -> bool {
        let b = self.vertex(x as f64, y as f64, z as f64)
            && self.color_f32(red, green, blue, alpha)
            && self.texture(u, v)
            && self.overlay_uv(overlay)
            && self.light_uv(light)
            && self.normal(normal_x, normal_y, normal_z);
        self.next();
        b
    }

    fn fixed_color(&mut self, red: u32, green: u32, blue: u32, alpha: u32);
    fn unfix_color(&mut self);

    fn color_f32(&mut self, red: f32, green: f32, blue: f32, alpha: f32) -> bool {
        self.color(
            (red * 255.0) as u32,
            (green * 255.0) as u32,
            (blue * 255.0) as u32,
            (alpha * 255.0) as u32,
        )
    }

    fn color_argb(&mut self, argb: u32) -> bool {
        let h = ArgbHelper(argb);
        self.color(h.red(), h.green(), h.blue(), h.alpha())
    }

    fn light_uv(&mut self, uv: i32) -> bool {
        self.light(
            uv & (LightmapTextureManager::MAX_BLOCK_LIGHT_COORDINATE as i32 | 0xFF0F),
            uv >> 16 & (LightmapTextureManager::MAX_BLOCK_LIGHT_COORDINATE as i32 | 0xFF0F),
        )
    }

    fn overlay_uv(&mut self, uv: i32) -> bool {
        self.overlay(uv & 0xFFFF, uv >> 16 & 0xFFFF)
    }

    fn vertex_with_matrix(&mut self, matrix: Mat4, x: f32, y: f32, z: f32) -> bool {
        let vec4 = matrix * Vec4::new(x, y, z, 1.0);
        self.vertex(vec4.x as f64, vec4.y as f64, vec4.z as f64)
    }

    fn normal_with_matrix(&mut self, matrix: Mat3, x: f32, y: f32, z: f32) -> bool {
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

    fn uv(&mut self, u: i16, v: i16, index: i32) -> bool {
        let e = self.current_element().3;
        if self.current_element().2 != index {
            return true;
        }
        if !matches!(
            e,
            AttributeType::I16 | AttributeType::I16I16 | AttributeType::I16I16I16
        ) || e.get_num_components() != 2
        {
            return false;
        }

        self.put_i16(0, u);
        self.put_i16(2, v);
        true
    }

    // Functions in `super`
    fn super_color(&mut self, red: u32, green: u32, blue: u32, alpha: u32) -> bool {
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

    fn super_texture(&mut self, u: f32, v: f32) -> bool {
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
        ) || e.get_num_components() != 2
        {
            return false;
        }

        self.put_f32(0, u);
        self.put_f32(0, v);
        self.next_element();
        true
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

    fn super_overlay(&mut self, u: i32, v: i32) -> bool {
        self.uv(u as i16, v as i16, 1)
    }

    fn super_light(&mut self, u: i32, v: i32) -> bool {
        self.uv(u as i16, v as i16, 2)
    }

    fn super_normal(&mut self, x: f32, y: f32, z: f32) -> bool {
        let e = self.current_element().3;
        if !matches!(
            e,
            AttributeType::I8
                | AttributeType::I8I8
                | AttributeType::I8I8I8
                | AttributeType::I8I8I8I8
        ) || e.get_num_components() != 3
        {
            return false;
        }
        self.put_u8(0, Self::pack_u8(x));
        self.put_u8(1, Self::pack_u8(y));
        self.put_u8(2, Self::pack_u8(z));
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
