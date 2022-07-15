//! Produce a stack trace diagnostic from an error.

use std::fmt::Write;

use diagnostic::{Diagnostic, DiagnosticCoordinator, Level};

use crate::{
    classes::Function,
    vm::{call_stack::CallFrame, VirtualMachine},
    Error,
};

impl VirtualMachine {
    pub fn stack_trace(
        &self,
        error: Error,
        coordinator: &mut DiagnosticCoordinator,
    ) {
        let mut d = Diagnostic::new(error.to_string());

        let id = self.current_closure().module().id();

        d.set_input(id);

        coordinator.register(if let Some(span) = self.last_op_span() {
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

    fn stack_trace_frame_diagnostic(&self, frame: &CallFrame) -> Diagnostic {
        let mut message = String::from("called by ");

        let prototype = self.stack[frame.bp()]
            .as_gc::<Function>()
            .expect("every frame base pointer is a closure")
            .prototype();

        let debug = prototype.debug_info();

        let name = prototype.name();

        write!(message, "{:?}", name)
            .expect("write failed while creating error message");

        if let Some(span) = debug.and_then(|d| d.span_of(frame.pc())) {
            write!(message, " at {span}")
                .expect("write failed while creating error message");
        }

        let mut d = Diagnostic::new(message);

        d.set_level(Level::Info);

        d
    }
}
