//! A custom shader widget for wgpu applications.
use crate::{Backend, Primitive};
use iced_core::layout::{Limits, Node};
use iced_core::renderer::Style;
use iced_core::widget::Tree;
use iced_core::{
    layout, Element, Layout, Length, Point, Rectangle, Size, Widget,
};
use std::collections::hash_map::DefaultHasher;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};

mod program;
use crate::primitive::CustomPipeline;
pub use program::{Program, RenderStatus};

pub struct Shader {
    width: Length,
    height: Length,
    init: fn(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        target_size: Size<u32>,
    ) -> Box<dyn Program + 'static>,
    id: u64,
}

impl Debug for Shader {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "custom_shader_widget({:?})", self.id)
    }
}

impl Shader {
    pub fn new(
        init: fn(
            device: &wgpu::Device,
            format: wgpu::TextureFormat,
            target_size: Size<u32>,
        ) -> Box<dyn Program + 'static>,
        id: impl Hash,
    ) -> Self {
        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);

        Self {
            width: Length::Fill,
            height: Length::Fill,
            init,
            id: hasher.finish(),
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

impl<B: Backend, T, M> Widget<M, crate::Renderer<B, T>> for Shader {
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
        _tree: &Tree,
        renderer: &mut crate::Renderer<B, T>,
        _theme: &T,
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
                id: self.id,
                init: self.init,
            },
        })
    }
}

impl<'a, M, B, T> From<Shader> for Element<'a, M, crate::Renderer<B, T>>
where
    M: 'a,
    B: Backend,
{
    fn from(custom: Shader) -> Element<'a, M, crate::Renderer<B, T>> {
        Element::new(custom)
    }
}
