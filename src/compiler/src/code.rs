//! Code listings

use std::iter::Iterator;

use diagnostic::Span;

use crate::{
    error::{Error, Result},
    index::{Index, Indexable},
    opcode::Op,
};

/// A listing of opcodes for our VM in order.
#[derive(Debug, Clone, Default)]
pub(crate) struct Code {
    opcodes: Vec<Op>,
    spans: Vec<Span>,
}

impl Code {
    /// The maximum number of opcodes that can be in a single code block.
    const MAX_OPS: usize = (u32::MAX) as usize;

    /// Push an [`Op`] to the of the code segment.
    pub fn emit(&mut self, op: Op, span: Span) -> Result<()> {
        self.opcodes.push(op);
        if self.opcodes.len() == Self::MAX_OPS {
            Err(Error::TooManyOps)
        } else {
            self.spans.push(span);
            Ok(())
        }
    }

    /// The span which produced some op code.
    pub fn get_span(&self, index: Index<Op>) -> Option<Span> {
        self.spans.get(index.as_usize()).cloned()
    }

    /// Does this code block have no opcodes?
    pub fn is_empty(&self) -> bool {
        self.opcodes.is_empty()
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&Op, &Span)> {
        self.opcodes.iter().zip(self.spans.iter())
    }
}

impl Indexable<Op> for Code {
    fn get(&self, index: Index<Op>) -> Option<&Op> {
        self.opcodes.get(index.as_usize())
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
