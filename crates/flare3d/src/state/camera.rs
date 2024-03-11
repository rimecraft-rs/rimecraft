use glam::{Mat4, Quat, Vec3};
use winit::{
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

pub struct Camera {
    pub eye: Vec3,
    pub direction: Vec3,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_rh(self.eye, self.eye + self.direction, Vec3::Y);
        let proj = Mat4::perspective_rh(self.fovy.to_radians(), self.aspect, self.znear, self.zfar);

        proj * view
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().to_cols_array_2d();
    }
}

pub struct CameraController {
    speed: f32,

    mv_speed: f32,
    keep_mv_speed: bool,
    mv_acceleration: f32,

    forward: bool,
    backward: bool,
    strafe_left: bool,
    strafe_right: bool,
    fly: bool,
    dive: bool,

    turn_up: bool,
    turn_down: bool,
    turn_left: bool,
    turn_right: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            forward: false,
            backward: false,
            strafe_left: false,
            strafe_right: false,
            fly: false,
            dive: false,
            turn_up: false,
            turn_down: false,
            turn_left: false,
            turn_right: false,
            mv_acceleration: -0.01,
            mv_speed: 0.0,
            keep_mv_speed: false,
        }
    }

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
                            self.keep_mv_speed = true;
                            match key_code {
                                KeyCode::KeyW => {
                                    self.forward = true;
                                    self.backward = false;
                                    self.strafe_left = false;
                                    self.strafe_right = false;
                                }
                                KeyCode::KeyS => {
                                    self.forward = false;
                                    self.backward = true;
                                    self.strafe_left = false;
                                    self.strafe_right = false;
                                }
                                KeyCode::KeyA => {
                                    self.forward = false;
                                    self.backward = false;
                                    self.strafe_left = true;
                                    self.strafe_right = false;
                                }
                                KeyCode::KeyD => {
                                    self.forward = false;
                                    self.backward = false;
                                    self.strafe_left = false;
                                    self.strafe_right = true;
                                }
                                _ => unreachable!(),
                            }
                        } else {
                            self.keep_mv_speed = false
                        }
                        true
                    }

                    KeyCode::Space => {
                        self.fly = is_pressed;
                        true
                    }
                    KeyCode::ShiftLeft | KeyCode::ShiftRight => {
                        self.dive = is_pressed;
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

    pub fn update_camera(&mut self, camera: &mut Camera) {
        if !self.keep_mv_speed {
            self.mv_speed += self.mv_acceleration;
            if self.mv_speed <= 0.0 {
                self.mv_speed = 0.0
            }
        } else {
            self.mv_speed = self.speed
        }

        camera.direction = camera.direction.normalize();

        let plane_normal = Vec3::Y;
        let forward = camera.direction - (camera.direction.dot(plane_normal) * plane_normal);
        let forward_normal = forward.normalize();

        if self.fly {
            camera.eye += plane_normal * self.speed;
        }
        if self.dive {
            camera.eye -= plane_normal * self.speed;
        }

        if self.forward {
            camera.eye += forward_normal * self.mv_speed;
        }
        if self.backward {
            camera.eye -= forward_normal * self.mv_speed;
        }

        let right_normal = forward_normal.cross(plane_normal).normalize();

        if self.strafe_right {
            camera.eye += right_normal * self.mv_speed;
        }
        if self.strafe_left {
            camera.eye -= right_normal * self.mv_speed;
        }

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
