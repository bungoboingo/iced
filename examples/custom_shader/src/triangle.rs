use iced::advanced::layout::{Limits, Node};
use iced::advanced::renderer::Style;
use iced::advanced::widget::Tree;
use iced::advanced::{layout, Layout, Widget};
use iced::{Color, Element, Length, Point, Rectangle, Size};
use iced_graphics::primitive::{ColoredVertex2D, CustomPipeline, Renderable};
use iced_graphics::{Backend, Primitive, Transformation};
use std::collections::hash_map::DefaultHasher;
use std::convert::Into;
use std::hash::{Hash, Hasher};

pub struct Triangle {
    height: Length,
    width: Length,
    id: u64,
}

impl Triangle {
    pub fn new() -> Self {
        Self {
            height: Length::Shrink,
            width: Length::Shrink,
            id: 0,
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

    pub fn id(mut self, id: impl Hash) -> Self {
        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);

        self.id = hasher.finish();
        self
    }
}

impl<'a, B, Theme, Message> From<Triangle>
    for Element<'a, Message, iced_graphics::Renderer<B, Theme>>
where
    Message: 'a,
    B: Backend,
{
    fn from(
        triangle: Triangle,
    ) -> Element<'a, Message, iced_graphics::Renderer<B, Theme>> {
        Element::new(triangle)
    }
}

impl<B, T, Message> Widget<Message, iced_graphics::Renderer<B, T>> for Triangle
where
    B: Backend,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        _renderer: &iced_graphics::Renderer<B, T>,
        limits: &Limits,
    ) -> Node {
        let limits = limits.width(self.width).height(self.height);
        let size = limits.resolve(Size::ZERO);

        layout::Node::new(size)
    }

    fn draw(
        &self,
        _state: &Tree,
        renderer: &mut iced_graphics::Renderer<B, T>,
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
                init: State::init,
            },
        })
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
struct Uniforms {
    transform: [f32; 16],
    scale: f32,
}
