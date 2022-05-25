//! Module debug info

use std::fmt::{self, Display, Formatter};

use common::Get;
use diagnostic::InputId;

use crate::{Function, Module, ModuleBuilder, Op};

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleDebug {
    input_id: Option<InputId>,
}

impl ModuleDebug {
    pub fn new(builder: &ModuleBuilder) -> ModuleDebug {
        ModuleDebug {
            input_id: builder.id(),
        }
    }

    pub fn input_id(&self) -> Option<InputId> {
        self.input_id
    }

    pub fn set_input_id(&mut self, id: InputId) {
        self.input_id = Some(id);
    }
}

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
                Op::LoadConstant(index) => {
                    write!(f, "{:<20} // ", format!("{op}"))?;

                    if let Some(constant) = module.get(*index) {
                        writeln!(f, "{}", constant)
                    } else {
                        writeln!(f, "???")
                    }
                }

                Op::LoadClosure(index) => {
                    write!(f, "{:<20} // ", format!("{op}"))?;

                    if let Some(name) = module
                        .get(*index)
                        .and_then(Function::debug_info)
                        .and_then(|d| d.name())
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
