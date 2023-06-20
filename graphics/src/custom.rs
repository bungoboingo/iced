//! A custom shader widget for wgpu applications.
use crate::{renderer, Backend};
use iced_core::layout::{Limits, Node};
use iced_core::renderer::Style;
use iced_core::widget::Tree;
use iced_core::{
    layout, widget, Element, Layout, Length, Point, Rectangle, Size, Widget,
};
use std::collections::hash_map::DefaultHasher;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

mod primitive;
mod program;
mod storage;

pub use primitive::Primitive;
pub use program::Program;
pub use storage::Storage;

type Renderer<T> = crate::Renderer<wgpu::Backend, T>;

pub struct Shader<Message, P: Program<Message>> {
    width: Length,
    height: Length,
    program: P,
    _message: PhantomData<Message>,
}

impl<Message, P: Program<Message>> Debug for Shader<Message, P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "custom_shader_widget({:?})", self.id)
    }
}

impl<Message, P: Program<Message>> Shader<Message, P> {
    pub fn new(program: P) -> Self {
        Self {
            width: Length::Fixed(100.0),
            height: Length::Fixed(100.0),
            program,
            _message: PhantomData,
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

impl<P, Message, Theme> Widget<Message, Renderer<Theme>> for Shader<Message, P>
where
    P: Program<Message>,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        _renderer: &Renderer<Theme>,
        limits: &Limits,
    ) -> Node {
        let limits = limits.width(self.width).height(self.height);
        let size = limits.resolve(Size::ZERO);

        layout::Node::new(size)
    }

    fn draw(
        &self,
        state: &widget::Tree,
        renderer: &mut Renderer<Theme>,
        _theme: &Theme,
        _style: &Style,
        layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        renderer.draw_primitive(crate::Primitive::Custom {
            bounds,
            primitive: Box::new(self.program.draw(self.program.state())),
        })
    }
}

impl<'a, M, P, Theme> From<Shader<M, P>> for Element<'a, M, Renderer<Theme>>
where
    M: 'a,
    P: Program<M>,
{
    fn from(custom: Shader<M, P>) -> Element<'a, M, Renderer<Theme>> {
        Element::new(custom)
    }
}
