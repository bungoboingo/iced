mod cursor;
mod event;
mod program;

use crate::core::event::Status;
use crate::core::renderer::Style;
use crate::core::widget::{tree, Tree};
use crate::core::{
    layout, Clipboard, Layout, Length, Point, Rectangle, Shell, Size, Widget,
};
use crate::custom::cursor::Cursor;
use crate::custom::event::Event;
use crate::{Backend, Renderer};
use iced_core::Vector;
pub use program::Program;

/// A widget for rendering custom shaders.
pub struct Shader<P> {
    width: Length,
    height: Length,
    program: P,
}

impl<P> Shader<P> {
    /// Creates a new [`Shader`] widget.
    pub fn new(program: P) -> Self {
        Self {
            width: Length::Fill,
            height: Length::Fill,
            program,
        }
    }

    /// Sets the width of the [`Shader`] widget.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Shader`] widget.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }
}

impl<P, B, Theme, Message> Widget<Message, Renderer<B, Theme>> for Shader<P>
where
    P: Program<Message, Renderer<B, Theme>>,
    B: Backend, //TODO this should be wgpu only; depend on iced_wgpu and use it's renderer directly?
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        renderer: &Renderer<B, Theme>,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width).height(self.height);
        let size = limits.resolve(Size::ZERO);

        layout::Node::new(size)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer<B, Theme>,
        theme: &Theme,
        style: &Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        if bounds.width < 1.0 || bounds.height < 1.0 {
            return;
        }

        let cursor = Cursor::from_window_position(cursor_position);
        let state = tree.state.downcast_ref::<P::State>();

        renderer.with_translation(Vector::new(bounds.x, bounds.y), |renderer| {
            renderer.draw_primitive(
                self.program.draw(state, renderer, theme, bounds, cursor),
            )
        })
    }

    fn tag(&self) -> tree::Tag {
        struct Tag<T>(T);
        tree::Tag::of::<Tag<P::State>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(P::State::default())
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: crate::core::Event,
        layout: Layout<'_>,
        cursor_position: Point,
        _renderer: &Renderer<B, Theme>,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> Status {
        let bounds = layout.bounds();

        let custom_event = match event {
            crate::core::Event::Mouse(mouse_event) => {
                Some(Event::Mouse(mouse_event))
            }
            crate::core::Event::Touch(touch_event) => {
                Some(Event::Touch(touch_event))
            }
            crate::core::Event::Keyboard(keyboard_event) => {
                Some(Event::Keyboard(keyboard_event))
            }
            _ => None,
        };

        let cursor = Cursor::from_window_position(cursor_position);

        if let Some(ev) = custom_event {
            let state = tree.state.downcast_mut::<P::State>();

            let (event_status, message) =
                self.program.update(state, ev, bounds, cursor);

            if let Some(message) = message {
                shell.publish(message);
            }

            return event_status;
        }

        Status::Ignored
    }
}
