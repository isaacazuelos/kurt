//! The rules for walking a syntax tree.

use diagnostic::Span;
use syntax::{self, Syntax};

use crate::{
    constant::Constant,
    error::{Error, Result},
    opcode::Op,
    Compiler,
};

impl Compiler {
    /// Compile a module.
    ///
    /// Note that this emits a 'Halt', so you can't really compile more code
    /// meaningfully after this.
    pub fn module(&mut self, syntax: &syntax::Module) -> Result<()> {
        self.statement_sequence(syntax.statements())?;
        self.emit(Op::Halt, syntax.span())
    }

    /// Compile a TopLevel.
    ///
    /// The code is added to main, and finished with a [`Op::Yield`] so the
    /// program can be restarted if more code is added later.
    pub fn top_level(&mut self, syntax: &syntax::TopLevel) -> Result<()> {
        self.statement_sequence(syntax.statements())?;
        self.emit(Op::Yield, syntax.span())
    }

    /// Compile a sequence of statements.
    fn statement_sequence(
        &mut self,
        syntax: &syntax::StatementSequence,
    ) -> Result<()> {
        // each statement with a semicolon gets compiled
        for i in 0..syntax.semicolons().len() {
            self.statement(&syntax.as_slice()[i])?;
            self.emit(Op::Pop, syntax.semicolons()[i])?;
        }

        if syntax.as_slice().is_empty() || syntax.has_trailing() {
            self.emit(Op::Unit, syntax.span())
        } else {
            self.statement(syntax.as_slice().last().unwrap())
        }
    }

    /// Compile a statement.
    ///
    /// Each statement should leave it's resulting value as a new value on the
    /// top of the stack, without consuming anything.
    fn statement(&mut self, syntax: &syntax::Statement) -> Result<()> {
        match syntax {
            syntax::Statement::Binding(b) => self.binding(b),
            syntax::Statement::Empty(span) => self.empty_statement(*span),
            syntax::Statement::Expression(e) => self.expression(e),
        }
    }

    /// Compile a binding statement, something like `let a = b` or `var x = y`.
    fn binding(&mut self, syntax: &syntax::Binding) -> Result<()> {
        if syntax.is_var() {
            return Err(Error::MutationNotSupported);
        }

        self.expression(syntax.body())?;
        self.bind_local(syntax.name());

        // We're keeping this slot on the stack.
        self.emit(Op::DefineLocal, syntax.span())?;

        Ok(())
    }

    /// Compiles an empty statement
    ///
    /// Empty statements have a value of `()`, so we need to push one to the stack.
    fn empty_statement(&mut self, span: Span) -> Result<()> {
        self.emit(Op::Unit, span)
    }

    /// Compile an expression
    fn expression(&mut self, syntax: &syntax::Expression) -> Result<()> {
        match syntax {
            syntax::Expression::Block(b) => self.block(b),
            syntax::Expression::Function(f) => self.function(f),
            syntax::Expression::Identifier(i) => self.identifier_expression(i),
            syntax::Expression::Literal(l) => self.literal(l),
        }
    }

    /// Compile a block expression.
    fn block(&mut self, syntax: &syntax::Block) -> Result<()> {
        self.begin_scope();
        self.statement_sequence(syntax.statements())?;
        self.end_scope();
        Ok(())
    }

    /// Compile a function.
    fn function(&mut self, _syntax: &syntax::Function) -> Result<()> {
        todo!()
    }

    /// Compile a identifier used as an expression.
    ///
    /// For now we only have local variables.
    fn identifier_expression(
        &mut self,
        syntax: &syntax::Identifier,
    ) -> Result<()> {
        let name = syntax.as_str();

        if let Some(index) = self.resolve_local(name) {
            self.emit(Op::LoadLocal(index), syntax.span())
        } else {
            Err(Error::UndefinedLocal)
        }
    }

    /// Compile a literal
    fn literal(&mut self, syntax: &syntax::Literal) -> Result<()> {
        match syntax.kind() {
            syntax::LiteralKind::Binary => self.binary(syntax),
            syntax::LiteralKind::Bool => self.bool(syntax),
            syntax::LiteralKind::Char => self.char(syntax),
            syntax::LiteralKind::Decimal => self.decimal(syntax),
            syntax::LiteralKind::Float => self.float(syntax),
            syntax::LiteralKind::Hexadecimal => self.hexadecimal(syntax),
            syntax::LiteralKind::Keyword => self.keyword(syntax),
            syntax::LiteralKind::Octal => self.octal(syntax),
            syntax::LiteralKind::String => self.string(syntax),
            syntax::LiteralKind::Unit => self.unit(syntax),
        }
    }

    /// Compile an binary numeric literal
    fn binary(&mut self, syntax: &syntax::Literal) -> Result<()> {
        let n = Constant::parse_radix(syntax.body(), 2)?;
        let index = self.constants.insert(n)?;
        self.emit(Op::LoadConstant(index), syntax.span())
    }

    /// Compile a boolean literal.
    fn bool(&mut self, syntax: &syntax::Literal) -> Result<()> {
        let op = if syntax.body() == "true" {
            Op::True
        } else {
            Op::False
        };

        self.emit(op, syntax.span())
    }

    /// Compile a character literal.
    fn char(&mut self, syntax: &syntax::Literal) -> Result<()> {
        let c = Constant::parse_char(syntax.body())?;
        let index = self.constants.insert(c)?;
        self.emit(Op::LoadConstant(index), syntax.span())
    }

    /// Compile a numeric literal
    fn decimal(&mut self, syntax: &syntax::Literal) -> Result<()> {
        let n = Constant::parse_int(syntax.body())?;
        let index = self.constants.insert(n)?;
        self.emit(Op::LoadConstant(index), syntax.span())
    }

    fn float(&mut self, syntax: &syntax::Literal) -> Result<()> {
        let f = Constant::parse_float(syntax.body())?;
        let index = self.constants.insert(f)?;
        self.emit(Op::LoadConstant(index), syntax.span())
    }

    /// Compile an octal numeric literal
    fn octal(&mut self, syntax: &syntax::Literal) -> Result<()> {
        let n = Constant::parse_radix(syntax.body(), 8)?;
        let index = self.constants.insert(n)?;
        self.emit(Op::LoadConstant(index), syntax.span())
    }

    /// Compile a keyword literal
    fn keyword(&mut self, syntax: &syntax::Literal) -> Result<()> {
        let kw = Constant::parse_keyword(syntax.body())?;
        let index = self.constants.insert(kw)?;
        self.emit(Op::LoadConstant(index), syntax.span())
    }

    /// Compile a hexadecimal numeric literal
    fn hexadecimal(&mut self, syntax: &syntax::Literal) -> Result<()> {
        let n = Constant::parse_radix(syntax.body(), 16)?;
        let index = self.constants.insert(n)?;
        self.emit(Op::LoadConstant(index), syntax.span())
    }

    fn string(&mut self, syntax: &syntax::Literal) -> Result<()> {
        let s = Constant::parse_string(syntax.body())?;
        let index = self.constants.insert(s)?;
        self.emit(Op::LoadConstant(index), syntax.span())
    }

    /// Compile a unit literal (i.e. `()`).
    fn unit(&mut self, syntax: &syntax::Literal) -> Result<()> {
        self.emit(Op::Unit, syntax.span())
    }
}
