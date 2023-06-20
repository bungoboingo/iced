use crate::{custom, Transformation};
use iced_core::{event, mouse, Color, Event, Rectangle, Shell, Size};
use std::time::Duration;

/// The state and logic of a custom `Shader` widget.
///
/// A [`Program`] can mutate internal state and produce messages for an
// application.
pub trait Program<Message> {
    type State;
    type Primitive: custom::Primitive;

    /// Update the internal [`State`] of the [`Program]. This can be used to reflect state changes
    /// based on mouse & other events. You can use the [`Shell`] to publish messages, request a
    /// redraw for the window, etc. which can be used for smooth animations.
    fn update(
        &mut self,
        _state: &mut Self::State,
        _event: Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
        _shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        event::Status::Ignored
    }

    fn draw(&self, _state: &Self::State) -> Self::Primitive;

    //TODO ? Some other way to get the state into the custom shader widget draw()
    fn state(&self) -> &Self::State;

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        mouse::Interaction::default()
    }
}


