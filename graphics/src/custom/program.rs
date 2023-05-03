use std::any::Any;
use crate::Transformation;
use iced_core::{Color, Rectangle, Size};
use std::hash::Hash;
use std::time::Duration;

#[derive(Debug)]
pub enum RenderState {
    Dirty,
    Clean,
}

pub trait Program {
    fn update(
        &mut self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _encoder: &mut wgpu::CommandEncoder,
        _scale_factor: f32,
        _transformation: Transformation,
        time: Duration,
    );

    fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        _device: &wgpu::Device,
        _target: &wgpu::TextureView,
        _clear_color: Option<Color>,
        _scale_factor: f32,
        _target_size: Size<u32>,
    ) -> RenderState {
        RenderState::Clean
    }
}
