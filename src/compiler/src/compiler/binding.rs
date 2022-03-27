//! Information about bound variables.

use syntax::ast::Identifier;

use crate::{index::Index, local::Local, Compiler};

impl Compiler {
    /// Bind the value at the top of the stack to the local variable named by
    /// the [`Identifier`], within the current scope.
    pub(crate) fn bind_local(&mut self, _name: &Identifier) {
        todo!()
    }

    /// Look up the index for a local variable
    pub(crate) fn lookup_local(
        &mut self,
        _name: &Identifier,
    ) -> Option<Index<Local>> {
        todo!()
    }
}
