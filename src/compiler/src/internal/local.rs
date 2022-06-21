use diagnostic::Span;
use syntax::{Identifier, Syntax};

#[derive(Debug, Clone, PartialEq)]
pub struct Local {
    name: String,
    span: Span,
    is_captured: bool,
    is_var: bool,
}

impl Local {
    /// Crate a new local binding definition.
    pub fn new(name: &str, span: Span, var: bool) -> Local {
        Local {
            name: name.into(),
            span,
            is_captured: false,
            is_var: var,
        }
    }

    /// The local binding's name, as a `&str`.
    pub fn as_str(&self) -> &str {
        &self.name
    }

    /// Where the local binding was defined.
    pub fn span(&self) -> Span {
        self.span
    }

    /// Is this captured by some later closure?
    pub fn is_captured(&self) -> bool {
        self.is_captured
    }

    pub fn is_var(&self) -> bool {
        self.is_var
    }

    pub fn capture(&mut self) {
        self.is_captured = true;
    }
}

impl<'a> From<&Identifier> for Local {
    fn from(id: &Identifier) -> Self {
        Local {
            name: id.as_str().into(),
            span: id.span(),
            is_captured: false,
            is_var: false,
        }
    }
}
