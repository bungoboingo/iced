use std::any::Any;
use crate::Transformation;
use iced_core::{Color, Rectangle, Size};
use std::hash::Hash;

pub trait Program {
    fn id() -> u64;

    fn init(device: &wgpu::Device, format: wgpu::TextureFormat) -> Box<dyn Any>;

    fn prepare(
        &mut self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _encoder: &mut wgpu::CommandEncoder,
        _scale_factor: f32,
        _transformation: Transformation,
    );

    fn render<'a, 'b>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'b>,
        _device: &wgpu::Device,
        _target: &wgpu::TextureView,
        _clear_color: Option<Color>,
        _scale_factor: f32,
        _target_size: Size<u32>,
    ) where
        'a: 'b;
}
