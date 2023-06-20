//! A custom shader widget for wgpu applications.
use crate::core::layout::{Limits, Node};
use crate::core::mouse::Cursor;
use crate::core::renderer::Style;
use crate::core::widget::tree::{State, Tag};
use crate::core::widget::{tree, Tree};
use crate::core::{
    self, layout, mouse, widget, Clipboard, Element, Layout, Length, Rectangle,
    Shell, Size, Widget,
};
use crate::{Backend, Renderer};
use iced_core::mouse::Interaction;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

mod event;
mod primitive;
mod program;
mod storage;

pub use event::Event;
pub use primitive::Primitive;
pub use program::Program;
pub use storage::Storage;

/// A widget which allows for the custom rendering with `wgpu`.
///
/// Must be initialized with a [`Program`], which describes widget state & how it's rendered.
pub struct Shader<Message, P: Program<Message>> {
    width: Length,
    height: Length,
    program: P,
    _message: PhantomData<Message>,
}

impl<Message, P: Program<Message>> Debug for Shader<Message, P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "custom_shader_widget")
    }
}

impl<Message, P: Program<Message>> Shader<Message, P> {
    /// Create a new custom [`Shader`].
    pub fn new(program: P) -> Self {
        Self {
            width: Length::Fixed(100.0),
            height: Length::Fixed(100.0),
            program,
            _message: PhantomData,
        }
    }

    /// Set the `width` of the custom [`Shader`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Set the `height` of the [`Shader`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }
}

impl<P, Message, B, Theme> Widget<Message, crate::Renderer<B, Theme>>
    for Shader<Message, P>
where
    P: Program<Message>,
    B: Backend,
{
    fn tag(&self) -> Tag {
        struct Tag<T>(T);
        tree::Tag::of::<Tag<P::State>>()
    }

    fn state(&self) -> State {
        tree::State::new(P::State::default())
    }

    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        _renderer: &crate::Renderer<B, Theme>,
        limits: &Limits,
    ) -> Node {
        let limits = limits.width(self.width).height(self.height);
        let size = limits.resolve(Size::ZERO);

        layout::Node::new(size)
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: core::Event,
        layout: Layout<'_>,
        cursor: Cursor,
        _renderer: &Renderer<B, Theme>,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> core::event::Status {
        let bounds = layout.bounds();

        let custom_shader_event = match event {
            core::Event::Mouse(mouse_event) => Some(Event::Mouse(mouse_event)),
            core::Event::Keyboard(keyboard_event) => {
                Some(Event::Keyboard(keyboard_event))
            }
            core::Event::Touch(touch_event) => Some(Event::Touch(touch_event)),
            _ => None,
        };

        if let Some(custom_shader_event) = custom_shader_event {
            let state = tree.state.downcast_mut::<P::State>();

            let (event_status, message) = self.program.update(
                state,
                custom_shader_event,
                bounds,
                cursor,
                shell,
            );

            if let Some(message) = message {
                shell.publish(message);
            }

            return event_status;
        }

        event::Status::Ignored
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer<B, Theme>,
    ) -> Interaction {
        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<P::State>();

        self.program.mouse_interaction(state, bounds, cursor)
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut crate::Renderer<B, Theme>,
        _theme: &Theme,
        _style: &Style,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<P::State>();

        renderer.draw_primitive(crate::Primitive::custom(
            bounds,
            self.program.draw(state, cursor_position, bounds),
        ));
    }
}

impl<'a, M, P, B, Theme> From<Shader<M, P>>
    for Element<'a, M, crate::Renderer<B, Theme>>
where
    M: 'a,
    P: Program<M> + 'a,
    B: Backend,
{
    fn from(custom: Shader<M, P>) -> Element<'a, M, crate::Renderer<B, Theme>> {
        Element::new(custom)
    }
}
