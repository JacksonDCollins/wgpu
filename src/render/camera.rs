use std::f32::consts::PI;

use crate::{game::camera_controller::CameraController, input, render::types};
#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: glam::Mat4 = glam::Mat4::from_cols_slice(&[
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,]
);

const SAFE_FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2 - 0.0001;

#[derive(Debug)]
pub struct Camera {
    up: glam::Vec3,
    target: glam::Vec3,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
    controller: CameraController,
    uniform: types::CameraUniform,
}

impl Camera {
    pub fn new((screen_width, screen_height): (f32, f32)) -> Self {
        Self {
            // which way is "up"
            up: glam::Vec3::Y,
            target: glam::vec3(0.0, 0.0, 0.0),
            aspect: screen_width / screen_height,
            fovy: PI / 3.0,
            znear: 0.1,
            zfar: 100.0,
            controller: CameraController::new(
                glam::vec3(0.0, 1.0, 2.0),
                cgmath::Deg(-90.0),
                cgmath::Deg(-20.0),
            ),
            uniform: types::CameraUniform::new(),
        }
    }

    pub fn update(&mut self, input: &crate::input::Input, delta: f64) {
        self.controller
            .update(input, delta, self.get_target(), self.get_up());
        self.uniform
            .update_view_proj(self.build_view_projection_matrix());
    }

    pub fn build_view_projection_matrix(&self) -> glam::Mat4 {
        // 1.
        let view = glam::Mat4::look_at_rh(self.controller.get_eye(), self.target, self.up);
        // 2.
        let proj = glam::Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar);

        // 3.
        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }

    pub fn get_target(&self) -> glam::Vec3 {
        self.target
    }

    pub fn set_target(&mut self, target: glam::Vec3) {
        self.target = target;
    }

    pub fn get_up(&self) -> glam::Vec3 {
        self.up
    }

    pub fn get_uniform(&self) -> types::CameraUniform {
        self.uniform
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }
}
