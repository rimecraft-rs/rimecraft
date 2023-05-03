use glam::{Mat3, Mat4, Quat, Vec3, Vec4};
use std::collections::VecDeque;

pub fn translate_mat4(mat: Mat4, x: f32, y: f32, z: f32) -> Mat4 {
    Mat4::from_cols(
        mat.x_axis,
        mat.y_axis,
        mat.z_axis,
        Vec4::new(x, y, z, mat.w_axis.w),
    )
}

pub fn scale_mat3(mat: Mat3, x: f32, y: f32, z: f32) -> Mat3 {
    Mat3::from_cols(
        Vec3::new(x, mat.x_axis.y, mat.x_axis.z),
        Vec3::new(mat.y_axis.x, y, mat.y_axis.z),
        Vec3::new(mat.z_axis.x, mat.z_axis.y, z),
    )
}

pub fn scale_mat4(mat: Mat4, x: f32, y: f32, z: f32) -> Mat4 {
    Mat4::from_cols(
        Vec4::new(x, mat.x_axis.y, mat.x_axis.z, mat.x_axis.w),
        Vec4::new(mat.y_axis.x, y, mat.y_axis.z, mat.y_axis.w),
        Vec4::new(mat.z_axis.x, mat.z_axis.y, z, mat.z_axis.w),
        mat.w_axis,
    )
}

fn quat_to_axes(mat3: Mat3, rotation: Quat) -> (Vec3, Vec3, Vec3) {
    let (x, y, z, w) = rotation.into();
    let x2 = x + x;
    let y2 = y + y;
    let z2 = z + z;
    let xx = x * x2;
    let xy = x * y2;
    let xz = x * z2;
    let yy = y * y2;
    let yz = y * z2;
    let zz = z * z2;
    let wx = w * x2;
    let wy = w * y2;
    let wz = w * z2;

    let x_axis = Vec3::new(mat3.x_axis.x - (yy + zz), xy + wz, xz - wy);
    let y_axis = Vec3::new(xy - wz, mat3.y_axis.y - (xx + zz), yz + wx);
    let z_axis = Vec3::new(xz + wy, yz - wx, mat3.z_axis.z - (xx + yy));
    (x_axis, y_axis, z_axis)
}

pub fn rotate_mat3(mat3: Mat3, quat: Quat) -> Mat3 {
    let (a, b, c) = quat_to_axes(mat3, quat);
    Mat3::from_cols(a, b, c)
}

pub fn rotate_mat4(mat4: Mat4, quat: Quat) -> Mat4 {
    let (x, y, z) = quat_to_axes(Mat3::from_mat4(mat4), quat);
    Mat4::from_cols(
        Vec4::new(x.x, x.y, x.z, mat4.x_axis.w),
        Vec4::new(y.x, y.y, y.z, mat4.y_axis.w),
        Vec4::new(z.x, z.y, z.z, mat4.z_axis.w),
        mat4.w_axis,
    )
}

pub fn rotate_mat4_translated(mat4: Mat4, quat: Quat, x: f32, y: f32, z: f32) -> Mat4 {
    let (xr, yr, zr) = quat_to_axes(Mat3::from_mat4(mat4), quat);
    Mat4::from_cols(
        Vec4::new(xr.x, xr.y, xr.z, mat4.x_axis.w),
        Vec4::new(yr.x, yr.y, yr.z, mat4.y_axis.w),
        Vec4::new(zr.x, zr.y, zr.z, mat4.z_axis.w),
        Vec4::new(x, y, z, mat4.w_axis.w),
    )
}

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
        let mut entry = self.stack.back_mut().unwrap();
        entry.0 = translate_mat4(entry.0, x, y, z);
    }

    pub fn scale(&mut self, x: f32, y: f32, z: f32) {
        let mut entry = self.stack.back_mut().unwrap();
        entry.0 = scale_mat4(entry.0, x, y, z);
        if x == y && y == z {
            if x > 0.0 {
                return;
            }
            entry.1 = scale_mat3(entry.1, -1.0, -1.0, -1.0)
        }
        let f = 1.0 / x;
        let g = 1.0 / y;
        let h = 1.0 / z;
        let i = (f * g * h).cbrt();
        entry.1 = scale_mat3(entry.1, i * f, i * g, i * h)
    }

    pub fn multiply(&mut self, quat: Quat) {
        let mut entry = self.stack.back_mut().unwrap();
        entry.0 = rotate_mat4(entry.0, quat);
        entry.1 = rotate_mat3(entry.1, quat);
    }

    pub fn multiply_translated(&mut self, quat: Quat, origin_x: f32, origin_y: f32, origin_z: f32) {
        let mut entry = self.stack.back_mut().unwrap();
        entry.0 = rotate_mat4_translated(entry.0, quat, origin_x, origin_y, origin_z);
        entry.1 = rotate_mat3(entry.1, quat);
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
