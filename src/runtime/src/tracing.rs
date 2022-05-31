//! Some helpers for tracing execution.
//!
//! This is all behind the "trace" feature gates, and useful for debugging the
//! runtime, but isn't really fit for any other use.

use std::fmt::{Display, Formatter, Result};

use crate::VirtualMachine;

impl VirtualMachine {
    pub(crate) fn trace(&self) {
        println!("{self}");
    }
}

impl Display for VirtualMachine {
    fn fmt(&self, f: &mut Formatter) -> Result {
        self.fmt_where(f)?;
        // write!(f, " ")?;
        // self.fmt_stack(f)?;
        Ok(())
    }
}

impl VirtualMachine {
    fn fmt_where(&self, f: &mut Formatter) -> Result {
        let op = format!("{}", self.op());

        write!(
            f,
            "{:>12} {:<4} {:16}",
            format!("{:?}", self.current_closure().name()),
            self.pc().as_usize(),
            op,
        )
    }

    #[allow(dead_code)] // useful, but rarely
    fn fmt_open_captures(&self, f: &mut Formatter) -> Result {
        write!(f, "[ ",)?;

        for v in self.open_captures.iter() {
            write!(f, "{:?}, ", v)?;
        }

        write!(f, "]")
    }

    #[allow(dead_code)] // useful, but rarely
    fn fmt_stack(&self, f: &mut Formatter) -> Result {
        let omitted = self.stack().len() - self.stack_frame().len();

        write!(f, "[ ...{} | ", omitted)?;

        for v in self.stack_frame() {
            write!(f, "{:?}, ", v)?;
        }

        write!(f, "]")
    }
}
