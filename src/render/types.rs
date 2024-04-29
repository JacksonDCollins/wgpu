use crate::render::camera;

use cgmath::Zero;
use nalgebra::UnitQuaternion;
use nalgebra_glm as glm;

#[rustfmt::skip]
pub const VERTICES: &[Vertex] = &[
    Vertex { position:  glm::Vec3::new(-0.0868241, 0.49240386, 0.0), tex_coords:  glm::Vec2::new(0.4131759, 0.00759614), }, // A
    Vertex { position:  glm::Vec3::new(-0.49513406, 0.06958647, 0.0), tex_coords:  glm::Vec2::new(0.0048659444, 0.43041354), }, // B
    Vertex { position:  glm::Vec3::new(-0.21918549, -0.44939706, 0.0), tex_coords:  glm::Vec2::new(0.28081453, 0.949397), }, // C
    Vertex { position:  glm::Vec3::new(0.35966998, -0.3473291, 0.0), tex_coords:  glm::Vec2::new(0.85967, 0.84732914), }, // D
    Vertex { position:  glm::Vec3::new(0.44147372, 0.2347359, 0.0), tex_coords:  glm::Vec2::new(0.9414737, 0.2652641), }, // E
];

#[rustfmt::skip]
pub const INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];

pub trait VertexDescription {
    const ATTRIBS: &'static [wgpu::VertexAttribute];
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: glm::Vec3,
    tex_coords: glm::Vec2,
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

impl VertexDescription for Vertex {
    const ATTRIBS: &'static [wgpu::VertexAttribute] =
        &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: Self::ATTRIBS,
        }
    }
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone)]
pub struct CameraUniform {
    view_proj: glm::Mat4,
}
unsafe impl bytemuck::Pod for CameraUniform {}
unsafe impl bytemuck::Zeroable for CameraUniform {}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: glm::Mat4::identity(),
        }
    }

    pub fn update_view_proj(&mut self, matrix: glm::Mat4) {
        self.view_proj = matrix;
    }
}

#[derive(Debug)]
pub struct Instance {
    position: glm::Vec3,
    rotation: glm::Quat,
}

impl Instance {
    pub fn new(position: glm::Vec3, rotation: glm::Quat) -> Self {
        Self { position, rotation }
    }

    pub fn get_data(i: &Self) -> InstanceRaw {
        i.into()
    }
}

impl Into<InstanceRaw> for &Instance {
    fn into(self) -> InstanceRaw {
        let t = cgmath::Matrix4::<f32>::from(cgmath::Quaternion::new(1.0, 2.0, 3.0, 4.0));
        let t2 = glm::quat_to_mat4(&glm::Quat::new(2.0, 3.0, 4.0, 1.0));

        println!("t {:?}", t);
        println!("t2 {:?}", t2);
        panic!();

        InstanceRaw {
            model: (glm::translation(&self.position) * glm::quat_cast(&self.rotation)),
        }
    }
}

impl VertexDescription for Instance {
    const ATTRIBS: &'static [wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
        5 => Float32x4, 6 => Float32x4, 7 => Float32x4, 8 =>Float32x4
    ];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct InstanceRaw {
    model: glm::Mat4,
}

unsafe impl bytemuck::Pod for InstanceRaw {}
unsafe impl bytemuck::Zeroable for InstanceRaw {}
