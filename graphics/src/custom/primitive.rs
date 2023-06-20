use crate::custom;
use iced_core::{Rectangle, Size};

pub trait Primitive {
    fn prepare(
        &self,
        format: wgpu::TextureFormat,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_size: Size<u32>,
        storage: &mut custom::Storage,
    );

    fn render(
        &self,
        storage: &custom::Storage,
        bounds: Rectangle<u32>,
        target: &wgpu::TextureView,
        target_size: Size<u32>,
        encoder: &mut wgpu::CommandEncoder,
    );
}
