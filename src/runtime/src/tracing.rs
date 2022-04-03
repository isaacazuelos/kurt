use std::fmt::{Display, Formatter, Result};

use compiler::index::Indexable;

use crate::Runtime;

impl Runtime {
    pub(crate) fn trace(&self) {
        println!("{self}");
    }
}

impl Display for Runtime {
    fn fmt(&self, f: &mut Formatter) -> Result {
        self.fmt_where(f)?;
        writeln!(f)?;
        self.fmt_stack(f)?;
        Ok(())
    }
}

impl Runtime {
    fn fmt_where(&self, f: &mut Formatter) -> Result {
        let pc = self.current_frame().pc;
        let prototype = self.main.prototypes.get(pc.prototype.as_usize());
        let op = prototype.and_then(|p| p.get(pc.instruction));
        let span = prototype.and_then(|p| p.span_for_op(pc.instruction));

        if let Some(op) = op {
            write!(f, "op: {op}")?;
            if let Some(span) = span {
                write!(f, " at {span}")?;
            }
        } else {
            write!(f, "op: <none>")?;
        }

        Ok(())
    }

    fn fmt_stack(&self, f: &mut Formatter) -> Result {
        write!(f, "stack: [ ... | ",)?;

        for v in self.current_stack() {
            write!(f, "{:?}, ", v)?;
        }

        write!(f, "]")
    }
}
