use glam::{Vec2, Vec3};
use iced::Color;

#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone, Debug)]
#[repr(C)]
pub struct Vertex3D {
    position: glam::Vec3,
    tex_coords: glam::Vec2,
    normal: glam::Vec3,
    color: [f32; 4],
}

impl Vertex3D {
    const ATTRIBS: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
        //position
        0 => Float32x3,
        //tex_coords
        1 => Float32x2,
        //normal
        2 => Float32x3,
        //color
        3 => Float32x4,
    ];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub fn v3d(
    position: Vec3,
    tex_coords: Vec2,
    normal: Vec3,
    color: Color,
) -> Vertex3D {
    Vertex3D {
        position,
        tex_coords,
        normal,
        color: color.into_linear(),
    }
}
