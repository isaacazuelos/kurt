//! The runtime value stack.

use common::Index;
use compiler::Local;

use crate::value::Value;

#[derive(Debug, Default)]
pub struct ValueStack {
    values: Vec<Value>,
}

impl ValueStack {
    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn as_slice(&self) -> &[Value] {
        &self.values
    }

    pub fn iter(&self) -> impl Iterator<Item = &Value> {
        self.values.iter()
    }

    pub fn push(&mut self, value: Value) {
        self.values.push(value);
    }

    /// Note that popping the stack doesn't return the value. This is because
    /// using the value after it's been popped for anything that could trigger
    /// the collector could mean that the value is deallocated.
    ///
    /// Instead, you should get the values, use them, and then overwrite the
    /// value on the stack with the result. Only then should you pop or truncate
    /// to get rid of the old values.
    pub fn pop(&mut self) {
        self.values.pop();
    }

    /// Drop all the values from the top of the stack down to (and including)
    /// the given index.
    pub fn truncate_to(&mut self, index: Index<ValueStack>) {
        self.values.truncate(index.as_usize());
    }

    /// Drop `count` values from the top of the stack. If `count` is larger then the
    /// length of the stack, it'll be left empty.
    pub fn truncate_by(&mut self, count: u32) {
        let len = self.values.len().saturating_sub(count as usize);
        self.values.truncate(len);
    }

    /// Get the last value added to the stack, sometimes called the 'top of
    /// stack'. If the stack is empty, `()` is returned instead. This is the
    /// same as `Stack::get_from_top(0)` if the stack is not empty.
    pub(crate) fn last(&self) -> Value {
        if let Some(last) = self.values.last() {
            // FIXME: This is almost certainly a bad idea for the GC.
            *last
        } else {
            Value::UNIT
        }
    }

    // Set the value at `index` to `new`.
    pub(crate) fn set(&mut self, index: Index<ValueStack>, new: Value) {
        if let Some(value) = self.values.get_mut(index.as_usize()) {
            *value = new;
        }
    }

    /// Get a local by it's index (with respect to the frame's base pointer).
    pub(crate) fn get_local(
        &self,
        base: Index<ValueStack>,
        local: Index<Local>,
    ) -> Value {
        // base points to the current closure, so local 0 is at base + 1
        let index = ValueStack::as_absolute_index(base, local);

        *self
            .values
            .get(index.as_usize())
            .expect("local index past end of stack")
    }

    /// Return the value `r_index` spots from the top of the stack.
    ///
    /// `get_from_top(0)` returns the same things as `pop()` without removing
    /// the value.
    pub(crate) fn get_from_top(&self, r_index: u32) -> Value {
        let index = self.index_from_top(r_index);
        self.get(index).expect("indexed past end of stack")
    }

    pub(crate) fn set_from_top(&mut self, r_index: u32, new_value: Value) {
        let index = self
            .values
            .len()
            .checked_sub(r_index as usize + 1)
            .expect("cannot set below start of stack");

        self.values[index] = new_value;
    }

    /// The index of the value `count` spaces from the top of the stack.
    ///
    /// index_from_top(0) is the index of the top of the stack.
    pub(crate) fn index_from_top(&self, count: u32) -> Index<ValueStack> {
        let i = self
            .values
            .len()
            .checked_sub(count as usize + 1)
            .expect("index from top would be below bottom of stack");

        // TODO: When would this happen?
        assert!(
            i <= u32::MAX as usize,
            "stack index {} from top is too big",
            count
        );

        Index::new(i as u32)
    }

    pub(crate) fn get(&self, index: Index<ValueStack>) -> Option<Value> {
        self.values.get(index.as_usize()).cloned()
    }

    pub(crate) fn last_n(&self, n: usize) -> &[Value] {
        let start = self.values.len().saturating_sub(n);
        &self.values[start..]
    }

    pub(crate) fn as_absolute_index(
        bp: Index<ValueStack>,
        index: Index<Local>,
    ) -> Index<ValueStack> {
        let big = bp.as_usize() + index.as_usize() + 1;

        if big > u32::MAX as usize {
            panic!("stack index too big");
        }

        Index::new(big as _)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn last_n() {
        let mut stack = ValueStack::default();
        stack.push(Value::from(false));
        stack.push(Value::from(true));

        assert_eq!(stack.last_n(1), [Value::from(true)]);
    }

    #[test]
    fn index_from_top() {
        let mut stack = ValueStack::default();
        stack.push(Value::from(false));
        stack.push(Value::from(true));

        assert_eq!(stack.index_from_top(0), Index::new(1));
        assert_eq!(stack.index_from_top(1), Index::new(0));
    }

    #[test]
    fn get_from_top() {
        let mut stack = ValueStack::default();
        stack.push(Value::from(false));
        stack.push(Value::from(true));

        assert_eq!(stack.get_from_top(0), Value::from(true));
        assert_eq!(stack.get_from_top(1), Value::from(false));
    }

    #[test]
    fn get_index_from_top() {
        let mut stack = ValueStack::default();
        stack.push(Value::from(false));
        stack.push(Value::from(true));

        assert_eq!(stack.get(stack.index_from_top(0)), Some(Value::from(true)));
        assert_eq!(
            stack.get(stack.index_from_top(1)),
            Some(Value::from(false))
        );
    }
}
