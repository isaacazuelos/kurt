//! The runtime value stack.

use compiler::{index::Index, local::Local};

use crate::{
    error::{Error, Result},
    value::Value,
};

#[derive(Debug, Default)]
pub struct Stack {
    values: Vec<Value>,
}

impl Stack {
    pub fn as_slice(&self) -> &[Value] {
        &self.values
    }

    pub fn push(&mut self, value: Value) {
        self.values.push(value);
    }

    pub fn pop(&mut self) -> Value {
        self.values.pop().unwrap_or_default()
    }

    pub fn truncate_to(&mut self, index: Index<Stack>) {
        self.values.truncate(index.as_usize());
    }

    pub(crate) fn last(&self) -> Value {
        if let Some(last) = self.values.last() {
            // FIXME: This is almost certainly a bad idea for the GC.
            *last
        } else {
            Value::UNIT
        }
    }

    pub(crate) fn get_local(
        &self,
        base: Index<Stack>,
        local: Index<Local>,
    ) -> Result<Value> {
        let index = base.as_usize() + local.as_usize();
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

    pub(crate) fn index_from_top(&self, arg_count: u32) -> Index<Stack> {
        let i = self.values.len() - arg_count as usize;

        assert!(i <= u32::MAX as usize);

        Index::new(i as u32)
    }
}
