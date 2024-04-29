use crate::{
    input::{self, InputEnum, InputMarker},
    render::camera,
};
use nalgebra_glm as glm;
use winit::event::Event;

#[derive(Debug)]
pub struct CameraController {
    speed: f32,
    position: glm::Vec3,
    yaw: cgmath::Rad<f32>,
    pitch: cgmath::Rad<f32>,
}

impl CameraController {
    pub fn new<Z: Into<glm::Vec3>, Y: Into<cgmath::Rad<f32>>, P: Into<cgmath::Rad<f32>>>(
        position: Z,
        yaw: Y,
        pitch: P,
    ) -> Self {
        Self {
            speed: 1.0,
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
        }
    }
    pub fn update(&mut self, input: &input::Input, delta: f64, target: glm::Vec3) {
        let forward = (target - self.position).normalize();
        let mut movement = glm::Vec3::zeros();
        if input.get_bool(input::KeyboardButton::KeyW) {
            movement += forward * self.speed;
        }
        if input.get_bool(input::KeyboardButton::KeyS) {
            movement -= forward * self.speed;
        }
        if input.get_bool(input::KeyboardButton::KeyA) {
            movement -= glm::cross(&forward, &glm::Vec3::y()).normalize() * self.speed;
        }
        if input.get_bool(input::KeyboardButton::KeyD) {
            movement += glm::cross(&forward, &glm::Vec3::y()).normalize() * self.speed;
        }

        self.position += delta as f32 * movement;
    }

    pub fn get_eye(&self) -> glm::Vec3 {
        self.position
    }
}
