use glam::{vec3, Vec2, Vec3};
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

fn v3d(
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

pub struct Cube {
    pub rotation: glam::Quat,
    pub position: Vec3,
    pub _padding: f32,
}

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug)]
#[repr(C)]
pub struct CubeRaw(glam::Mat4);

impl CubeRaw {
    const ATTRIBS: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
        //mat4
        5 => Float32x4,
        6 => Float32x4,
        7 => Float32x4,
        8 => Float32x4,
    ];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

const FRONT_TOP_LEFT: Vec3 = vec3(-1.0, -1.0, 1.0);
const FRONT_TOP_RIGHT: Vec3 = vec3(1.0, -1.0, 1.0);
const FRONT_BOTTOM_RIGHT: Vec3 = vec3(1.0, -1.0, -1.0);
const FRONT_BOTTOM_LEFT: Vec3 = vec3(-1.0, -1.0, -1.0);

const BACK_BOTTOM_LEFT: Vec3 = vec3(-1.0, 1.0, -1.0);
const BACK_TOP_LEFT: Vec3 = vec3(-1.0, 1.0, 1.0);
const BACK_TOP_RIGHT: Vec3 = vec3(1.0, 1.0, 1.0);
const BACK_BOTTOM_RIGHT: Vec3 = vec3(1.0, 1.0, -1.0);

impl Cube {
    pub fn to_raw(&self) -> CubeRaw {
        CubeRaw(glam::Mat4::from_rotation_translation(
            self.rotation,
            self.position,
        ))
    }

    // could create this with normals
    pub fn vertices(scale: f32) -> [Vertex3D; 24] {
        let front_n: Vec3 =
            normal_of(FRONT_TOP_LEFT, FRONT_TOP_RIGHT, FRONT_BOTTOM_RIGHT);
        let left_n: Vec3 =
            normal_of(BACK_TOP_LEFT, FRONT_TOP_LEFT, FRONT_BOTTOM_LEFT);
        let back_n: Vec3 =
            normal_of(BACK_TOP_LEFT, BACK_TOP_RIGHT, BACK_BOTTOM_RIGHT);
        let right_n: Vec3 =
            normal_of(BACK_TOP_RIGHT, FRONT_TOP_RIGHT, FRONT_BOTTOM_RIGHT);
        let bottom_n: Vec3 =
            normal_of(FRONT_BOTTOM_LEFT, BACK_BOTTOM_LEFT, BACK_BOTTOM_RIGHT);
        let top_n: Vec3 =
            normal_of(FRONT_TOP_LEFT, BACK_TOP_LEFT, BACK_TOP_RIGHT);

        let light_blue_front_bottom = Color::from_rgba8(75, 118, 156, 0.8);
        let light_blue_front_top = Color::from_rgba8(179, 245, 255, 0.8);
        let darker_blue_back_bottom = Color::from_rgba8(48, 86, 120, 0.8);
        let darker_blue_back_top = Color::from_rgba8(115, 208, 222, 0.8);

        [
            //front vertices
            v3d(FRONT_TOP_LEFT * scale, Vec2::ZERO, front_n, light_blue_front_top),
            v3d(FRONT_TOP_RIGHT * scale, Vec2::ZERO, front_n, light_blue_front_top),
            v3d(
                FRONT_BOTTOM_RIGHT * scale,
                Vec2::ZERO,
                front_n,
                light_blue_front_bottom,
            ),
            v3d(
                FRONT_BOTTOM_LEFT * scale,
                Vec2::ZERO,
                front_n,
                light_blue_front_bottom,
            ),
            //left vertices
            v3d(BACK_TOP_LEFT * scale, Vec2::ZERO, left_n, darker_blue_back_top),
            v3d(FRONT_TOP_LEFT * scale, Vec2::ZERO, left_n, light_blue_front_top),
            v3d(
                FRONT_BOTTOM_LEFT * scale,
                Vec2::ZERO,
                left_n,
                light_blue_front_bottom,
            ),
            v3d(
                BACK_BOTTOM_LEFT * scale,
                Vec2::ZERO,
                left_n,
                darker_blue_back_bottom,
            ),
            //back vertices
            v3d(BACK_TOP_RIGHT * scale, Vec2::ZERO, back_n, darker_blue_back_top),
            v3d(BACK_TOP_LEFT * scale, Vec2::ZERO, back_n, darker_blue_back_top),
            v3d(
                BACK_BOTTOM_LEFT * scale,
                Vec2::ZERO,
                back_n,
                darker_blue_back_bottom,
            ),
            v3d(
                BACK_BOTTOM_RIGHT * scale,
                Vec2::ZERO,
                back_n,
                darker_blue_back_bottom,
            ),
            //right vertices
            v3d(FRONT_TOP_RIGHT * scale, Vec2::ZERO, right_n, darker_blue_back_top),
            v3d(BACK_TOP_RIGHT * scale, Vec2::ZERO, right_n, light_blue_front_top),
            v3d(
                BACK_BOTTOM_RIGHT * scale,
                Vec2::ZERO,
                right_n,
                light_blue_front_bottom,
            ),
            v3d(
                FRONT_BOTTOM_RIGHT * scale,
                Vec2::ZERO,
                right_n,
                darker_blue_back_bottom,
            ),
            //bottom vertices
            v3d(
                FRONT_BOTTOM_LEFT * scale,
                Vec2::ZERO,
                bottom_n,
                light_blue_front_bottom,
            ),
            v3d(
                BACK_BOTTOM_LEFT * scale,
                Vec2::ZERO,
                bottom_n,
                darker_blue_back_bottom,
            ),
            v3d(
                BACK_BOTTOM_RIGHT * scale,
                Vec2::ZERO,
                bottom_n,
                darker_blue_back_bottom,
            ),
            v3d(
                FRONT_BOTTOM_RIGHT * scale,
                Vec2::ZERO,
                bottom_n,
                light_blue_front_bottom,
            ),
            //top vertices
            v3d(FRONT_TOP_LEFT * scale, Vec2::ZERO, top_n, light_blue_front_top),
            v3d(BACK_TOP_LEFT * scale, Vec2::ZERO, top_n, darker_blue_back_top),
            v3d(BACK_TOP_RIGHT * scale, Vec2::ZERO, top_n, darker_blue_back_top),
            v3d(FRONT_TOP_RIGHT * scale, Vec2::ZERO, top_n, light_blue_front_top),
        ]
    }
}

//TODO can I just calc from two verts..?
fn normal_of(vert_1: Vec3, vert_2: Vec3, vert_3: Vec3) -> Vec3 {
    let u = vec3(
        vert_2.x - vert_1.x,
        vert_2.y - vert_1.y,
        vert_2.z - vert_1.z,
    );
    let v = vec3(
        vert_3.x - vert_1.x,
        vert_3.y - vert_1.y,
        vert_3.z - vert_1.z,
    );

    vec3(
        (u.y * v.z) - (u.z * v.y),
        (u.z * v.x) - (u.x * v.z),
        (u.x * v.y) - (u.y * v.x),
    )
}
