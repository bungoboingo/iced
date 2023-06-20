use glam::{vec2, vec3};
use crate::primitive::{Cube, Vertex};

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug)]
#[repr(C)]
pub struct RawCube {
    transformation: glam::Mat4,
    normal: glam::Mat3,
    _padding: [f32; 3],
}

impl RawCube {
    const ATTRIBS: [wgpu::VertexAttribute; 7] = wgpu::vertex_attr_array![
        //transformation mat4
        4 => Float32x4,
        5 => Float32x4,
        6 => Float32x4,
        7 => Float32x4,
        //normal mat3
        8 => Float32x3,
        9 => Float32x3,
        10 => Float32x3,
    ];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

impl RawCube {
    pub fn from_cube(cube: &Cube) -> RawCube {
        RawCube {
            transformation: glam::Mat4::from_scale_rotation_translation(
                glam::vec3(cube.size, cube.size, cube.size),
                cube.rotation,
                cube.position,
            ),
            normal: glam::Mat3::from_quat(cube.rotation),
            _padding: [0.0; 3],
        }
    }

    pub fn vertices() -> [Vertex; 36] {
        let mut v = [
            //face 1
            Vertex {
                pos: vec3(-0.5, -0.5, -0.5),
                normal: vec3(0.0, 0.0, -1.0),
                tangent: Default::default(),
                uv: vec2(0.0, 1.0),
            },
            Vertex {
                pos: vec3(0.5, -0.5, -0.5),
                normal: vec3(0.0, 0.0, -1.0),
                tangent: Default::default(),
                uv: vec2(1.0, 1.0),
            },
            Vertex {
                pos: vec3(0.5, 0.5, -0.5),
                normal: vec3(0.0, 0.0, -1.0),
                tangent: Default::default(),
                uv: vec2(1.0, 0.0),
            },
            Vertex {
                pos: vec3(0.5, 0.5, -0.5),
                normal: vec3(0.0, 0.0, -1.0),
                tangent: Default::default(),
                uv: vec2(1.0, 0.0),
            },
            Vertex {
                pos: vec3(-0.5, 0.5, -0.5),
                normal: vec3(0.0, 0.0, -1.0),
                tangent: Default::default(),
                uv: vec2(0.0, 0.0),
            },
            Vertex {
                pos: vec3(-0.5, -0.5, -0.5),
                normal: vec3(0.0, 0.0, -1.0),
                tangent: Default::default(),
                uv: vec2(0.0, 1.0),
            },
            //face 2
            Vertex {
                pos: vec3(-0.5, -0.5, 0.5),
                normal: vec3(0.0, 0.0, 1.0),
                tangent: Default::default(),
                uv: vec2(0.0, 1.0)
            },
            Vertex {
                pos: vec3(0.5, -0.5, 0.5),
                normal: vec3(0.0, 0.0, 1.0),
                tangent: Default::default(),
                uv: vec2(1.0, 1.0),
            },
            Vertex {
                pos: vec3(0.5, 0.5, 0.5),
                normal: vec3(0.0, 0.0, 1.0),
                tangent: Default::default(),
                uv: vec2(1.0, 0.0),
            },
            Vertex {
                pos: vec3(0.5, 0.5, 0.5),
                normal: vec3(0.0, 0.0, 1.0),
                tangent: Default::default(),
                uv: vec2(1.0, 0.0),
            },
            Vertex {
                pos: vec3(-0.5, 0.5, 0.5),
                normal: vec3(0.0, 0.0, 1.0),
                tangent: Default::default(),
                uv: vec2(0.0, 0.0),
            },
            Vertex {
                pos: vec3(-0.5, -0.5, 0.5),
                normal: vec3(0.0, 0.0, 1.0),
                tangent: Default::default(),
                uv: vec2(0.0, 1.0),
            },
            //face 3
            Vertex {
                pos: vec3(-0.5, 0.5, 0.5),
                normal: vec3(-1.0, 0.0, 0.0),
                tangent: Default::default(),
                uv: vec2(0.0, 1.0),
            },
            Vertex {
                pos: vec3(-0.5, 0.5, -0.5),
                normal: vec3(-1.0, 0.0, 0.0),
                tangent: Default::default(),
                uv: vec2(1.0, 1.0),
            },
            Vertex {
                pos: vec3(-0.5, -0.5, -0.5),
                normal: vec3(-1.0, 0.0, 0.0),
                tangent: Default::default(),
                uv: vec2(1.0, 0.0),
            },
            Vertex {
                pos: vec3(-0.5, -0.5, -0.5),
                normal: vec3(-1.0, 0.0, 0.0),
                tangent: Default::default(),
                uv: vec2(1.0, 0.0),
            },
            Vertex {
                pos: vec3(-0.5, -0.5, 0.5),
                normal: vec3(-1.0, 0.0, 0.0),
                tangent: Default::default(),
                uv: vec2(0.0, 0.0),
            },
            Vertex {
                pos: vec3(-0.5, 0.5, 0.5),
                normal: vec3(-1.0, 0.0, 0.0),
                tangent: Default::default(),
                uv: vec2(0.0, 1.0),
            },
            //face 4
            Vertex {
                pos: vec3(0.5, 0.5, 0.5),
                normal: vec3(1.0, 0.0, 0.0),
                tangent: Default::default(),
                uv: vec2(0.0, 1.0),
            },
            Vertex {
                pos: vec3(0.5, 0.5, -0.5),
                normal: vec3(1.0, 0.0, 0.0),
                tangent: Default::default(),
                uv: vec2(1.0, 1.0),
            },
            Vertex {
                pos: vec3(0.5, -0.5, -0.5),
                normal: vec3(1.0, 0.0, 0.0),
                tangent: Default::default(),
                uv: vec2(1.0, 0.0),
            },
            Vertex {
                pos: vec3(0.5, -0.5, -0.5),
                normal: vec3(1.0, 0.0, 0.0),
                tangent: Default::default(),
                uv: vec2(1.0, 0.0),
            },
            Vertex {
                pos: vec3(0.5, -0.5, 0.5),
                normal: vec3(1.0, 0.0, 0.0),
                tangent: Default::default(),
                uv: vec2(0.0, 0.0),
            },
            Vertex {
                pos: vec3(0.5, 0.5, 0.5),
                normal: vec3(1.0, 0.0, 0.0),
                tangent: Default::default(),
                uv: vec2(0.0, 1.0),
            },
            //face 5
            Vertex {
                pos: vec3(-0.5, -0.5, -0.5),
                normal: vec3(0.0, -1.0, 0.0),
                tangent: Default::default(),
                uv: vec2(0.0, 1.0),
            },
            Vertex {
                pos: vec3(0.5, -0.5, -0.5),
                normal: vec3(0.0, -1.0, 0.0),
                tangent: Default::default(),
                uv: vec2(1.0, 1.0),
            },
            Vertex {
                pos: vec3(0.5, -0.5, 0.5),
                normal: vec3(0.0, -1.0, 0.0),
                tangent: Default::default(),
                uv: vec2(1.0, 0.0),
            },
            Vertex {
                pos: vec3(0.5, -0.5, 0.5),
                normal: vec3(0.0, -1.0, 0.0),
                tangent: Default::default(),
                uv: vec2(1.0, 0.0),
            },
            Vertex {
                pos: vec3(-0.5, -0.5, 0.5),
                normal: vec3(0.0, -1.0, 0.0),
                tangent: Default::default(),
                uv: vec2(0.0, 0.0),
            },
            Vertex {
                pos: vec3(-0.5, -0.5, -0.5),
                normal: vec3(0.0, -1.0, 0.0),
                tangent: Default::default(),
                uv: vec2(0.0, 1.0),
            },
            //face 6
            Vertex {
                pos: vec3(-0.5, 0.5, -0.5),
                normal: vec3(0.0, 1.0, 0.0),
                tangent: Default::default(),
                uv: vec2(0.0, 1.0),
            },
            Vertex {
                pos: vec3(0.5, 0.5, -0.5),
                normal: vec3(0.0, 1.0, 0.0),
                tangent: Default::default(),
                uv: vec2(1.0, 1.0),
            },
            Vertex {
                pos: vec3(0.5, 0.5, 0.5),
                normal: vec3(0.0, 1.0, 0.0),
                tangent: Default::default(),
                uv: vec2(1.0, 0.0),
            },
            Vertex {
                pos: vec3(0.5, 0.5, 0.5),
                normal: vec3(0.0, 1.0, 0.0),
                tangent: Default::default(),
                uv: vec2(1.0, 0.0),
            },
            Vertex {
                pos: vec3(-0.5, 0.5, 0.5),
                normal: vec3(0.0, 1.0, 0.0),
                tangent: Default::default(),
                uv: vec2(0.0, 0.0),
            },
            Vertex {
                pos: vec3(-0.5, 0.5, -0.5),
                normal: vec3(0.0, 1.0, 0.0),
                tangent: Default::default(),
                uv: vec2(0.0, 1.0),
            },
        ];

        calc_tangents(&mut v);

        v
    }
}

fn calc_tangents(vertices: &mut [Vertex; 36]) { //TODO hard code this
    for triangle in vertices.chunks_mut(3) {
        let v0 = triangle[0];
        let v1 = triangle[1];
        let v2 = triangle[2];

        let pos0 = v0.pos;
        let pos1 = v1.pos;
        let pos2 = v2.pos;

        let uv0 = v0.uv;
        let uv1 = v1.uv;
        let uv2 = v2.uv;

        let edge1 = pos1 - pos0;
        let edge2 = pos2 - pos0;

        let delta_uv1 = uv1 - uv0;
        let delta_uv2 = uv2 - uv0;

        let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
        let tangent = (edge1 * delta_uv2.y - edge2 * delta_uv1.y) * r;

        triangle[0].tangent = tangent;
        triangle[1].tangent = tangent;
        triangle[2].tangent = tangent;
    }
}

