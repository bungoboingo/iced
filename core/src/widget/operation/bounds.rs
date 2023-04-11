//! Operate on any [`Layout`].

use crate::widget::{Id, Operation};
use crate::{Layout, Rectangle};
use crate::widget::operation::Outcome;

/// Returns the bounds of a widget.
pub fn bounds(target: Id) -> impl Operation<Rectangle> {
    struct Bounds {
        target: Id,
        bounds: Rectangle,
    }

    impl Operation<Rectangle> for Bounds {
        fn container(
            &mut self,
            id: Option<&Id>,
            layout: Layout<'_>,
            operate_on_children: &mut dyn FnMut(&mut dyn Operation<Rectangle>),
        ) {
            match id {
                Some(id) if id == &self.target => {
                    self.bounds = layout.bounds();
                }
                _ => operate_on_children(self)
            }
        }

        fn finish(&self) -> Outcome<Rectangle> {
            Outcome::Some(self.bounds)
        }
    }

    Bounds {
        target,
        bounds: Rectangle::default()
    }
}