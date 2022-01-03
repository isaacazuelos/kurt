//! Code listings

use diagnostic::Span;

use crate::opcode::Op;

/// A listing of opcodes for our VM in order.
#[derive(Debug, Clone, Default)]
pub struct Code {
    opcodes: Vec<Op>,
    spans: Vec<Span>,
}

impl Code {
    /// Push an [`Op`] to the of the code segment
    pub(crate) fn emit(&mut self, op: Op, span: Span) {
        self.opcodes.push(op);
        self.spans.push(span);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emit() {
        let mut code = Code::default();
        code.emit(Op::False, Span::default());
        assert_eq!(code.opcodes[0], Op::False);
        assert_eq!(code.spans[0], Span::default());
    }
}
