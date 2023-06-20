use std::any::{Any, TypeId};
use std::collections::HashMap;

#[derive(Default, Debug)]
/// Stores pipelines.
pub struct Storage {
    pipelines: HashMap<TypeId, Box<dyn Any>>,
}

impl Storage {
    /// Returns `true` if `Storage` contains a pipeline with type `T`.
    pub fn has<T: 'static>(&self) -> bool {
        self.pipelines.get(&TypeId::of::<T>()).is_some()
    }

    /// Inserts the pipeline `T` in [`Storage`].
    pub fn store<T: 'static>(&mut self, pipeline: T) {
        let _ = self.pipelines.insert(TypeId::of::<T>(), Box::new(pipeline));
    }

    /// Returns a reference to pipeline with type `T` if it exists in [`Storage`].
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.pipelines.get(&TypeId::of::<T>()).map(|pipeline| {
            pipeline
                .downcast_ref::<T>()
                .expect("Pipeline with this type does not exist in Storage.")
        })
    }

    /// Returns a mutable reference to pipeline `T` if it exists in [`Storage`].
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.pipelines.get_mut(&TypeId::of::<T>()).map(|pipeline| {
            pipeline.downcast_mut::<T>()
                .expect("Pipeline with this type does not exist in Storage.")
        })
    }
}
