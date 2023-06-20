use crate::custom;
use iced_core::{event, mouse, Rectangle, Shell};

/// The state and logic of a custom `Shader` widget.
///
/// A [`Program`] can mutate internal state and produce messages for an application.
pub trait Program<Message> {
    /// The internal state of the [`Program`].
    type State: Default + 'static;
    /// The type of primitive this [`Program`] can render.
    type Primitive: custom::Primitive + 'static;

    /// Update the internal [`State`] of the [`Program`]. This can be used to reflect state changes
    /// based on mouse & other events. You can use the [`Shell`] to publish messages, request a
    /// redraw for the window, etc. which can be used for smooth animations.
    ///
    /// By default, this method does and returns nothing.
    fn update(
        &mut self,
        _state: &mut Self::State,
        _event: custom::Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
        _shell: &mut Shell<'_, Message>,
    ) -> (event::Status, Option<Message>) {
        (event::Status::Ignored, None)
    }

    /// Returns the [`Primitive`] to be rendered.
    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        _bounds: Rectangle,
    ) -> Self::Primitive;

    /// Returns the current mouse interaction of the [`Program`].
    ///
    /// The interaction returned will be in effect even if the cursor position is out of
    /// bounds of the program's [`custom::Shader`].
    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        mouse::Interaction::default()
    }
}


