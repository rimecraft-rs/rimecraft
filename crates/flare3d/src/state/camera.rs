//! Camera implementations.

use glam::{Mat4, Quat, Vec3};
use winit::{
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

/// Represents a camera with position, direction, and perspective parameters.
#[derive(Debug, Copy, Clone)]
pub struct Camera {
    /// Camera position.
    pub eye: Vec3,
    /// Camera direction.
    pub direction: Vec3,

    /// Aspect ratio of the viewport.
    pub aspect: f32,
    /// Field of view in the y direction.
    pub fov_y: f32,

    /// Near frustum plane.
    pub z_near: f32,
    /// Far frustum plane.
    pub z_far: f32,
}

impl Camera {
    /// Eular pitch in radians.
    pub fn pitch(&self) -> f32 {
        self.direction.y.atan2(self.direction.x)
    }

    /// Eular yaw in radians.
    pub fn yaw(&self) -> f32 {
        self.direction.z.atan2(self.direction.x)
    }
    /// Eular pitch in degrees.
    pub fn pitch_deg(&self) -> f32 {
        self.pitch().to_degrees()
    }

    /// Eular yaw in degrees.
    pub fn yaw_deg(&self) -> f32 {
        self.yaw().to_degrees()
    }
}

impl Camera {
    /// Builds the view projection matrix for the camera.
    pub fn build_view_projection_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_rh(self.eye, self.eye + self.direction, Vec3::Y);
        let proj = Mat4::perspective_rh(self.fov_y.to_radians(), self.aspect, self.z_near, self.z_far);

        proj * view
    }

    /// Builds the orthographic projection matrix for the camera.
    pub fn build_orthographic_projection_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_rh(self.eye, self.eye + self.direction, Vec3::Y);
        let proj = Mat4::orthographic_rh(
            -self.fov_y * self.z_near,
            self.fov_y * self.z_near,
            -self.fov_y * self.z_near,
            self.fov_y * self.z_near,
            self.z_near,
            self.z_far,
        );

        proj * view
    }
}

/// Represents a camera uniform buffer object.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    /// Creates a new camera uniform buffer object.
    pub fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    /// Updates the view projection matrix of a [Camera].
    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().to_cols_array_2d()
    }

    /// Updates the orthographic projection matrix of a [Camera].
    pub fn update_orthographic_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_orthographic_projection_matrix().to_cols_array_2d()
    }
}

/// Represents a camera controller.
#[derive(Debug)]
pub struct CameraController {
    speed: f32,

    mv: Vec3,
    mv_lerp: f32,

    forwarding: bool,
    backwarding: bool,
    strafing_left: bool,
    strafing_right: bool,
    flying: bool,
    diving: bool,

    turn_up: bool,
    turn_down: bool,
    turn_left: bool,
    turn_right: bool,
}

impl CameraController {
    /// Creates a new camera controller.
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            forwarding: false,
            backwarding: false,
            strafing_left: false,
            strafing_right: false,
            flying: false,
            diving: false,

            turn_up: false,
            turn_down: false,
            turn_left: false,
            turn_right: false,

            mv_lerp: 0.1,
            mv: Vec3::ZERO,
        }
    }

    /// Processes a window event.
    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(key_code),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match key_code {
                    KeyCode::KeyW | KeyCode::KeyA | KeyCode::KeyS | KeyCode::KeyD => {
                        if is_pressed {
                            match key_code {
                                KeyCode::KeyW => {
                                    self.forwarding = true;
                                }
                                KeyCode::KeyS => {
                                    self.backwarding = true;
                                }
                                KeyCode::KeyA => {
                                    self.strafing_left = true;
                                }
                                KeyCode::KeyD => {
                                    self.strafing_right = true;
                                }
                                _ => unreachable!(),
                            }
                        } else {
                            match key_code {
                                KeyCode::KeyW => {
                                    self.forwarding = false;
                                }
                                KeyCode::KeyS => {
                                    self.backwarding = false;
                                }
                                KeyCode::KeyA => {
                                    self.strafing_left = false;
                                }
                                KeyCode::KeyD => {
                                    self.strafing_right = false;
                                }
                                _ => unreachable!(),
                            }
                        }

                        true
                    }

                    KeyCode::Space => {
                        self.flying = is_pressed;
                        true
                    }
                    KeyCode::ShiftLeft | KeyCode::ShiftRight => {
                        self.diving = is_pressed;
                        true
                    }
                    KeyCode::ArrowUp | KeyCode::KeyK => {
                        self.turn_up = is_pressed;
                        true
                    }
                    KeyCode::ArrowDown | KeyCode::KeyJ => {
                        self.turn_down = is_pressed;
                        true
                    }
                    KeyCode::ArrowLeft | KeyCode::KeyH => {
                        self.turn_left = is_pressed;
                        true
                    }
                    KeyCode::ArrowRight | KeyCode::KeyL => {
                        self.turn_right = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// Updates a [Camera].
    pub fn update_camera(&mut self, camera: &mut Camera) {
        let plane_normal = Vec3::Y;
        let forward = camera.direction - (camera.direction.dot(plane_normal) * plane_normal);
        let forward_normal = forward.normalize();
        let right_normal = forward_normal.cross(plane_normal).normalize();

        // Movement

        let mut mv_target = Vec3::ZERO;

        if self.forwarding {
            mv_target += forward_normal;
        }
        if self.backwarding {
            mv_target -= forward_normal;
        }

        if self.strafing_left {
            mv_target -= right_normal;
        }
        if self.strafing_right {
            mv_target += right_normal;
        }

        if self.flying {
            mv_target += plane_normal;
        }
        if self.diving {
            mv_target -= plane_normal;
        }

        mv_target = if mv_target == Vec3::ZERO { Vec3::ZERO } else { mv_target.normalize() * self.speed };
        self.mv = self.mv.lerp(mv_target, self.mv_lerp);

        camera.eye += self.mv;

        // Rotation

        let rotation = 1.0_f32.to_radians();

        if self.turn_right {
            camera.direction = Quat::from_rotation_y(-rotation) * camera.direction;
        }
        if self.turn_left {
            camera.direction = Quat::from_rotation_y(rotation) * camera.direction;
        }
        if self.turn_up {
            camera.direction = Quat::from_axis_angle(right_normal, rotation) * camera.direction;
        }
        if self.turn_down {
            camera.direction = Quat::from_axis_angle(right_normal, -rotation) * camera.direction;
        }
    }
}
