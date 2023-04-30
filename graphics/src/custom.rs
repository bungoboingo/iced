//! A custom shader widget for wgpu applications.
use crate::{Backend, Primitive, Transformation};
use iced_core::layout::{Limits, Node};
use iced_core::renderer::Style;
use iced_core::widget::Tree;
use iced_core::{
    layout, Color, Element, Layout, Length, Point, Rectangle, Size, Widget,
};
use std::any::Any;

mod program;
pub use program::Program;
use crate::primitive::CustomPipeline;

#[derive(Debug)]
pub struct Custom<P: Program> {
    width: Length,
    height: Length,
    program: P,
}

impl<P: Program> Custom<P> {
    pub fn new(program: P) -> Self {
        Self {
            width: Length::Fill,
            height: Length::Fill,
            program,
        }
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }
}

impl<P, B, T, M> Widget<M, crate::Renderer<B, T>> for Custom<P>
where
    B: Backend,
    P: Program,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        _renderer: &crate::Renderer<B, T>,
        limits: &Limits,
    ) -> Node {
        let limits = limits.width(self.width).height(self.height);
        let size = limits.resolve(Size::ZERO);

        layout::Node::new(size)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut crate::Renderer<B, T>,
        theme: &T,
        _style: &Style,
        layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        if bounds.width < 1.0 || bounds.height < 1.0 {
            return;
        }

        renderer.draw_primitive(Primitive::Custom {
            bounds,
            pipeline: CustomPipeline {
                id: P::id(),
                pipeline: P::init,
                prepare: |state: &mut Box<dyn Any>,
                          device: &wgpu::Device,
                          queue: &wgpu::Queue,
                          encoder: &mut wgpu::CommandEncoder,
                          scale_factor: f32,
                          transformation: Transformation| {
                    P::prepare(
                        state.downcast_mut::<P>().unwrap(),
                        device,
                        queue,
                        encoder,
                        scale_factor,
                        transformation,
                    )
                },
                render: |state: Box<dyn Any>,
                         render_pass: &mut wgpu::RenderPass<'_>,
                         device: &wgpu::Device,
                         target: &wgpu::TextureView,
                         clear_color: Option<Color>,
                         scale_factor: f32,
                         target_size: Size<u32>| {
                    P::render(
                        state.downcast_ref::<P>().unwrap(),
                        render_pass,
                        device,
                        target,
                        clear_color,
                        scale_factor,
                        target_size
                    )
                },
            }
        })
    }
}

impl<'a, P, M, B, T> From<Custom<P>> for Element<'a, M, crate::Renderer<B, T>>
where
    M: 'a,
    P: Program + 'a,
    B: Backend,
{
    fn from(custom: Custom<P>) -> Element<'a, M, crate::Renderer<B, T>> {
        Element::new(custom)
    }
}
