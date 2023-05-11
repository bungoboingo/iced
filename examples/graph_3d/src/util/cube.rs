use glam::{vec2, Vec3, vec3};

pub const VERTICES: [Vertex; 8] = [
    //front
    Vertex {
        position: vec3(-0.5, -0.5, -0.5),
        color: vec3(1.0, 0.0, 0.0),
    },
    Vertex {
        position: vec3(0.5, -0.5, -0.5),
        color: vec3(1.0, 0.0, 0.0),
    },
    Vertex {
        position: vec3(0.5, 0.5, -0.5),
        color: vec3(1.0, 0.0, 0.0),
    },
    Vertex {
        position: vec3(-0.5, 0.5, -0.5),
        color: vec3(1.0, 0.0, 0.0),
    },
    //back
    Vertex {
        position: vec3(-0.5, -0.5, 0.5),
        color: vec3(0.0, 0.0, 1.0),
    },
    Vertex {
        position: vec3(0.5, -0.5, 0.5),
        color: vec3(0.0, 0.0, 1.0),
    },
    Vertex {
        position: vec3(0.5, 0.5, 0.5),
        color: vec3(0.0, 0.0, 1.0),
    },
    Vertex {
        position: vec3(-0.5, 0.5, 0.5),
        color: vec3(0.0, 0.0, 1.0),
    },
];

pub const INDICES: [u16; 36] = [
    //front
    0, 1, 2, 2, 3, 0,
    //left
    4, 0, 3, 3, 7, 4,
    //back
    5, 4, 7, 7, 6, 5,
    //right
    5, 6, 2, 2, 1, 5,
    //bottom
    0, 4, 5, 5, 1, 0,
    //top
    3, 2, 6, 6, 7, 3,
];

#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Vertex {
    pub position: Vec3,
    pub color: Vec3,
}