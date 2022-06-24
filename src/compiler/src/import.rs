//! A description of an imported module

use diagnostic::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    name: String,
    span: Span,

    required_exports: Vec<String>,
}

impl Import {
    pub fn new(name: &str, span: Span) -> Import {
        Import {
            name: name.into(),
            span,
            required_exports: Vec::new(),
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn add_required_export(&mut self, name: &str) {
        for e in &self.required_exports {
            if e == name {
                return;
            }
        }

        self.required_exports.push(name.into());
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}
