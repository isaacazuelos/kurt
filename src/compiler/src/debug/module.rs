//! Module display code

use std::fmt::{self, Display, Formatter};

use crate::{Function, Module, Op};

impl Display for Module {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "module {{")?;

        self.display_exports(f)?;

        if !self.functions.is_empty() {
            writeln!(f)?;
        }

        for function in &self.functions {
            function.display_listing(self, f)?;
        }

        write!(f, "}}")
    }
}

impl Module {
    fn display_exports(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "export {{ ")?;

        for export in &self.exports {
            write!(f, "{}, ", export)?;
        }

        writeln!(f, " }}")
    }
}

impl Function {
    fn display_listing(
        &self,
        module: &Module,
        f: &mut Formatter,
    ) -> fmt::Result {
        self.display_signature(f, module)?;
        writeln!(f, ":")?;
        self.display_code(f, module)
    }

    fn display_signature(
        &self,
        f: &mut Formatter,
        module: &Module,
    ) -> fmt::Result {
        self.display_name(f, module)?;

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

    fn display_name(&self, f: &mut Formatter, module: &Module) -> fmt::Result {
        if let Some(name) = self.name().map(|n| &module[n]) {
            write!(f, "{name}")
        } else {
            write!(f, "{}", Function::DEFAULT_NAMELESS_NAME)
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
                Op::LoadLocal(index) => {
                    write!(f, "{:<20} // ", format!("{op}"))?;

                    let local = self.debug_info().and_then(|d| {
                        d.parameter_names().get(index.as_usize())
                    });

                    if let Some(local) = local {
                        writeln!(f, "{}", local)
                    } else {
                        writeln!(f, "???")
                    }
                }

                Op::LoadConstant(index) => {
                    write!(f, "{:<20} // ", format!("{op}"))?;

                    writeln!(f, "{}", module[*index])
                }

                Op::LoadFunction(index) => {
                    write!(f, "{:<20} // ", format!("{op}"))?;

                    if let Some(name) =
                        module[*index].name().map(|n| &module[n])
                    {
                        writeln!(f, "{}", name)
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
