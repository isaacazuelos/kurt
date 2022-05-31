//! The runtime stack.

use common::Index;
use compiler::Local;

use crate::value::Value;

/// This is used for creating `Index<StackTop>` indexes, to refer to values
/// relative to the top of the stack.
pub struct StackTop;

/// The [`VirtualMachine`][crate::VirtualMachine]'s stack. This is where values
/// are kept while the program is working with them.
///
/// Stacks will work with a few types of indexes:
///
/// - [`Index<Stack>`] is an absolute index into the stack, from the bottom.
/// - [`Index<Local>`] is an index into the stack referring to a local binding,
///   and is relative to it's originating call frame's base pointer.
/// - [`Index<StackTop>`] is an index relative to the top of the stack. This is
///   relative at the time of lookup, not the time when the index is created.
///
/// # Layout during execution
///
/// Here's an ASCII diagram of the stack, growing up.
///
/// ``` text
/// |                   <empty>                  |
/// | <values for current expression evaluating> | <- top of stack
/// |                     ...                    |
/// |                   <locals>                 |
/// |                     ...                    |
/// |         <closure that's executing>         | <- base pointer
/// ```
#[derive(Debug, Default)]
pub struct Stack {
    values: Vec<Value>,
}

impl Stack {
    /// The maximum number of values on the stack.
    ///
    /// This is the limit enforced to ensure that methods which index into the
    /// stack using a [`Index::<ValueStack>`] or a [`u32`] can access the whole
    /// stack.
    pub const MAX: usize = Index::<Stack>::MAX;

    /// The base pointer points to a closure, and that closure stores it's local
    /// bindings in the indexes above itself while executing.
    ///
    /// This offset is the distance between a [`CallFrame`]'s base pointer and
    /// that frame's local bindings.
    const LOCAL_BP_OFFSET: usize = 1;

    /// The length of the stack, the number of values on it.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Is the stack empty?
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// View the stack as a slice of values.
    pub fn as_slice(&self) -> &[Value] {
        &self.values
    }

    /// Push a new value on to the stack.
    ///
    /// # Panics
    ///
    /// This will panic if the stack size exceeds
    pub fn push(&mut self, value: Value) {
        if self.len() + 1 >= Stack::MAX {
            panic!("stack overflow")
        }

        self.values.push(value);
    }

    /// Pop a value off the stack. Popping the stack doesn't return the value
    /// because using the value after it's been popped could mean that value has
    /// been recycled by the garbage collector before it's used.
    ///
    /// Instead, you should get the value with [`Stack::last`] to use it. Only
    /// once the results of that use are safely back were the collector can find
    /// them is it safe to pop or truncate to get rid of the old values.
    pub fn pop(&mut self) {
        self.values.pop();
    }

    /// Returns the value on the top of the stack, if there is one.
    ///
    /// This operation is sometimes called `peek`.
    pub fn last(&self) -> Option<&Value> {
        self.values.last()
    }

    /// Drop the values on the stack above the given index.
    ///
    /// If the index is past the top fo the stack, nothing happens.
    ///
    /// # Note
    ///
    /// This is a bit different than [`Vec::truncate`], as that that sets the
    /// new length, and this sets the new top
    pub fn truncate_above(&mut self, new_top: Index<Stack>) {
        self.values.truncate(new_top.as_usize() + 1);
    }

    /// Returns a slice containing all the values on the stack above the given
    /// index.
    pub fn above(&self, top: Index<StackTop>) -> &[Value] {
        let start = self.from_top(top).as_usize() + 1;
        &self.values[start..]
    }

    /// Convert a [`Index<StackTop>`] into a [`Index<Stack>`] given the current
    /// length of the stack.
    ///
    /// # Panics
    ///
    /// This will panic if the resulting index is below the start of the stack.
    pub fn from_top(&self, index: Index<StackTop>) -> Index<Stack> {
        let i = self
            .values
            .len()
            .checked_sub(index.as_usize() + 1)
            .expect("stack underflow");

        Index::new(i as u32)
    }

    /// Convert a [`Index<Local>`] into an [`Index<Stack>`], using the `base`
    /// index the local index is relative to.
    pub fn from_local(base: Index<Stack>, index: Index<Local>) -> Index<Stack> {
        // local binding 0 still needs to be one more than the base pointer, so
        // we need to add 1. Since both index and bp are u32s, we can add them
        // (and the +1) without fear of overflows.
        let absolute =
            base.as_usize() + index.as_usize() + Self::LOCAL_BP_OFFSET;

        Index::new(absolute as _)
    }
}

impl std::ops::Index<Index<Stack>> for Stack {
    type Output = Value;
    fn index(&self, i: Index<Stack>) -> &Self::Output {
        self.values.index(i.as_usize())
    }
}

impl std::ops::IndexMut<Index<Stack>> for Stack {
    fn index_mut(&mut self, i: Index<Stack>) -> &mut Self::Output {
        self.values.index_mut(i.as_usize())
    }
}

impl std::ops::Index<(Index<Stack>, Index<Local>)> for Stack {
    type Output = Value;
    fn index(
        &self,
        (base, local): (Index<Stack>, Index<Local>),
    ) -> &Self::Output {
        let index = Stack::from_local(base, local);
        self.index(index)
    }
}

impl std::ops::IndexMut<(Index<Stack>, Index<Local>)> for Stack {
    fn index_mut(
        &mut self,
        (base, local): (Index<Stack>, Index<Local>),
    ) -> &mut Self::Output {
        let index = Stack::from_local(base, local);
        self.index_mut(index)
    }
}

impl std::ops::Index<common::Index<StackTop>> for Stack {
    type Output = Value;
    fn index(&self, index: Index<StackTop>) -> &Self::Output {
        let i = self.from_top(index);
        self.index(i)
    }
}

impl std::ops::IndexMut<common::Index<StackTop>> for Stack {
    fn index_mut(&mut self, index: Index<StackTop>) -> &mut Self::Output {
        let i = self.from_top(index);
        self.index_mut(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn above() {
        let mut stack = Stack::default();
        stack.push(Value::from(false));
        stack.push(Value::from(true));

        assert_eq!(stack.above(Index::START), []);
        assert_eq!(stack.above(Index::new(1)), [Value::from(true)]);
    }

    #[test]
    fn convert_from_top() {
        let mut stack = Stack::default();
        stack.push(Value::from(false));
        stack.push(Value::from(true));

        assert_eq!(stack.from_top(Index::new(0)), Index::new(1));
        assert_eq!(stack.from_top(Index::new(1)), Index::new(0));
    }

    #[test]
    fn get_from_top() {
        let mut stack = Stack::default();
        stack.push(Value::from(false));
        stack.push(Value::from(true));

        assert_eq!(stack[Index::<StackTop>::new(0)], Value::from(true));
        assert_eq!(stack[Index::<StackTop>::new(1)], Value::from(false));
    }

    #[test]
    fn index_from_top() {
        let mut stack = Stack::default();
        stack.push(Value::from(false));
        stack.push(Value::from(true));

        assert_eq!(stack[Index::<StackTop>::new(0)], Value::from(true));
        assert_eq!(stack[Index::<StackTop>::new(1)], Value::from(false));
    }

    #[test]
    fn truncate_above() {
        let mut stack = Stack::default();
        stack.push(Value::from(false));
        stack.push(Value::from(true));

        stack.truncate_above(Index::new(1));
        assert_eq!(stack.as_slice(), [Value::FALSE, Value::TRUE]);

        stack.truncate_above(Index::START);
        assert_eq!(stack.len(), 1);
    }
}
