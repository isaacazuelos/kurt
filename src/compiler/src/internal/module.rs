use common::{Get, Index};
use diagnostic::{Diagnostic, InputId, Span};
use syntax::{Identifier, Syntax};

use crate::{
    error::Error,
    internal::{ConstantPool, FunctionBuilder},
    Capture, Constant, Export, Function, Local, Module, Op,
};

pub struct ModuleBuilder {
    /// The input that was used to produce this code.
    id: Option<InputId>,

    /// The constant pool of all constants seen by this compiler so far.
    constants: ConstantPool,

    /// A stack of currently compiling functions. Once completed, they're moved
    /// to `functions`.
    compiling: Vec<FunctionBuilder>,

    /// Code is compiled into [`Function`]s which are kept here once complete.
    functions: Vec<Function>,

    /// Exported values
    exports: Vec<Export>,
}

impl Default for ModuleBuilder {
    fn default() -> Self {
        let mut compiler = Self {
            id: None,
            constants: Default::default(),
            compiling: Default::default(),
            functions: Default::default(),
            exports: Default::default(),
        };

        compiler.prime();

        compiler
    }
}

impl ModuleBuilder {
    pub const MAIN_NAME: &'static str = "main";

    const MAIN: usize = 0;

    /// Convert the current compiler state into a new [`Module`] that can be
    /// loaded into the runtime.
    pub fn build(&self) -> Module {
        debug_assert_eq!(
            self.compiling.len(),
            1,
            "only 'main' should be left compiling."
        );

        let mut functions = self.functions.clone();

        let main = self.compiling[ModuleBuilder::MAIN].build_as_main();

        functions[Module::MAIN.as_usize()] = main;

        let exports = self.exports.iter().map(|e| e.name().into()).collect();

        Module {
            input: None,
            constants: self.constants.as_vec(),
            functions,
            exports,
        }
    }

    pub fn id(&self) -> Option<InputId> {
        self.id
    }

    pub fn with_id(mut self, id: Option<InputId>) -> Self {
        self.id = id;
        self
    }

    pub fn set_id(&mut self, id: InputId) {
        self.id = Some(id);
    }

    /// Push some input through the module builder.
    ///
    /// This behaves the same way as [`ModuleBuilder::syntax`], but does the
    /// parsing, and returns [`Diagnostic`] instead of [`Error`] so that parsing
    /// errors can be captured as well.
    pub fn input(mut self, input: &str) -> Result<Self, Diagnostic> {
        self.push_input(input)?;
        Ok(self)
    }

    /// Push some syntax through the module builder.
    ///
    /// If you call this multiple times with one builder, successive calls will
    /// _add_ to the end of the resulting module.
    ///
    /// This consumes the builder on failure -- if this isn't what you need, you
    /// should use [`ModuleBuilder::push_syntax`] instead.
    pub fn syntax(mut self, syntax: &syntax::Module) -> Result<Self, Error> {
        self.push_syntax(syntax)?;
        Ok(self)
    }

    /// Push some input through the module builder.
    pub fn push_input(&mut self, input: &str) -> Result<(), Diagnostic> {
        use syntax::Parse;
        let syntax = syntax::Module::parse(input)?;
        self.push_syntax(&syntax)?;
        Ok(())
    }

    /// Push some syntax through the module builder, concatenating the
    /// statements to the end of the existing module's code.
    ///
    /// If an error is returned, the module should (hopefully!) be back in a
    /// state where it's safe to use.
    ///
    /// # Note
    ///
    /// See the note on [`syntax::Module`] about it being use for all top-level
    /// code, for now.
    pub fn push_syntax(
        &mut self,
        syntax: &syntax::Module,
    ) -> Result<(), Error> {
        debug_assert_eq!(
            self.compiling.len(),
            1,
            "only 'main' should be left compiling."
        );

        let backup = self.compiling[0].clone();
        let old_function_count = self.functions.len();
        let old_constant_count = self.constants.len();

        if let Err(e) = self.statement_sequence(syntax) {
            // We need to recover on failure before we can return the error.

            self.compiling[0] = backup;

            self.compiling.truncate(1);
            self.functions.truncate(old_function_count);
            self.constants.truncate(old_constant_count);

            Err(e)
        } else {
            Ok(())
        }
    }

    pub(crate) fn get_export_mut(
        &mut self,
        index: Index<Export>,
    ) -> &mut Export {
        &mut self.exports[index.as_usize()]
    }
}

impl ModuleBuilder {
    /// Prime the compiler by defining a 'main' function at index zero that just
    /// halts.
    fn prime(&mut self) {
        debug_assert!(
            self.compiling.is_empty(),
            "the builder should only be primed once, when made"
        );

        let mut main = FunctionBuilder::new(Span::default());

        let main_name = self.insert_constant(Self::MAIN_NAME);

        main.set_name(main_name);

        self.compiling.push(main);

        // We push also need to push a placeholder into the finished functions,
        // to keep the indexes aligned, and reserve teh spot for the final
        // 'main' to go.
        self.functions.push(Function::default());
    }
}

// Helpers used by the visitors
impl ModuleBuilder {
    /// Insert a new constant into the constant pool,
    pub(crate) fn insert_constant(
        &mut self,
        constant: impl Into<Constant>,
    ) -> Option<Index<Constant>> {
        self.constants.insert(constant)
    }

    /// This is a shorthand for emitting to the currently compiling function.
    pub(crate) fn emit(&mut self, op: Op, span: Span) -> Result<(), Error> {
        self.current_function_mut().emit(op, span)
    }

    /// Get a reference to the currently-compiling function.
    pub(crate) fn current_function(&self) -> &FunctionBuilder {
        self.compiling.last().unwrap()
    }

    /// Get a mutable reference to the currently-compiling function.
    pub(crate) fn current_function_mut(&mut self) -> &mut FunctionBuilder {
        self.compiling.last_mut().unwrap()
    }

    /// Is the module compiling it's top-level code in it's `main`?
    pub(crate) fn on_main_top_level(&self) -> bool {
        (self.compiling.len() == 1) && (self.compiling[0].on_top_level())
    }
}

pub(crate) struct PatchObligation;

// Bindings and scopes
impl ModuleBuilder {
    pub(crate) fn next_op(&self, span: Span) -> Result<Index<Op>, Error> {
        self.compiling
            .last()
            .unwrap()
            .code()
            .next_index()
            .ok_or_else(|| Error::TooManyOps(span))
    }

    pub(crate) fn patch(
        &mut self,
        obligation: Index<PatchObligation>,
        op: Op,
    ) -> Option<Op> {
        self.current_function_mut().code_mut().patch(obligation, op)
    }

    pub(crate) fn new_patch_obligation(
        &mut self,
        span: Span,
    ) -> Result<Index<PatchObligation>, Error> {
        let index = self.next_op(span)?;
        self.emit(Op::Nop, span)?;
        Ok(Index::new(index.into()))
    }

    pub(crate) fn with_scope<F, T>(
        &mut self,
        inner: F,
        span: Span,
    ) -> Result<T, Error>
    where
        F: FnOnce(&mut ModuleBuilder) -> Result<T, Error>,
    {
        self.current_function_mut().begin_scope();

        let result = inner(self);

        self.current_function_mut().end_scope(span)?;

        result
    }

    pub(crate) fn begin_function(&mut self, span: Span) -> Result<(), Error> {
        if self.compiling.len() + self.functions.len() >= u32::MAX as usize {
            return Err(Error::TooManyFunctions(span));
        }

        self.compiling.push(FunctionBuilder::new(span));

        Ok(())
    }

    pub(crate) fn end_function(&mut self) -> Result<Index<Function>, Error> {
        let builder = self.compiling.pop().unwrap();

        let function = builder.build();

        if self.functions.len() >= Module::MAX_FUNCTIONS {
            Err(Error::TooManyFunctions(function.span()))
        } else {
            self.functions.push(function);
            Ok(Index::new((self.functions.len() - 1) as u32))
        }
    }

    pub(crate) fn bind_local(
        &mut self,
        id: &Identifier,
        var: bool,
    ) -> Result<(), Error> {
        self.current_function_mut().bind_local(Local::new(
            id.as_str(),
            id.span(),
            var,
        ))
    }

    pub(crate) fn bind_export(
        &mut self,
        id: &Identifier,
    ) -> Result<Index<Export>, Error> {
        debug_assert!(
            self.on_main_top_level(),
            "new exports can only be bound from the top level of main"
        );

        // you cannot shadow exported values
        if let Some(index) = self.resolve_export(id.as_str()) {
            let previous = self.exports[index.as_usize()].span();
            return Err(Error::ShadowExport(id.span(), previous));
        }

        let index = self.exports.len();
        if index >= Index::<Export>::MAX as usize {
            return Err(Error::TooManyExports(id.span()));
        }

        self.exports.push(Export::new(id.as_str(), id.span()));

        Ok(Index::new(index as u32))
    }

    pub(crate) fn resolve_local(&mut self, name: &str) -> Option<Index<Local>> {
        self.current_function_mut().resolve_local(name)
    }

    pub(crate) fn resolve_export(
        &mut self,
        name: &str,
    ) -> Option<Index<Export>> {
        for (i, e) in self.exports.iter().enumerate() {
            if e.name() == name {
                // won't overflow, as bind_export keeps us under the max
                return Some(Index::new(i as u32));
            }
        }

        None
    }

    pub(crate) fn resolve_recursive(&self, syntax: &Identifier) -> bool {
        if !self.current_function().is_recursive() {
            return false;
        }

        if let Some(name_index) = self.current_function().name() {
            match self.constants.get(name_index) {
                Some(Constant::String(name)) => {
                    name.as_str() == syntax.as_str()
                }
                _ => false,
            }
        } else {
            false
        }
    }

    pub(crate) fn resolve_capture(
        &mut self,
        syntax: &syntax::Identifier,
    ) -> Result<Option<Index<Capture>>, Error> {
        if let Some((current, enclosing)) = self.compiling.split_last_mut() {
            current.resolve_capture(syntax.as_str(), syntax.span(), enclosing)
        } else {
            Ok(None)
        }
    }
}
