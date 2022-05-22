//! Some helpers for tracing execution.

use std::fmt::{Display, Formatter, Result};

use crate::{vm::Address, VirtualMachine};

impl VirtualMachine {
    pub(crate) fn trace(&self) {
        println!("{self}");
    }
}

impl Display for VirtualMachine {
    fn fmt(&self, f: &mut Formatter) -> Result {
        self.fmt_where(f)?;
        write!(f, " stack: ")?;
        self.fmt_stack(f)?;
        // write!(f, "\nopen captures: ")?;
        // self.fmt_open_captures(f)?;
        Ok(())
    }
}

impl VirtualMachine {
    fn fmt_where(&self, f: &mut Formatter) -> Result {
        if let Ok(op) = self.op() {
            write!(f, "{} op: {:16}", self.pc(), format!("{op}"))?;
        } else {
            write!(f, "op: <none>          ")?;
        }

        Ok(())
    }

    #[allow(dead_code)] // useful, but rarely
    fn fmt_open_captures(&self, f: &mut Formatter) -> Result {
        write!(f, "[ ",)?;

        for v in self.open_captures.iter() {
            write!(f, "{:?}, ", v)?;
        }

        write!(f, "]")
    }

    fn fmt_stack(&self, f: &mut Formatter) -> Result {
        write!(f, "[ ... | ",)?;

        for v in self.stack_frame() {
            write!(f, "{:?}, ", v)?;
        }

        write!(f, "]")
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "m{:03}/p{:03}/i{:03}",
            self.module.as_usize(),
            self.function.as_usize(),
            self.instruction.as_usize()
        )
    }
}
