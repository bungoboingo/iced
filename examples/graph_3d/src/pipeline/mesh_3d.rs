use std::collections::hash_set::Union;
use bytemuck::{Pod, Zeroable};
use crate::graph_3d::Axis;
use glam::{vec3, Vec3};
use iced::Size;
use rand::{thread_rng, Rng};
use wgpu::{BindGroupEntry, BufferAddress};
use wgpu::util::DeviceExt;
use crate::util::{cube, quad};

pub struct Mesh3DBundle {
    pub mesh_3d: Mesh3D,
    pub instances: wgpu::Buffer,
    pub vertices: wgpu::Buffer,
    pub indices: wgpu::Buffer,
    pub uniforms: Uniforms,
    pub uniform_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub uniform_layout: wgpu::BindGroupLayout,
}

impl Mesh3DBundle {
    pub fn new(
        x_axis: &Axis,
        y_axis: &Axis,
        z_axis: &Axis,
        device: &wgpu::Device,
        size: Size<u32>,
    ) -> Self {
        let uniforms = Uniforms {
            resolution: [size.width, size.height],
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("mesh_3d.uniform_buffer"),
            contents: bytemuck::bytes_of(&uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("mesh_3d.uniform_bind_group_layout"),
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

        // one instance is just a single vec3 position currently
        let instances = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("graph_3d.mesh_3d.instances"),
            size: std::mem::size_of::<Mesh3D>() as BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("graph_3d.mesh_3d.cube_vertices"),
            contents: bytemuck::cast_slice(&cube::VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("graph_3d.mesh_3d.cube_indices"),
            contents: bytemuck::cast_slice(&cube::INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            mesh_3d: Mesh3D::gen_rnd(x_axis, y_axis, z_axis),
            instances,
            vertices,
            indices,
            uniforms,
            uniform_buffer,
            bind_group,
            uniform_layout,
        }
    }
}

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct Uniforms {
    pub resolution: [u32; 2],
}

impl Uniforms {
    pub fn bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("mesh_3d.uniform_bind_group"),
            layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        })
    }
}

pub struct Mesh3D {
    pub vertices: [Vec3; 100], //hard-coded 100 for now
}

impl Mesh3D {
    pub fn gen_rnd(x_axis: &Axis, y_axis: &Axis, z_axis: &Axis) -> Self {
        Self {
            vertices: std::array::from_fn(|index| {
                vec3(x_axis.rnd_step(), y_axis.rnd_step(), z_axis.rnd_step())
            }),
        }
    }
}
