//! The virtual machine is the heart of how the language executes code.

use common::Index;
use compiler::{Constant, Op};
use diagnostic::Span;

mod call_stack;
mod exit;
mod instructions;
mod open_captures;
mod stack_trace;
mod value_stack;

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
    value_stack::ValueStack,
};

/// A struct that manages an instance of the language runtime.
#[derive(Default)]
pub struct VirtualMachine {
    modules: Vec<Gc<Module>>,

    // VM
    value_stack: ValueStack,
    call_stack: CallStack,

    // Heap
    pub(crate) heap_head: Option<GcAny>,
    pub(crate) open_captures: OpenCaptures,
}

impl VirtualMachine {
    /// Load a module into the runtime and execute its top-level code.
    pub fn load(&mut self, module: compiler::Module) -> Result<Exit> {
        self.load_without_running(module)?;

        let new_module = self
            .modules
            .last()
            .expect("load_without_running left a module for us")
            .clone();

        let main_closure: Gc<Closure> = self.make_from(new_module.main());

        self.value_stack.push(Value::from(main_closure));
        let bp = self.value_stack.index_from_top(0);

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

        let old_main: Gc<Closure> =
            self.value_stack().get(self.bp()).unwrap().as_gc().unwrap();

        let new_main: Gc<Closure> =
            self.make_from(self.modules.last().unwrap().main());

        for i in 0..old_main.capture_count() {
            let index = Index::new(i);
            let cell = old_main.get_capture_cell(index);
            new_main.push_capture_cell(cell);
        }

        let mut frame = self.call_stack.pop();

        frame.pc.saturating_decrement();

        let bp = frame.bp;

        self.call_stack.push(frame);
        self.value_stack.set(bp, Value::from(new_main));

        Ok(())
    }

    /// Resume the runtime. If it hasn't been started before this will also
    /// start it.
    ///
    /// See the implementation note on [`last_result`] for details.
    pub fn resume(&mut self) -> Result<Exit> {
        self.value_stack.pop();
        self.run()
    }

    /// A string containing a representation of the last value on the stack.
    ///
    /// This is useful for the `repl` and `eval` subcommands.
    pub fn last_result(&self) -> std::string::String {
        format!("{:?}", self.value_stack.last())
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
            // TODO: there are probably crazy GC issues here
            Module::destructively_set_up_from_compiler_module(
                live_module,
                module,
                self,
            )
        };

        Ok(())
    }

    pub(crate) fn value_stack(&self) -> &ValueStack {
        &self.value_stack
    }

    /// The base pointer, or the value which indicates where in the stack values
    /// pertaining to the currently executing closure begin.
    ///
    /// The value below the base pointer is the closure that's currently
    /// executing.
    pub fn bp(&self) -> Index<ValueStack> {
        self.call_stack.frame().bp
    }

    /// The program counter is the address of the currently executing piece of
    /// code.
    pub fn pc(&self) -> Index<Op> {
        self.call_stack.frame().pc()
    }

    pub fn op(&self) -> Option<Op> {
        let index = self.pc();
        self.current_closure().get_op(index)
    }

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
        &self.value_stack.as_slice()[start..]
    }

    pub fn current_closure(&self) -> Gc<Closure> {
        let bp = self.bp();
        self.value_stack()
            .get(bp)
            .expect("no current closure")
            .as_gc()
            .expect("bp not pointing to closure")
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
