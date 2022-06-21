//! Exported top-level bindings

use diagnostic::Span;

pub struct Export {
    name: String,
    span: Span,
    is_var: bool,
}

impl Export {
    pub fn new(name: &str, span: Span) -> Export {
        Export {
            name: name.into(),
            span,
            is_var: false,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn is_var(&self) -> bool {
        self.is_var
    }

    pub fn set_var(&mut self, is_var: bool) {
        self.is_var = is_var;
    }
}
