//! Module debug info

use std::fmt::{self, Display, Formatter};

use crate::{Function, Get, Module, Op};

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleDebug {}

impl Display for Module {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "module {{")?;

        if !self.functions.is_empty() {
            writeln!(f)?;
        }

        for function in &self.functions {
            function.display_listing(self, f)?;
        }

        write!(f, "}}")
    }
}

impl Function {
    fn display_listing(
        &self,
        module: &Module,
        f: &mut Formatter,
    ) -> fmt::Result {
        self.display_signature(f)?;
        writeln!(f, ":")?;
        self.display_code(f, module)
    }

    fn display_signature(&self, f: &mut Formatter) -> fmt::Result {
        self.display_name(f)?;

        if let Some(debug) = self.debug_info() {
            write!(f, "( ")?;
            for p in debug.parameter_names() {
                write!(f, "{p}, ")?;
            }
            write!(f, ")")
        } else {
            write!(f, "( {} )", self.parameter_count())
        }
    }

    fn display_name(&self, f: &mut Formatter) -> fmt::Result {
        if let Some(name) = self.debug_info().and_then(|d| d.name()) {
            write!(f, "{name}")
        } else {
            write!(f, "<anonymous function>")
        }
    }

    fn display_code(
        &self,
        f: &mut std::fmt::Formatter,
        module: &Module,
    ) -> fmt::Result {
        for (i, op) in self.code.iter().enumerate() {
            if let Some(span) =
                self.debug_info().and_then(|d| d.code_spans.get(i))
            {
                write!(
                    f,
                    " {:03} ({:03}:{:03}) | ",
                    i,
                    span.start().line(),
                    span.start().column()
                )?;
            }

            match op {
                Op::LoadConstant(constant_index) => {
                    write!(f, "{:<20} // ", format!("{op}"))?;

                    if let Some(constant) = module.get(*constant_index) {
                        writeln!(f, "{}", constant)
                    } else {
                        writeln!(f, "???")
                    }
                }
                op => writeln!(f, "{}", op),
            }?;
        }

        Ok(())
    }
}
