use glam::{mat4, vec3, vec4};
use wgpu::BindGroupEntry;
use wgpu::util::{BufferInitDescriptor, DeviceExt};

pub struct Camera {
    eye: glam::Vec3,
    target: glam::Vec3,
    up: glam::Vec3,
    aspect: f32,
    fov_y: f32,
    near: f32,
    far: f32,
}

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Uniforms {
    projection: glam::Mat4,
}

impl Uniforms {
    pub fn new() -> Self {
        Self {
            projection: glam::Mat4::IDENTITY,
        }
    }

    pub fn update(&mut self, camera: &Camera) {
        self.projection = camera.build_view_proj_matrix();
    }

    pub fn raw(&self, device: &wgpu::Device) -> crate::util::uniforms::Uniforms {
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("graph_3d.camera.uniform_buffer"),
            contents: bytemuck::bytes_of(self),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("graph_3d.camera.uniform_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("graph_3d.camera.uniform_bind_group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        crate::util::uniforms::Uniforms {
            buffer,
            bind_group_layout,
            bind_group,
        }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            eye: vec3(0.0, 1.0, 2.0),
            target: glam::Vec3::ZERO,
            up: glam::Vec3::Y,
            aspect: 1024.0/768.0,
            fov_y: 45.0,
            near: 0.1,
            far: 100.0,
        }
    }
}

pub const OPENGL_TO_WGPU_MATRIX: glam::Mat4 = mat4(
    vec4(1.0, 0.0, 0.0, 0.0),
    vec4(0.0, 1.0, 0.0, 0.0),
    vec4(0.0, 0.0, 0.5, 0.0),
    vec4(0.0, 0.0, 0.5, 1.0),
);

impl Camera {
    pub fn build_view_proj_matrix(&self) -> glam::Mat4 {
        let view = glam::Mat4::look_at_rh(self.eye, self.target, self.up);
        let proj = glam::Mat4::perspective_rh(self.fov_y, self.aspect, self.near, self.far);

        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}