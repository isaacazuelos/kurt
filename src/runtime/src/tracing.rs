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
        writeln!(f)?;
        self.fmt_stack(f)?;
        Ok(())
    }
}

impl Runtime {
    fn fmt_where(&self, f: &mut Formatter) -> Result {
        if let Ok(op) = self.current_op() {
            write!(f, "op: {op}")?;
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
