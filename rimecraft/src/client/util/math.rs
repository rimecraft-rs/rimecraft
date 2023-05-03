use glam::{Mat3, Mat4, Quat, Vec2, Vec3};
use std::collections::VecDeque;

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

    pub fn translate(&mut self, x: f32, y: f32, z: f32) {
        let entry = self.stack.back_mut().unwrap();
        entry.0 *= Mat4::from_translation(Vec3::new(x, y, z))
    }

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

    pub fn multiply(&mut self, quat: Quat) {
        let entry = self.stack.back_mut().unwrap();
        entry.0 *= Mat4::from_quat(quat);
        entry.1 *= Mat3::from_quat(quat);
    }

    pub fn multiply_translated(&mut self, quat: Quat, origin_x: f32, origin_y: f32, origin_z: f32) {
        let entry = self.stack.back_mut().unwrap();
        entry.0 *= Mat4::from_rotation_translation(quat, Vec3::new(origin_x, origin_y, origin_z));
        entry.1 *= Mat3::from_quat(quat);
    }

    pub fn push(&mut self) {
        self.stack.push_back(*self.stack.back().unwrap());
    }

    pub fn pop(&mut self) {
        if !self.is_empty() {
            self.stack.pop_back();
        }
    }

    pub fn peek(&mut self) -> (Mat4, Mat3) {
        *self.stack.back().unwrap()
    }

    pub fn is_empty(&self) -> bool {
        self.stack.len() <= 1
    }

    pub fn load_identity(&mut self) {
        let mut entry = self.stack.back_mut().unwrap();
        entry.0 = Mat4::IDENTITY;
        entry.1 = Mat3::IDENTITY;
    }

    pub fn multiply_position_matrix(&mut self, matrix: Mat4) {
        self.stack.back_mut().unwrap().0 *= matrix
    }
}

impl Default for MatrixStack {
    fn default() -> Self {
        Self::new()
    }
}
