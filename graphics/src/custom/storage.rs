use std::any::{Any, TypeId};
use std::collections::HashMap;

#[derive(Default)]
pub struct Storage {
    pipelines: HashMap<TypeId, Box<dyn Any>>,
}

impl Storage {
    pub fn get<T>(&self) -> Option<&T> {
        self.pipelines
            .get(&T::type_id())
            .map(|pipeline| pipeline.downcast_ref::<T>())
            .flatten()
    }

    pub fn get_mut<T>(&mut self) -> Option<&mut T> {
        self.pipelines
            .get_mut(&T::type_id())
            .map(|pipeline| pipeline.downcast_mut::<T>())
            .flatten()
    }

    pub fn store<T>(&mut self, mut pipeline: T) -> &mut T {
        if self.pipelines.get(&T::type_id()).is_none() {
            //pipeline not currently stored. Maybe we should replace it?
            self.pipelines.insert(T::type_id(), Box::new(pipeline))
        }

        &mut pipeline
    }
}
