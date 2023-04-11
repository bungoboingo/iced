//! Query or update internal widget state.
pub mod focusable;
pub mod scrollable;
pub mod text_input;
mod bounds;

pub use focusable::Focusable;
pub use scrollable::Scrollable;
pub use text_input::TextInput;
pub use bounds::bounds;

use crate::widget::Id;

use std::any::Any;
use std::fmt;
use std::rc::Rc;
use crate::Layout;

/// A piece of logic that can traverse the widget tree of an application in
/// order to query or update some widget state.
pub trait Operation<T> {
    /// Operates on a widget that contains other widgets.
    ///
    /// The `operate_on_children` function can be called to return control to
    /// the widget tree and keep traversing it.
    fn container(
        &mut self,
        id: Option<&Id>,
        _layout: Layout<'_>,
        operate_on_children: &mut dyn FnMut(&mut dyn Operation<T>),
    );

    /// Operates on a widget that can be focused.
    fn focusable(&mut self, _state: &mut dyn Focusable, _id: Option<&Id>) {}

    /// Operates on a widget that can be scrolled.
    fn scrollable(&mut self, _state: &mut dyn Scrollable, _id: Option<&Id>) {}

    /// Operates on a widget that has text input.
    fn text_input(&mut self, _state: &mut dyn TextInput, _id: Option<&Id>) {}

    /// Operates on a custom widget with some state.
    fn custom(&mut self, _state: &mut dyn Any, _id: Option<&Id>) {}

    /// Finishes the [`Operation`] and returns its [`Outcome`].
    fn finish(&self) -> Outcome<T> {
        Outcome::None
    }
}

/// The result of an [`Operation`].
pub enum Outcome<T> {
    /// The [`Operation`] produced no result.
    None,

    /// The [`Operation`] produced some result.
    Some(T),

    /// The [`Operation`] needs to be followed by another [`Operation`].
    Chain(Box<dyn Operation<T>>),
}

impl<T> fmt::Debug for Outcome<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "Outcome::None"),
            Self::Some(output) => write!(f, "Outcome::Some({output:?})"),
            Self::Chain(_) => write!(f, "Outcome::Chain(...)"),
        }
    }
}

/// Maps the output of an [`Operation`] using the given function.
pub fn map<A, B>(
    operation: Box<dyn Operation<A>>,
    f: impl Fn(A) -> B + 'static,
) -> impl Operation<B>
where
    A: 'static,
    B: 'static,
{
    #[allow(missing_debug_implementations)]
    struct Map<A, B> {
        operation: Box<dyn Operation<A>>,
        f: Rc<dyn Fn(A) -> B>,
    }

    impl<A, B> Operation<B> for Map<A, B>
    where
        A: 'static,
        B: 'static,
    {
        fn container(
            &mut self,
            id: Option<&Id>,
            layout: Layout<'_>,
            operate_on_children: &mut dyn FnMut(&mut dyn Operation<B>),
        ) {
            struct MapRef<'a, A> {
                operation: &'a mut dyn Operation<A>,
            }

            impl<'a, A, B> Operation<B> for MapRef<'a, A> {
                fn container(
                    &mut self,
                    id: Option<&Id>,
                    layout: Layout<'_>,
                    operate_on_children: &mut dyn FnMut(&mut dyn Operation<B>),
                ) {
                    let Self { operation, .. } = self;

                    operation.container(id, layout, &mut |operation| {
                        operate_on_children(&mut MapRef { operation });
                    });
                }

                fn scrollable(
                    &mut self,
                    state: &mut dyn Scrollable,
                    id: Option<&Id>,
                ) {
                    self.operation.scrollable(state, id);
                }

                fn focusable(
                    &mut self,
                    state: &mut dyn Focusable,
                    id: Option<&Id>,
                ) {
                    self.operation.focusable(state, id);
                }

                fn text_input(
                    &mut self,
                    state: &mut dyn TextInput,
                    id: Option<&Id>,
                ) {
                    self.operation.text_input(state, id);
                }

                fn custom(&mut self, state: &mut dyn Any, id: Option<&Id>) {
                    self.operation.custom(state, id);
                }
            }

            let Self { operation, .. } = self;

            MapRef {
                operation: operation.as_mut(),
            }
            .container(id, layout, operate_on_children);
        }

        fn focusable(&mut self, state: &mut dyn Focusable, id: Option<&Id>) {
            self.operation.focusable(state, id);
        }

        fn scrollable(&mut self, state: &mut dyn Scrollable, id: Option<&Id>) {
            self.operation.scrollable(state, id);
        }

        fn text_input(&mut self, state: &mut dyn TextInput, id: Option<&Id>) {
            self.operation.text_input(state, id);
        }

        fn custom(&mut self, state: &mut dyn Any, id: Option<&Id>) {
            self.operation.custom(state, id);
        }

        fn finish(&self) -> Outcome<B> {
            match self.operation.finish() {
                Outcome::None => Outcome::None,
                Outcome::Some(output) => Outcome::Some((self.f)(output)),
                Outcome::Chain(next) => Outcome::Chain(Box::new(Map {
                    operation: next,
                    f: self.f.clone(),
                })),
            }
        }
    }

    Map {
        operation,
        f: Rc::new(f),
    }
}

/// Produces an [`Operation`] that applies the given [`Operation`] to the
/// children of a container with the given [`Id`].
pub fn scope<T: 'static>(
    target: Id,
    operation: impl Operation<T> + 'static,
) -> impl Operation<T> {
    struct ScopedOperation<Message> {
        target: Id,
        operation: Box<dyn Operation<Message>>,
    }

    impl<Message: 'static> Operation<Message> for ScopedOperation<Message> {
        fn container(
            &mut self,
            id: Option<&Id>,
            _layout: Layout<'_>,
            operate_on_children: &mut dyn FnMut(&mut dyn Operation<Message>),
        ) {
            if id == Some(&self.target) {
                operate_on_children(self.operation.as_mut());
            } else {
                operate_on_children(self);
            }
        }

        fn finish(&self) -> Outcome<Message> {
            match self.operation.finish() {
                Outcome::Chain(next) => {
                    Outcome::Chain(Box::new(ScopedOperation {
                        target: self.target.clone(),
                        operation: next,
                    }))
                }
                outcome => outcome,
            }
        }
    }

    ScopedOperation {
        target,
        operation: Box::new(operation),
    }
}
