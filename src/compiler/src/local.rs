use diagnostic::Span;

#[derive(Clone)]
pub struct Local {
    name: String,
    _definition_site: Span,
}

impl Local {
    pub fn new(name: &str, _definition_site: Span) -> Local {
        Local {
            name: name.into(),
            _definition_site,
        }
    }

    pub fn as_str(&self) -> &str {
        &self.name
    }
}
