//! Type-erased resource container that holds both persistent singleton and per-scene scoped resources.

use std::any::{Any, TypeId};
use std::collections::HashMap;

/// A type-erased container for engine resources, split into persistent singletons and per-scene scoped entries.
pub struct World {
    singletons: HashMap<TypeId, Box<dyn Any + Send>>,
    scoped: HashMap<TypeId, Box<dyn Any + Send>>,
}

impl World {
    /// Creates an empty [`World`] with no registered resources.
    pub fn new() -> Self {
        Self {
            singletons: HashMap::new(),
            scoped: HashMap::new(),
        }
    }

    /// Inserts `val` as a singleton resource of type `T`.
    pub fn register<T: Any + Send + 'static>(&mut self, val: T) -> &mut Self {
        self.singletons.insert(TypeId::of::<T>(), Box::new(val));
        self
    }

    /// Inserts `val` as a scoped resource of type `T` (cleared by [`clear_scoped`](Self::clear_scoped)).
    pub fn register_scoped<T: Any + Send + 'static>(&mut self, val: T) -> &mut Self {
        self.scoped.insert(TypeId::of::<T>(), Box::new(val));
        self
    }

    /// Returns a shared reference to the resource of type `T`, checking singletons then scoped.
    pub fn get<T: Any + 'static>(&self) -> Option<&T> {
        self.singletons
            .get(&TypeId::of::<T>())
            .or_else(|| self.scoped.get(&TypeId::of::<T>()))
            .and_then(|b| b.downcast_ref::<T>())
    }

    /// Returns an exclusive reference to the resource of type `T`, checking singletons then scoped.
    pub fn get_mut<T: Any + 'static>(&mut self) -> Option<&mut T> {
        let id = TypeId::of::<T>();
        if self.singletons.contains_key(&id) {
            self.singletons
                .get_mut(&id)
                .and_then(|b| b.downcast_mut::<T>())
        } else {
            self.scoped.get_mut(&id).and_then(|b| b.downcast_mut::<T>())
        }
    }

    /// Drops all scoped resources, typically called on scene transitions.
    pub fn clear_scoped(&mut self) {
        self.scoped.clear();
    }

    /// Calls `f` with simultaneous shared access to `A` and exclusive access to `B`.
    ///
    /// Both resources must be registered as singletons (not scoped). Avoids the need
    /// to clone `A` just to satisfy the borrow checker when `B` also needs `&mut World`.
    ///
    /// # Panics
    /// Will not panic as long as `A != B` (enforced by distinct `TypeId`s).
    pub fn with_ref_and_mut<A, B, F, R>(&mut self, f: F) -> Option<R>
    where
        A: Any + Send + 'static,
        B: Any + Send + 'static,
        F: FnOnce(&A, &mut B) -> R,
    {
        assert_ne!(
            TypeId::of::<A>(),
            TypeId::of::<B>(),
            "with_ref_and_mut: A and B must be different types"
        );
        // SAFETY: TypeId::of::<A>() != TypeId::of::<B>() (asserted above), so
        // the two HashMap entries are distinct. We capture a raw pointer to A
        // to hold a shared reference while separately mutably borrowing B.
        // The HashMap is not structurally modified between these two borrows.
        let a_ptr = self
            .singletons
            .get(&TypeId::of::<A>())?
            .downcast_ref::<A>()? as *const A;
        let b = self
            .singletons
            .get_mut(&TypeId::of::<B>())?
            .downcast_mut::<B>()?;
        let a = unsafe { &*a_ptr };
        Some(f(a, b))
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
