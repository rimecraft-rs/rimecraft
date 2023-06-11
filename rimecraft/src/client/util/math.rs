use glam::{Mat3, Mat4, Quat, Vec2, Vec3};
use std::{collections::VecDeque, ops::Add};

use crate::util::math::lerp_f32_u32;

/// A stack of transformation matrices used to specify how 3D objects are [`Self::translate()`] translated,
/// [`Self::scale()`] scaled or [`Self::multiply()`] rotated in 3D space.
/// Each entry consists of a position matrix and its corresponding normal matrix.
///
/// By putting matrices in a stack, a transformation can be expressed relative to another.
/// You can [`Self::push()`], transform, render and [`Self::pop()`] pop, which allows you to
/// restore the original matrix after rendering.
///
/// An entry of identity matrix is pushed when a stack is created.
/// This means that a stack is [`Self::is_empty()`] if and only if the stack contains exactly one entry.
pub struct MatrixStack {
    stack: VecDeque<(Mat4, Mat3)>,
}

impl MatrixStack {
    pub fn new() -> Self {
        Self {
            stack: {
                let mut deque = VecDeque::new();
                deque.push_back((Mat4::default(), Mat3::default()));
                deque
            },
        }
    }

    /// Applies the translation transformation to the top entry.
    pub fn translate(&mut self, x: f32, y: f32, z: f32) {
        let entry = self.stack.back_mut().unwrap();
        entry.0 *= Mat4::from_translation(Vec3::new(x, y, z))
    }

    /// Applies the scale transformation to the top entry.
    ///
    /// *This does not scale the normal matrix correctly when the scaling is uniform and the scaling factor is negative.*
    pub fn scale(&mut self, x: f32, y: f32, z: f32) {
        let entry = self.stack.back_mut().unwrap();
        entry.0 *= Mat4::from_scale(Vec3::new(x, y, z));
        if x == y && y == z {
            if x > 0.0 {
                return;
            }
            entry.1 *= Mat3::from_scale(Vec2::new(-1.0, -1.0))
        }
        let f = 1.0 / x;
        let g = 1.0 / y;
        let h = 1.0 / z;
        let i = (f * g * h).cbrt();
        entry.1 *= Mat3::from_mat4(Mat4::from_scale(Vec3::new(i * f, i * g, i * h)))
    }

    /// Applies the rotation transformation to the top entry.
    pub fn multiply(&mut self, quat: Quat) {
        let entry = self.stack.back_mut().unwrap();
        entry.0 *= Mat4::from_quat(quat);
        entry.1 *= Mat3::from_quat(quat);
    }

    /// See [`Self::multiply()`].
    pub fn multiply_translated(&mut self, quat: Quat, origin_x: f32, origin_y: f32, origin_z: f32) {
        let entry = self.stack.back_mut().unwrap();
        entry.0 *= Mat4::from_rotation_translation(quat, Vec3::new(origin_x, origin_y, origin_z));
        entry.1 *= Mat3::from_quat(quat);
    }

    /// Pushes a copy of the top entry onto this stack.
    pub fn push(&mut self) {
        self.stack.push_back(*self.stack.back().unwrap());
    }

    /// Removes the entry at the top of this stack.
    pub fn pop(&mut self) {
        if !self.is_empty() {
            self.stack.pop_back();
        }
    }

    /// The entry at the top of this stack
    pub fn peek(&mut self) -> (Mat4, Mat3) {
        *self.stack.back().unwrap()
    }

    /// Whether this stack contains exactly one entry
    pub fn is_empty(&self) -> bool {
        self.stack.len() <= 1
    }

    /// Sets the top entry to be the identity matrix.
    pub fn load_identity(&mut self) {
        let mut entry = self.stack.back_mut().unwrap();
        entry.0 = Mat4::IDENTITY;
        entry.1 = Mat3::IDENTITY;
    }

    /// Multiplies the top position matrix with the given matrix.
    ///
    /// This does not update the normal matrix unlike other transformation methods.
    pub fn multiply_position_matrix(&mut self, matrix: Mat4) {
        self.stack.back_mut().unwrap().0 *= matrix
    }
}

impl Default for MatrixStack {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy)]
pub struct AbgrHelper(pub u32);

#[derive(Clone, Copy)]
pub struct ArgbHelper(pub u32);

impl AbgrHelper {
    pub fn alpha(&self) -> u32 {
        self.0 >> 24
    }

    pub fn red(&self) -> u32 {
        self.0 & 0xFF
    }

    pub fn blue(&self) -> u32 {
        self.0 >> 16 & 0xFF
    }

    pub fn bgr(&self) -> u32 {
        self.0 & 0xFFFFFF
    }

    pub fn opaque(&self) -> u32 {
        self.0 | 0xFF000000
    }

    pub fn abgr(a: u32, b: u32, g: u32, r: u32) -> u32 {
        a << 24 | b << 16 | g << 8 | r
    }

    pub fn abgr_from_bgr_with_alpha(alpha: u32, bgr: u32) -> u32 {
        alpha << 24 | bgr & 0xFFFFFF
    }
}

impl ArgbHelper {
    pub fn alpha(&self) -> u32 {
        self.0 >> 24
    }

    pub fn red(&self) -> u32 {
        self.0 >> 16 & 0xFF
    }

    pub fn green(&self) -> u32 {
        self.0 >> 8 & 0xFF
    }

    pub fn blue(&self) -> u32 {
        self.0 & 0xFF
    }

    pub fn argb(alpha: u32, red: u32, green: u32, blue: u32) -> u32 {
        alpha << 24 | red << 16 | green << 8 | blue
    }

    pub fn lerp(delta: f32, start: u32, end: u32) -> u32 {
        let s = Self(start);
        let e = Self(end);
        Self::argb(
            lerp_f32_u32(delta, s.alpha(), e.alpha()),
            lerp_f32_u32(delta, s.red(), e.red()),
            lerp_f32_u32(delta, s.green(), e.green()),
            lerp_f32_u32(delta, s.blue(), e.blue()),
        )
    }
}

impl Add<u32> for ArgbHelper {
    type Output = u32;

    fn add(self, rhs: u32) -> Self::Output {
        (self + Self(rhs)).0
    }
}

impl Add for ArgbHelper {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(Self::argb(
            self.alpha() * rhs.alpha() / 255,
            self.red() * rhs.red() / 255,
            self.green() * rhs.green() / 255,
            self.blue() * rhs.blue() / 255,
        ))
    }
}
