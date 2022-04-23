use diagnostic::Span;
use syntax::{Identifier, Syntax};

#[derive(Debug, Clone, PartialEq)]
pub struct Local {
    name: String,
    span: Span,
}

impl Local {
    /// Crate a new local binding definition.
    pub fn new(name: &str, span: Span) -> Local {
        Local {
            name: name.into(),
            span,
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
}

impl<'a> From<&Identifier> for Local {
    fn from(id: &Identifier) -> Self {
        Local {
            name: id.as_str().into(),
            span: id.span(),
        }
    }
}
