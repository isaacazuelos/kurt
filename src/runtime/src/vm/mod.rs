//! The virtual machine is the heart of how the language executes code.

use common::Index;
use compiler::{Constant, Op};
use diagnostic::Span;

mod call_stack;
mod exit;
mod instructions;
mod open_captures;
mod stack;
mod stack_trace;

use crate::{
    classes::{Closure, Keyword, Module, String},
    memory::{Gc, GcAny},
    value::Value,
    vm::open_captures::OpenCaptures,
    Result,
};

pub use self::{
    call_stack::{CallFrame, CallStack},
    exit::Exit,
    stack::Stack,
};

/// A struct that manages an instance of the language runtime.
#[derive(Default)]
pub struct VirtualMachine {
    modules: Vec<Gc<Module>>,

    // VM
    stack: Stack,
    call_stack: CallStack,

    // Heap
    pub(crate) heap_head: Option<GcAny>,
    pub(crate) open_captures: OpenCaptures,
}

impl VirtualMachine {
    /// Load a module into the runtime and execute its top-level code.
    pub fn load(&mut self, module: compiler::Module) -> Result<Exit> {
        self.load_without_running(module)?;

        let new_module = *self
            .modules
            .last()
            .expect("load_without_running left a module for us");

        let main_closure: Gc<Closure> = self.make_from(new_module.main());

        self.stack.push(Value::from(main_closure));
        let bp = self.stack.from_top(Index::START);

        self.call_stack.push(CallFrame::new(Index::START, bp));

        self.run()
    }

    pub fn reload_main(
        &mut self,
        new_main_module: compiler::Module,
    ) -> Result<()> {
        debug_assert_eq!(
            self.call_stack.len(),
            1,
            "runtime have halted because it hit the end of main."
        );

        self.load_without_running(new_main_module)?;

        // To replace main, we want to stash the instruction index, swap out
        // the closure, and keep any captures it has.

        let old_main: Gc<Closure> = self.stack[self.bp()].as_gc().unwrap();

        let new_main: Gc<Closure> =
            self.make_from(self.modules.last().unwrap().main());

        for i in 0..old_main.capture_count() {
            let index = Index::new(i);
            let cell = old_main.get_capture_cell(index);
            new_main.push_capture_cell(cell);
        }

        let mut frame = self.call_stack.pop();

        frame.pc_mut().saturating_decrement();

        let bp = frame.bp();

        self.call_stack.push(frame);
        self.stack[bp] = Value::from(new_main);

        Ok(())
    }

    /// Resume the runtime. If it hasn't been started before this will also
    /// start it.
    ///
    /// See the implementation note on [`last_result`] for details.
    pub fn resume(&mut self) -> Result<Exit> {
        self.stack.pop();
        self.run()
    }

    /// A string containing a representation of the last value on the stack.
    ///
    /// This is useful for the `repl` and `eval` subcommands.
    pub fn last_result(&self) -> std::string::String {
        if let Some(value) = self.stack.last() {
            format!("{:?}", value)
        } else {
            "<stack empty>".into()
        }
    }
}

impl VirtualMachine {
    pub(crate) fn load_without_running(
        &mut self,
        module: compiler::Module,
    ) -> Result<()> {
        let live_module: Gc<Module> = self.make_from(());

        self.modules.push(live_module);

        unsafe {
            Module::destructively_set_up_from_compiler_module(
                live_module,
                module,
                self,
            )
        };

        Ok(())
    }

    /// A reference to the [`VirtualMachine`]'s [`Stack`].
    pub(crate) fn stack(&self) -> &Stack {
        &self.stack
    }

    /// The base pointer, or the value which indicates where in the stack values
    /// pertaining to the currently executing closure begin.
    ///
    /// The value below the base pointer is the closure that's currently
    /// executing.
    pub fn bp(&self) -> Index<Stack> {
        self.call_stack.frame().bp()
    }

    /// The program counter is the address of the currently executing piece of
    /// code.
    pub fn pc(&self) -> Index<Op> {
        self.call_stack.frame().pc()
    }

    /// The next opcode in the current closure, if there is one.
    pub fn op(&self) -> Op {
        self.stack()[self.bp()]
            .as_gc::<Closure>()
            .expect("base pointer wasn't a closure")
            .get_op(self.pc())
            .expect("program counter out of range")
    }

    /// The span of the [`Op`] before, the current frame's program counter.
    ///
    /// Note that this won't unwind calls, so if this is called at the start of
    /// a closure, it still just returns the first op of that closure.
    pub fn last_op_span(&self) -> Option<Span> {
        let index = self.pc().saturating_previous();

        self.current_closure()
            .prototype()
            .debug_info()?
            .span_of(index)
    }

    /// The values on the stack in the current stack frame.
    pub fn stack_frame(&self) -> &[Value] {
        let start = self.bp().as_usize();
        &self.stack.as_slice()[start..]
    }

    /// The currently executing closure.
    ///
    /// # Panics
    ///
    /// This will panic if there's no call frame (so no base pointer), or if the
    /// base pointer isn't pointing to a closure.
    pub fn current_closure(&self) -> Gc<Closure> {
        let bp = self.bp();
        self.stack[bp].as_gc().expect("bp not pointing to closure")
    }

    /// The program counter is the address of the currently executing piece of
    /// code.
    pub fn pc_mut(&mut self) -> &mut Index<Op> {
        self.call_stack.frame_mut().pc_mut()
    }
}

impl VirtualMachine {
    /// Inflate a [`Constant`] into a full-fledged runtime value.
    fn inflate(&mut self, constant: &Constant) -> Result<Value> {
        match constant {
            Constant::Character(c) => Ok(Value::char(*c)),
            Constant::Float(bits) => Ok(Value::float(f64::from_bits(*bits))),
            Constant::String(s) => {
                let string: Gc<String> = self.make_from(s.as_str());
                Ok(Value::gc(string))
            }
            Constant::Keyword(kw) => {
                let keyword: Gc<Keyword> = self.make_from(kw.as_str());
                Ok(Value::gc(keyword))
            }
        }
    }
}
