//! Code listings

use diagnostic::Span;

use crate::{
    error::{Error, Result},
    index::{Get, Index},
    opcode::Op,
};

/// A listing of opcodes for our VM in order.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct Code {
    opcodes: Vec<Op>,
    spans: Vec<Span>,
}

impl Code {
    /// Push an [`Op`] to the of the code segment.
    pub fn emit(&mut self, op: Op, span: Span) -> Result<()> {
        if self.opcodes.len() >= Index::<Op>::MAX {
            Err(Error::TooManyOps(span))
        } else {
            self.opcodes.push(op);
            self.spans.push(span);
            Ok(())
        }
    }

    pub(crate) fn ops(&self) -> &[Op] {
        &self.opcodes
    }

    pub(crate) fn next_index(&self) -> Index<Op> {
        Index::new(self.opcodes.len() as u32)
    }

    /// Patch an existing instruction with another given instruction, at a
    /// specific index. Returns the replaced op, or `None` if the index is
    /// invalid.
    pub(crate) fn patch(&mut self, index: Index<Op>, op: Op) -> Option<Op> {
        if let old @ Some(_) = self.get(index).cloned() {
            self.opcodes[index.as_usize()] = op;
            old
        } else {
            None
        }
    }
}

impl Get<Op> for Code {
    fn get(&self, index: Index<Op>) -> Option<&Op> {
        self.opcodes.get(index.as_usize())
    }
}

impl From<Code> for Vec<Op> {
    fn from(val: Code) -> Self {
        val.opcodes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emit() {
        let mut code = Code::default();
        code.emit(Op::False, Span::default()).unwrap();
        assert_eq!(code.opcodes[0], Op::False);
        assert_eq!(code.spans[0], Span::default());
    }
}
