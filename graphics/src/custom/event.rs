//! Handle events of a custom shader widget.
use crate::core::keyboard;
use crate::core::mouse;
use crate::core::touch;

pub use crate::core::event::Status;

/// A [`custom::Shader`] event.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    /// A mouse event.
    Mouse(mouse::Event),

    /// A touch event.
    Touch(touch::Event),

    /// A keyboard event.
    Keyboard(keyboard::Event),
}
