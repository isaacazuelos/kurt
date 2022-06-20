//! Exported top-level bindings

use diagnostic::Span;

pub struct Export {
    name: String,
    span: Span,
}

impl Export {
    pub fn new(name: &str, span: Span) -> Export {
        Export {
            name: name.into(),
            span,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn span(&self) -> Span {
        self.span
    }
}
