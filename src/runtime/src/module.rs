//! Runtime module representation.

use std::{collections::HashMap, iter::Product};

use crate::value::Value;

use compiler::prototype::Prototype;

#[derive(Debug)]
pub struct Module {
    /// All the constants in this module.
    pub(crate) constants: Vec<Value>,

    /// The prototype for this modules `_main`, i.e. the top level code.
    pub(crate) main: Prototype,

    /// The other prototypes used by this modules functions.
    pub(crate) prototypes: Vec<Prototype>,
}

impl Default for Module {
    fn default() -> Self {
        Module {
            constants: Vec::new(),
            prototypes: Vec::new(),
            main: Prototype::new_main(),
        }
    }
}

impl Module {}
