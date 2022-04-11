//! An object is the result of compiling some code.
//!
//! It's ready for the runtime. Like a python `.pyc` or C `.o` file.

use std::fmt::{self, Display, Formatter};

use crate::{
    constant::Constant,
    index::{Index, Indexable},
    opcode::Op,
    prototype::Prototype,
};

#[derive(Debug, Clone)]
pub struct Object {
    pub(crate) constants: Vec<Constant>,
    pub(crate) prototypes: Vec<Prototype>,
}

impl Default for Object {
    fn default() -> Self {
        Object {
            constants: Vec::new(),
            prototypes: Vec::new(),
        }
    }
}

impl Object {
    /// A view of all the constants used, the ordering matches the
    /// [`Index<Constant>`]s used within prototypes.
    pub fn constants(&self) -> &[Constant] {
        &self.constants
    }

    /// A view of all the constants used, the indexes match the
    /// [`Index<Prototype>`]s used by code within the object.
    pub fn prototypes(&self) -> &[Prototype] {
        &self.prototypes
    }
}

impl Indexable<Prototype> for Object {
    fn get(&self, index: Index<Prototype>) -> Option<&Prototype> {
        self.prototypes.get(index.as_usize())
    }
}

impl Indexable<Constant> for Object {
    fn get(&self, index: Index<Constant>) -> Option<&Constant> {
        self.constants.get(index.as_usize())
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for prototype in self.prototypes() {
            self.display_prototype(prototype, f)?;
        }

        Ok(())
    }
}

impl Object {
    fn display_prototype(
        &self,
        prototype: &Prototype,
        f: &mut Formatter,
    ) -> fmt::Result {
        if let Some(name) = prototype.name() {
            writeln!(f, "{}:", name)?;
        } else {
            writeln!(
                f,
                "<anonymous function, parameter_count: {}>:",
                prototype.parameter_count()
            )?;
        }

        for (op, span) in prototype.code().iter() {
            write!(
                f,
                "  {:03}:{:03} | ",
                span.start().line(),
                span.start().column()
            )?;

            match op {
                Op::LoadConstant(i) => {
                    if let Some(c) = self.get(*i) {
                        writeln!(f, "{:<20} // {}", format!("{op}"), c)
                    } else {
                        writeln!(f, "{} // ???", op)
                    }
                }
                op => writeln!(f, "{}", op),
            }?;
        }

        Ok(())
    }
}
