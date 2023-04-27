use crate::custom::cursor::Cursor;
use crate::custom::event::Event;
use crate::Transformation;
use iced_core::{event, Rectangle};
use iced_core::{mouse, Size};
use wgpu::{Queue, TextureView};

/// The state & logic of a [`CustomShader`] widget.
pub trait Program<Message, Renderer>
where
    Renderer: iced_core::Renderer,
{
    /// The internal state mutated by the [`Program`].
    type State: Default + 'static;

    /// Updates the [`State`] of the [`Program`].
    ///
    /// When a [`Program`] is used in a [`custom::Shader`], the runtime will call this method for
    /// each [`Event`].
    ///
    /// This method can optionally return a `Message` to notify an application of any meaningful
    /// interactions. By default, this method does nothing.
    fn update(
        &self,
        _state: &mut Self::State,
        _event: Event, //TODO can share with canvas
        _bounds: Rectangle,
        _cursor: Cursor, //TODO can share with canvas
    ) -> (event::Status, Option<Message>) {
        (event::Status::Ignored, None)
    }

    /// Draws the state of the [`Program`] directly to a [`wgpu::RenderPipeline`].
    fn draw(
        &self,
        state: &Self::State,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        bounds: Rectangle,
        cursor: Cursor,
    );

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: Rectangle,
        _cursor: Cursor,
    ) -> mouse::Interaction {
        mouse::Interaction::default()
    }
}

impl<Message, Renderer, T> Program<Message, Renderer> for &T
where
    Renderer: iced_core::Renderer,
    T: Program<Message, Renderer>,
{
    type State = T::State;

    fn update(
        &self,
        state: &mut Self::State,
        event: Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> (event::Status, Option<Message>) {
        T::update(self, state, event, bounds, cursor)
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        bounds: Rectangle,
        cursor: Cursor,
    ) {
        T::draw(self, state, renderer, theme, bounds, cursor)
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> mouse::Interaction {
        T::mouse_interaction(self, state, bounds, cursor)
    }
}
