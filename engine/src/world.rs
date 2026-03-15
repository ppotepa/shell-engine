use std::any::{Any, TypeId};
use std::collections::HashMap;

pub struct World {
    singletons: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    scoped: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            singletons: HashMap::new(),
            scoped: HashMap::new(),
        }
    }

    pub fn register<T: Any + Send + Sync + 'static>(&mut self, val: T) -> &mut Self {
        self.singletons.insert(TypeId::of::<T>(), Box::new(val));
        self
    }

    pub fn register_scoped<T: Any + Send + Sync + 'static>(&mut self, val: T) -> &mut Self {
        self.scoped.insert(TypeId::of::<T>(), Box::new(val));
        self
    }

    pub fn get<T: Any + 'static>(&self) -> Option<&T> {
        self.singletons
            .get(&TypeId::of::<T>())
            .or_else(|| self.scoped.get(&TypeId::of::<T>()))
            .and_then(|b| b.downcast_ref::<T>())
    }

    pub fn get_mut<T: Any + 'static>(&mut self) -> Option<&mut T> {
        let id = TypeId::of::<T>();
        if self.singletons.contains_key(&id) {
            self.singletons.get_mut(&id).and_then(|b| b.downcast_mut::<T>())
        } else {
            self.scoped.get_mut(&id).and_then(|b| b.downcast_mut::<T>())
        }
    }

    pub fn clear_scoped(&mut self) {
        self.scoped.clear();
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
