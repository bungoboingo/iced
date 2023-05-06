use std::marker::PhantomData;
use wgpu::{BindGroupLayoutEntry, BufferAddress};

pub struct Uniforms {
    pub buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}