use core::panic;

#[rustfmt::skip]
pub const VERTICES: &[Vertex] = &[
    Vertex { position:  glam::Vec3::new(-0.0868241, 0.49240386, 0.0), tex_coords:  glam::Vec2::new(0.4131759, 0.00759614), }, // A
    Vertex { position:  glam::Vec3::new(-0.49513406, 0.06958647, 0.0), tex_coords:  glam::Vec2::new(0.0048659444, 0.43041354), }, // B
    Vertex { position:  glam::Vec3::new(-0.21918549, -0.44939706, 0.0), tex_coords:  glam::Vec2::new(0.28081453, 0.949397), }, // C
    Vertex { position:  glam::Vec3::new(0.35966998, -0.3473291, 0.0), tex_coords:  glam::Vec2::new(0.85967, 0.84732914), }, // D
    Vertex { position:  glam::Vec3::new(0.44147372, 0.2347359, 0.0), tex_coords:  glam::Vec2::new(0.9414737, 0.2652641), }, // E
];

#[rustfmt::skip]
pub const INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];

pub trait VertexDescription {
    type Data: bytemuck::Pod + bytemuck::Zeroable;
    const ATTRIBS: &'static [wgpu::VertexAttribute];
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: glam::Vec3,
    tex_coords: glam::Vec2,
}

impl VertexDescription for Vertex {
    type Data = Self;
    const ATTRIBS: &'static [wgpu::VertexAttribute] =
        &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self::Data>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: Self::ATTRIBS,
        }
    }
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: glam::Mat4,
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: glam::Mat4::IDENTITY,
        }
    }

    pub fn update_view_proj(&mut self, matrix: glam::Mat4) {
        self.view_proj = matrix;
    }
}

pub trait RawInstanceVector {
    fn to_raw(&self) -> Vec<InstanceRaw>;
}

impl RawInstanceVector for Vec<Instance> {
    fn to_raw(&self) -> Vec<InstanceRaw> {
        self.iter().collect()
    }
}
#[derive(Debug, Clone)]
pub struct Instance {
    position: glam::Vec3,
    rotation: glam::Quat,
}

impl Instance {
    pub fn new(position: glam::Vec3, rotation: glam::Quat) -> Self {
        Self { position, rotation }
    }

    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: glam::Mat4::from_rotation_translation(self.rotation, self.position),
        }
    }
}

impl<'a> FromIterator<&'a Instance> for Vec<InstanceRaw> {
    fn from_iter<T: IntoIterator<Item = &'a Instance>>(iter: T) -> Self {
        iter.into_iter().map(|i| i.to_raw()).collect()
    }
}

impl VertexDescription for Instance {
    type Data = InstanceRaw;
    const ATTRIBS: &'static [wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
        5 => Float32x4, 6 => Float32x4, 7 => Float32x4, 8 => Float32x4
    ];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self::Data>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: glam::Mat4,
}
