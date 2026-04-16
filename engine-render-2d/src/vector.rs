use engine_render::VectorPrimitive;
use std::cell::RefCell;

thread_local! {
    static VECTOR_PRIMITIVES: RefCell<Vec<VectorPrimitive>> = const { RefCell::new(Vec::new()) };
}

pub fn clear_vector_primitives() {
    VECTOR_PRIMITIVES.with(|v| v.borrow_mut().clear());
}

pub fn push_vector_primitive(primitive: VectorPrimitive) {
    VECTOR_PRIMITIVES.with(|v| v.borrow_mut().push(primitive));
}

pub fn take_vector_primitives() -> Vec<VectorPrimitive> {
    VECTOR_PRIMITIVES.with(|v| std::mem::take(&mut *v.borrow_mut()))
}
