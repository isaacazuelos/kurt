//! Some helpers for tracing execution.

use std::fmt::{Display, Formatter, Result};

use crate::Runtime;

impl Runtime {
    pub(crate) fn trace(&self) {
        println!("{self}");
    }
}

impl Display for Runtime {
    fn fmt(&self, f: &mut Formatter) -> Result {
        self.fmt_where(f)?;
        write!(f, " stack: ")?;
        self.fmt_stack(f)?;
        Ok(())
    }
}

impl Runtime {
    fn fmt_where(&self, f: &mut Formatter) -> Result {
        if let Ok(op) = self.op() {
            write!(f, "op: {:16}", format!("{op}"))?;
        } else {
            write!(f, "op: <none>          ")?;
        }

        Ok(())
    }

    fn fmt_stack(&self, f: &mut Formatter) -> Result {
        write!(f, "stack: [ ... | ",)?;

        for v in self.stack_frame() {
            write!(f, "{:?}, ", v)?;
        }

        write!(f, "]")
    }
}
