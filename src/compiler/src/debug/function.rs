//! Function debug info

use diagnostic::Span;

use crate::{internal::FunctionBuilder, Index, Op};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct FunctionDebug {
    pub(crate) name: Option<String>,
    pub(crate) parameter_names: Vec<String>,
    pub(crate) code_spans: Vec<Span>,
}

impl FunctionDebug {
    pub(crate) fn new(builder: &FunctionBuilder) -> Option<FunctionDebug> {
        let parameter_names = builder
            .parameters()
            .iter()
            .map(|l| l.as_str().to_owned())
            .collect();

        Some(FunctionDebug {
            name: builder.name().map(ToOwned::to_owned),
            parameter_names,
            code_spans: builder.code().spans().to_owned(),
        })
    }

    pub fn parameter_names(&self) -> &[String] {
        &self.parameter_names
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn span_of(&self, index: Index<Op>) -> Option<Span> {
        self.code_spans.get(index.as_usize()).cloned()
    }
}