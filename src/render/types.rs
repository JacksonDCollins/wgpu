use nalgebra as na;

#[rustfmt::skip]
pub const VERTICES: &[Vertex] = &[
    Vertex { position:  na::Vector3::new(-0.0868241, 0.49240386, 0.0), tex_coords:  na::Vector2::new(0.4131759, 0.99240386), }, // A
    Vertex { position:  na::Vector3::new(-0.49513406, 0.06958647, 0.0), tex_coords:  na::Vector2::new(0.0048659444, 0.56958647), }, // B
    Vertex { position:  na::Vector3::new(-0.21918549, -0.44939706, 0.0), tex_coords:  na::Vector2::new(0.28081453, 0.05060294), }, // C
    Vertex { position:  na::Vector3::new(0.35966998, -0.3473291, 0.0), tex_coords:  na::Vector2::new(0.85967, 0.1526709), }, // D
    Vertex { position:  na::Vector3::new(0.44147372, 0.2347359, 0.0), tex_coords:  na::Vector2::new(0.9414737, 0.7347359), }, // E
];

#[rustfmt::skip]
pub const INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: na::Vector3<f32>,
    tex_coords: na::Vector2<f32>,
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

impl Vertex {
    const ATTRIBS: &'static [wgpu::VertexAttribute] =
        &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: Self::ATTRIBS,
        }
    }
}
