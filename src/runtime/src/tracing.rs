//! Some helpers for tracing execution.

use std::fmt::{Display, Formatter, Result};

use compiler::FunctionDebug;

use crate::VirtualMachine;

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
        let op = if let Some(op) = self.op() {
            format!("{op}")
        } else {
            String::from("<none>")
        };

        write!(
            f,
            "{:>12} {:<4} {:16}",
            self.current_closure()
                .prototype()
                .debug_info()
                .and_then(FunctionDebug::name)
                .unwrap_or("<unknown>"),
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

    fn fmt_stack(&self, f: &mut Formatter) -> Result {
        let omitted = self.value_stack().len() - self.stack_frame().len();

        write!(f, "[ ...{} | ", omitted)?;

        for v in self.stack_frame() {
            write!(f, "{:?}, ", v)?;
        }

        write!(f, "]")
    }
}
