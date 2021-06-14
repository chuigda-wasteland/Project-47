use crate::vm::al31f::stack::Stack;

pub trait Alloc {
    fn add_stack(&mut self, stack: *const Stack);
    fn collect(&mut self);

    // TODO what API would we like for allocating?
}
