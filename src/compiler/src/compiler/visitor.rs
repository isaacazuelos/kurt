//! The rules for walking a syntax tree.

use diagnostic::Span;
use syntax::{self, Expression, Sequence, Statement, Syntax};

use crate::{
    constant::Constant,
    error::{Error, Result},
    opcode::Op,
    prototype::Prototype,
    Compiler,
};

impl Compiler {
    /// Compile a [`Module`][syntax::Module].
    ///
    /// Note that this emits a 'Halt', so you can't really compile more code
    /// meaningfully after this.
    pub fn module(&mut self, syntax: &syntax::Module) -> Result<()> {
        let mut main = Prototype::new();
        main.set_name(Prototype::MAIN_NAME);
        self.compiling.push(main);

        self.statement_sequence(syntax)?;

        self.emit(Op::Halt, syntax.span())
    }

    /// Compile a [`TopLevel`][syntax::TopLevel].
    ///
    /// The code is added to main, and finished with a [`Op::Yield`] so the
    /// program can be restarted if more code is added later.
    pub fn top_level(&mut self, syntax: &syntax::TopLevel) -> Result<()> {
        let mut main = Prototype::new();
        main.set_name(Prototype::MAIN_NAME);
        self.compiling.push(main);

        self.statement_sequence(syntax)?;

        self.emit(Op::Yield, syntax.span())
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
            syntax::Statement::If(i) => self.if_only(i),
        }
    }

    /// Compile a sequence of statements.
    ///
    /// If it's empty or if it has a trailing semicolon, a `()` is left on the
    /// stack, otherwise only the last result is.
    fn statement_sequence<'a, S>(&mut self, syntax: &S) -> Result<()>
    where
        S: Sequence<Element = Statement<'a>>,
    {
        // Each separator (and it's statement) gets compiled and popped.
        for i in 0..syntax.separators().len() {
            self.statement(&syntax.elements()[i])?;
            self.emit(Op::Pop, syntax.separators()[i])?;
        }

        if syntax.is_empty() || syntax.has_trailing() {
            // If it's empty, or has a trailing semicolon, we need to return `()`
            self.emit(Op::Unit, syntax.span())
        } else {
            // There might be a statement without a semicolon, in which case we
            // compile it and leave it on the stack.
            self.statement(syntax.elements().last().unwrap())
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

    /// Compile an `if` with no `else` as a statement.
    fn if_only(&mut self, syntax: &syntax::IfOnly) -> Result<()> {
        self.expression(syntax.condition())?;

        let end_span = syntax.block().close();

        let patch_emit_unit = self.next_index();
        self.emit(Op::Nop, end_span)?;

        self.block(syntax.block())?;

        let patch_jump_end = self.next_index();
        self.emit(Op::Nop, end_span)?;

        let emit_unit = self.next_index();
        self.emit(Op::Unit, end_span)?;

        let end = self.next_index();

        self.patch(patch_emit_unit, Op::BranchFalse(emit_unit));
        self.patch(patch_jump_end, Op::Jump(end));

        Ok(())
    }

    /// Compile an expression
    fn expression(&mut self, syntax: &syntax::Expression) -> Result<()> {
        match syntax {
            syntax::Expression::Binary(b) => self.binary(b),
            syntax::Expression::Block(b) => self.block(b),
            syntax::Expression::Call(c) => self.call(c),
            syntax::Expression::Function(f) => self.function(f),
            syntax::Expression::Grouping(g) => self.grouping(g),
            syntax::Expression::Identifier(i) => self.identifier_expression(i),
            syntax::Expression::If(i) => self.if_else(i),
            syntax::Expression::List(l) => self.list(l),
            syntax::Expression::Literal(l) => self.literal(l),
            syntax::Expression::Unary(u) => self.unary(u),
            syntax::Expression::Subscript(s) => self.subscript(s),
        }
    }

    /// Compile a sequence of expressions, leaving each on the stack.
    fn expression_sequence<'a, S>(&mut self, syntax: &S) -> Result<()>
    where
        S: Sequence<Element = Expression<'a>>,
    {
        for arg in syntax.elements() {
            self.expression(arg)?;
        }

        Ok(())
    }

    /// Compile a binary operator expression.
    fn binary(&mut self, syntax: &syntax::Binary) -> Result<()> {
        self.expression(syntax.left())?;
        self.expression(syntax.right())?;

        let op = match syntax.operator() {
            // math
            "+" => Ok(Op::Add),
            "-" => Ok(Op::Sub),
            "*" => Ok(Op::Mul),
            "/" => Ok(Op::Div),
            "^" => Ok(Op::Pow),
            "%" => Ok(Op::Rem),
            // bitwise
            "&" => Ok(Op::BitAnd),
            "|" => Ok(Op::BitOr),
            "⊕" => Ok(Op::BitXOR),
            "<<" => Ok(Op::SHL),
            ">>" => Ok(Op::SHR),
            // comparison
            "==" => Ok(Op::Eq),
            "!=" => Ok(Op::Ne),
            ">" => Ok(Op::Gt),
            ">=" => Ok(Op::Ge),
            "<" => Ok(Op::Lt),
            "<=" => Ok(Op::Le),

            _ => Err(Error::UndefinedInfix),
        }?;

        self.emit(op, syntax.operator_span())
    }

    /// Compile a unary operator expression.
    ///
    /// We want to evaluate left-to-right, and we're not sure if retrieving a
    /// definition of an operator could have side effects, so we'll need to be
    /// careful when this is doing more than compiling to a single op code.
    fn unary(&mut self, syntax: &syntax::Unary) -> Result<()> {
        // This is mostly temporary until a real built-ins system is in place.
        self.expression(syntax.operand())?;

        let span = syntax.operator_span();
        if syntax.is_prefix() {
            match syntax.operator() {
                "!" => self.emit(Op::Not, span),
                "-" => self.emit(Op::Neg, span),
                "+" => Ok(()),
                _ => Err(Error::UndefinedPostfix),
            }
        } else {
            match syntax.operator() {
                // None defined, yet.
                _ => Err(Error::UndefinedPostfix),
            }
        }
    }

    /// Compile a block expression.
    fn block(&mut self, syntax: &syntax::Block) -> Result<()> {
        self.with_scope(|compiler| compiler.statement_sequence(syntax))
    }

    /// Compile a function call.
    fn call(&mut self, syntax: &syntax::Call) -> Result<()> {
        self.expression(syntax.target())?;

        self.expression_sequence(syntax)?;

        let count = syntax.elements().len();
        if count >= u32::MAX as usize {
            Err(Error::TooManyArguments)
        } else {
            self.emit(Op::Call(count as u32), syntax.open() + syntax.close())
        }
    }

    /// Compile a function.
    fn function(&mut self, syntax: &syntax::Function) -> Result<()> {
        let i = self.with_prototype(|compiler| {
            if syntax.elements().len() > u32::MAX as usize {
                return Err(Error::TooManyParameters);
            }

            compiler
                .active_prototype_mut()
                .set_parameter_count(syntax.elements().len() as u32);

            for parameter in syntax.elements() {
                compiler.bind_local(parameter.name());
            }

            compiler.expression(syntax.body())?;
            compiler.emit(Op::Return, syntax.body().span())
        })?;

        self.emit(Op::LoadClosure(i), syntax.span())
    }

    /// Compile an expression wrapped in parens.
    fn grouping(&mut self, syntax: &syntax::Grouping) -> Result<()> {
        self.expression(syntax.body())
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

    /// Compile an `if` with and `else`.
    fn if_else(&mut self, syntax: &syntax::IfElse) -> Result<()> {
        self.expression(syntax.condition())?;

        let patch_branch_false = self.next_index();
        self.emit(Op::Nop, syntax.else_span())?;

        self.block(syntax.true_block())?;

        let patch_jump_end = self.next_index();
        self.emit(Op::Nop, syntax.false_block().close())?;

        let false_start = self.next_index();

        self.block(syntax.false_block())?;

        let end = self.next_index();

        self.patch(patch_branch_false, Op::BranchFalse(false_start));
        self.patch(patch_jump_end, Op::Jump(end));

        Ok(())
    }

    /// Compile a list literal
    fn list(&mut self, syntax: &syntax::List) -> Result<()> {
        self.expression_sequence(syntax)?;
        self.emit(Op::List(syntax.elements().len() as u32), syntax.span())
    }

    fn subscript(&mut self, syntax: &syntax::Subscript) -> Result<()> {
        self.expression(syntax.target())?;
        self.expression(syntax.index())?;
        let span = syntax.open() + syntax.close();
        self.emit(Op::Subscript, span)
    }

    /// Compile a literal
    fn literal(&mut self, syntax: &syntax::Literal) -> Result<()> {
        match syntax.kind() {
            syntax::LiteralKind::Binary => self.binary_literal(syntax),
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
    fn binary_literal(&mut self, syntax: &syntax::Literal) -> Result<()> {
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
