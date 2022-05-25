//! The runtime stack.

use common::Index;
use compiler::Local;

use crate::value::Value;

/// The [`VirtualMachine`][crate::VirtualMachine]'s stack. This is where values
/// are kept while the program is working with them.
///
/// Stacks will work with a few types of indexes:
///
/// - [`Index<Stack>`] is an absolute index into the stack.
/// - [`Index<Local>`] is an index into the stack referring to a local binding,
///   and is relative to it's originating call frame's base pointer.
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
    pub const MAX: usize = common::min(Index::<Stack>::MAX, u32::MAX as usize);

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

    /// Returns the value on the top of the stack.
    ///
    /// This operation is sometimes called `peek`. Rust uses `last`
    ///
    ///  If the stack is empty, `()` is returned instead.
    /// This is the same as `Stack::get_from_top(0)` if the stack is not empty.
    pub fn last(&self) -> Value {
        if let Some(last) = self.values.last() {
            // FIXME: This is almost certainly a bad idea for the GC.
            *last
        } else {
            Value::UNIT
        }
    }
    /// Drop all the values from the top of the stack down to (and including)
    /// the given index.
    pub fn truncate_to(&mut self, index: Index<Stack>) {
        self.values.truncate(index.as_usize());
    }

    /// Drop `count` values from the top of the stack.
    ///
    /// # Panics
    ///
    /// This will panic if `count` is larger then the length of the stack.
    pub fn truncate_by(&mut self, count: u32) {
        let len = self
            .values
            .len()
            .checked_sub(count as usize)
            .expect("cannot truncate by more than the length of the stack");

        self.values.truncate(len);
    }

    pub fn last_n(&self, count: u32) -> &[Value] {
        if count == 0 {
            &[]
        } else {
            let index = self.index_from_top(count - 1).as_usize();
            &self.as_slice()[index..]
        }
    }

    /// Get a value at the given `index`. If the index is out of range, `None`
    /// is returned.
    pub fn get(&self, index: Index<Stack>) -> Option<Value> {
        self.values.get(index.as_usize()).cloned()
    }

    /// Set the value at `index` to `new`, replacing it.
    ///
    /// # Panics
    ///
    /// This will panic if `index` is past the end of the stack.
    pub fn set(&mut self, index: Index<Stack>, new: Value) {
        if let Some(value) = self.values.get_mut(index.as_usize()) {
            *value = new;
        } else {
            panic!("indexes to set on the stack should be in range")
        }
    }

    /// Get a value by it's local index. Local indexes are relative to some
    /// other absolute index, `base`, which would typically be a call frame's
    /// base pointer.
    ///
    /// # Panics
    ///
    /// This will panic if the resulting index is past the end of the stack.
    pub(crate) fn get_local(
        &self,
        base: Index<Stack>,
        local: Index<Local>,
    ) -> Value {
        let index = Stack::as_absolute_index(base, local);

        *self
            .values
            .get(index.as_usize())
            .expect("local indexes should not point past the top of the stack")
    }

    /// Return the value `count` spots from the top of the stack.
    ///
    /// # Panics
    ///
    /// This will panic if the resulting value would need to be below the start
    /// of the stack, i.e. if count is greater than [`Stack::len`].
    pub(crate) fn get_from_top(&self, count: u32) -> Value {
        let index = self.index_from_top(count);
        match self.get(index) {
            Some(v) => v,
            None => unreachable!(
                "index_from_top means it's not too big, and at least 0"
            ),
        }
    }

    /// Overwrite the value `count` spots from the top of the stack with the
    /// `new_value`.
    ///
    /// # Panics
    ///
    /// This will panic if the value would need to go below the start of the
    /// stack, i.e. if count is greater than [`Stack::len`].
    pub(crate) fn set_from_top(&mut self, count: u32, new_value: Value) {
        let index = self.index_from_top(count);
        self.set(index, new_value);
    }

    /// The index of the value `count` spaces from the top of the stack.
    ///
    /// When `count` is 0, this is the same as calling [`Stack::last`].
    ///
    /// # Panics
    ///
    /// This will panic if the resulting index is below the start of the stack.
    pub(crate) fn index_from_top(&self, count: u32) -> Index<Stack> {
        let i = self
            .values
            .len()
            .checked_sub(count as usize + 1)
            .expect("index from top would be below start of the stack");

        Index::new(i as u32)
    }

    /// Convert a [`Index<Local>`] into an [`Index<Stack>`], using the `base`
    /// index it's local relative to.
    ///
    /// See the documentation on [`Stack`] for more.
    pub fn as_absolute_index(
        base: Index<Stack>,
        index: Index<Local>,
    ) -> Index<Stack> {
        // local binding 0 still needs to be one more than the base pointer, so
        // we need to add 1. Since both index and bp are u32s, we can add them
        // (and the +1) without fear of overflows.
        let absolute = base.as_usize() + index.as_usize() + 1;

        Index::new(absolute as _)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn last_n() {
        let mut stack = Stack::default();
        stack.push(Value::from(false));
        stack.push(Value::from(true));

        assert_eq!(stack.last_n(1), [Value::from(true)]);
    }

    #[test]
    fn index_from_top() {
        let mut stack = Stack::default();
        stack.push(Value::from(false));
        stack.push(Value::from(true));

        assert_eq!(stack.index_from_top(0), Index::new(1));
        assert_eq!(stack.index_from_top(1), Index::new(0));
    }

    #[test]
    fn get_from_top() {
        let mut stack = Stack::default();
        stack.push(Value::from(false));
        stack.push(Value::from(true));

        assert_eq!(stack.get_from_top(0), Value::from(true));
        assert_eq!(stack.get_from_top(1), Value::from(false));
    }

    #[test]
    fn get_index_from_top() {
        let mut stack = Stack::default();
        stack.push(Value::from(false));
        stack.push(Value::from(true));

        assert_eq!(stack.get(stack.index_from_top(0)), Some(Value::from(true)));
        assert_eq!(
            stack.get(stack.index_from_top(1)),
            Some(Value::from(false))
        );
    }
}
