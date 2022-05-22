//! The runtime value stack.

use compiler::{Index, Local};

use crate::{
    error::{Error, Result},
    value::Value,
};

#[derive(Debug, Default)]
pub struct ValueStack {
    values: Vec<Value>,
}

impl ValueStack {
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

    /// Get a local by it's index (with respect to the frame's base pointer).
    pub(crate) fn get_local(
        &self,
        base: Index<ValueStack>,
        local: Index<Local>,
    ) -> Result<Value> {
        // base points to the current closure, so local 0 is at base + 1
        let index = base.as_usize() + 1 + local.as_usize();
        self.values
            .get(index)
            .cloned()
            .ok_or(Error::LocalIndexOutOfRange)
    }

    pub(crate) fn get_from_top(&self, r_index: u32) -> Result<Value> {
        if self.values.len() <= r_index as _ {
            Err(Error::StackIndexBelowZero)
        } else {
            Ok(self.values[self.values.len() - r_index as usize - 1])
        }
    }

    pub(crate) fn set_from_top(
        &mut self,
        r_index: u32,
        new_value: Value,
    ) -> Result<()> {
        if self.values.len() <= r_index as _ {
            Err(Error::StackIndexBelowZero)
        } else {
            let index = self.values.len() - r_index as usize - 1;
            self.values[index] = new_value;
            Ok(())
        }
    }

    pub(crate) fn index_from_top(&self, arg_count: u32) -> Index<ValueStack> {
        let i = self.values.len() - arg_count as usize;

        assert!(i <= u32::MAX as usize);

        Index::new(i as u32)
    }

    pub(crate) fn get(&self, index: Index<ValueStack>) -> Option<Value> {
        self.values.get(index.as_usize()).cloned()
    }

    pub(crate) fn last_n(&self, n: usize) -> &[Value] {
        let start = self.values.len().saturating_sub(n);
        &self.values[start..]
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
}
