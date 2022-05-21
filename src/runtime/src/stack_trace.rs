//! Produce a stack trace diagnostic from an error.

use diagnostic::{Diagnostic, DiagnosticCoordinator, Level, Span};

use compiler::{Function, Get, Module, ModuleDebug};

use crate::{address::Address, call_stack::CallFrame, Error, Runtime};

impl Runtime {
    pub fn stack_trace(
        &self,
        error: Error,
        coordinator: &mut DiagnosticCoordinator,
    ) {
        // this sucks
        let pc = self.pc();

        let mut d = Diagnostic::new(error.to_string());

        let id = self
            .get(pc.module)
            .and_then(Module::debug_info)
            .and_then(ModuleDebug::input_id);

        d.set_input(id);

        coordinator.register(if let Some(span) = self.span_of(pc) {
            d.highlight(span, "this is what caused the error")
        } else {
            d.info("debug info was stripped")
        });

        for frame in self.call_stack.iter() {
            let mut d = self.stack_trace_frame_diagnostic(frame);
            d.set_input(id);
            coordinator.register(d);
        }
    }

    fn span_of(&self, pc: Address) -> Option<Span> {
        self.get(pc.module)
            .and_then(|m| m.get(pc.function))
            .and_then(Function::debug_info)
            .and_then(|i| i.span_of(pc.instruction))
    }

    fn stack_trace_frame_diagnostic(&self, frame: CallFrame) -> Diagnostic {
        let mut message = String::from("called by ");

        let name = self
            .get(frame.pc.module)
            .and_then(|m| m.get(frame.pc.function))
            .and_then(Function::debug_info)
            .and_then(|d| d.name())
            .unwrap_or(Function::DEFAULT_NAMELESS_NAME);

        let span = self.span_of(frame.pc);

        message.push_str(name);

        if let Some(span) = span {
            message.push_str(&format!(" at {}", span));
        }

        let mut d = Diagnostic::new(message);

        d.set_level(Level::Info);

        d
    }
}
