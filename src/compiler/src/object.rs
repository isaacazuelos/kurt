//! An object is the result of compiling some code.
//!
//! It's ready for the runtime. Like a python `.pyc` or C `.o` file.

use std::fmt::{self, Display, Formatter};

use crate::{
    constant::Constant,
    index::{Get, Index},
    opcode::Op,
    prototype::Prototype,
};

#[derive(Debug, Clone, Default)]
pub struct Object {
    pub(crate) constants: Vec<Constant>,
    pub(crate) prototypes: Vec<Prototype>,
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

impl Get<Prototype> for Object {
    fn get(&self, index: Index<Prototype>) -> Option<&Prototype> {
        self.prototypes.get(index.as_usize())
    }
}

impl Get<Constant> for Object {
    fn get(&self, index: Index<Constant>) -> Option<&Constant> {
        self.constants.get(index.as_usize())
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for (i, c) in self.constants().iter().enumerate() {
            writeln!(f, "constant {:03} = {}", i, c)?;
        }

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
        let name = prototype.name().unwrap_or("anonymous");
        write!(f, "{} ( ", name)?;

        for binding in prototype.parameters() {
            write!(f, "{}, ", binding.as_str())?;
        }

        writeln!(f, "):")?;

        for (i, (op, span)) in prototype.code().iter().enumerate() {
            write!(
                f,
                " {:03} ({:03}:{:03}) | ",
                i,
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
