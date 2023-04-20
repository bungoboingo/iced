use crate::custom::cursor::Cursor;
use crate::custom::event::Event;
use crate::{Primitive, Transformation};
use iced_core::{event, Rectangle};
use iced_core::{mouse, Color, Size};
use wgpu::{CommandEncoder, Device, Queue, TextureView};

/// The state & logic of a [`CustomShader`] widget.
pub trait Program<Message, Renderer>
where
    Renderer: iced_core::Renderer,
{
    /// The internal state mutated by the [`Program`].
    type State: Default;

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
        _event: Event, //TODO own event type
        _bounds: Rectangle,
        _cursor: Cursor, //TODO cursor type,
    ) -> (event::Status, Option<Message>) {
        (event::Status::Ignored, None)
    }

    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _encoder: &mut wgpu::CommandEncoder,
        scale_factor: f32,
        transformation: Transformation,
        primitives: &[Primitive],
    );

    /// Draws the state of the [`Program`], producing a custom primitive.
    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &Renderer::Theme,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Primitive;

    /// Renders the custom `Primitive`s.
    fn render(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clear_color: Option<Color>,
        scale_factor: f32,
        target_size: Size<u32>,
        primitives: &[Primitive],
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

    fn render(
        &self,
        device: &Device,
        encoder: &mut CommandEncoder,
        target: &TextureView,
        clear_color: Option<Color>,
        scale_factor: f32,
        target_size: Size<u32>,
        primitives: &[Primitive],
    ) {
        T::render(
            self,
            device,
            encoder,
            target,
            clear_color,
            scale_factor,
            target_size,
            primitives,
        )
    }

    fn prepare(
        &self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        scale_factor: f32,
        transformation: Transformation,
        primitives: &[Primitive],
    ) {
        T::prepare(
            self,
            device,
            queue,
            encoder,
            scale_factor,
            transformation,
            primitives,
        )
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &Renderer::Theme,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Primitive {
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
