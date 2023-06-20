use std::fmt::Debug;
use crate::{custom, Transformation};
use iced_core::{Rectangle, Size};

/// A set of methods which allows a [`Primitive`] to be rendered.
pub trait Primitive: Debug + 'static {
    /// Processes the [`Primitive`], allowing for GPU buffer allocation.
    fn prepare(
        &self,
        format: wgpu::TextureFormat,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_size: Size<u32>,
        scale_factor: f32,
        transform: Transformation,
        storage: &mut custom::Storage,
    );

    /// Renders the [`Primitive`].
    fn render(
        &self,
        storage: &custom::Storage,
        bounds: Rectangle<u32>,
        target: &wgpu::TextureView,
        target_size: Size<u32>,
        encoder: &mut wgpu::CommandEncoder,
    );
}
