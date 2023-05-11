use crate::util;
use glam::{mat4, vec3, vec4, Mat4, Vec3};
use rand::distributions::Uniform;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::BindGroupEntry;

pub struct CameraBundle {
    pub camera: Camera,
    pub uniforms: Uniforms,
    pub uniform_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub uniform_layout: wgpu::BindGroupLayout,
    pub controller: Controller,
}

impl CameraBundle {
    pub fn new(device: &wgpu::Device) -> Self {
        let camera = Camera::default();
        let mut uniforms = Uniforms::new();
        uniforms.update(&camera);

        let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("graph_3d.camera.uniform_buffer"),
            contents: bytemuck::bytes_of(&uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let bind_group = Uniforms::bind_group(device, &uniform_layout, &uniform_buffer);

        Self {
            camera: Camera::default(),
            uniforms,
            uniform_buffer,
            bind_group,
            uniform_layout,
            controller: Controller::default(),
        }
    }
}

pub struct Camera {
    position: Vec3,
    target: Vec3,
    up: Vec3,
    aspect: f32,
    fov_y: f32,
    near: f32,
    far: f32,
}

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Uniforms {
    projection: Mat4,
    position: Vec3,
    _padding: f32,
}

impl Uniforms {
    pub fn new() -> Self {
        Self {
            position: Vec3::ZERO,
            projection: Mat4::IDENTITY,
            _padding: 0.0,
        }
    }

    pub fn update(&mut self, camera: &Camera) {
        self.position = camera.position;
        self.projection = camera.build_view_proj_matrix();
    }

    pub fn bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("graph_3d.camera.uniform_bind_group"),
            layout: &layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        })
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: vec3(0.0, 1.0, 3.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            aspect: 1024.0 / 768.0,
            fov_y: 45.0,
            near: 0.1,
            far: 100.0,
        }
    }
}

pub const OPENGL_TO_WGPU_MATRIX: Mat4 = mat4(
    vec4(1.0, 0.0, 0.0, 0.0),
    vec4(0.0, 1.0, 0.0, 0.0),
    vec4(0.0, 0.0, 0.5, 0.0),
    vec4(0.0, 0.0, 0.5, 1.0),
);

impl Camera {
    pub fn build_view_proj_matrix(&self) -> glam::Mat4 {
        let view = glam::Mat4::look_at_rh(self.position, self.target, self.up);
        let proj = glam::Mat4::perspective_rh(
            self.fov_y,
            self.aspect,
            self.near,
            self.far,
        );

        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}

#[derive(Default)]
pub struct Controller {
    pub w: bool,
    pub a: bool,
    pub s: bool,
    pub d: bool,
    pub right_mouse: bool,
}

impl Controller {

}