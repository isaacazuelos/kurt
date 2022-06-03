//! The rules for walking a syntax tree.

use diagnostic::Span;
use syntax::{self, Expression, Sequence, Statement, Syntax};

use crate::{
    constant::Constant,
    error::{Error, Result},
    internal::ModuleBuilder,
    opcode::Op,
    Function,
};
impl ModuleBuilder {
    /// Compile a sequence of statements.
    ///
    /// If it's empty or if it has a trailing semicolon, a `()` is left on the
    /// stack, otherwise only the last result is.
    pub(crate) fn statement_sequence<'a, S>(&mut self, syntax: &S) -> Result<()>
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

    /// Compile a binding statement, something like `let a = b` or `var x = y`.
    fn binding(&mut self, syntax: &syntax::Binding) -> Result<()> {
        if syntax.is_var() {
            return Err(Error::MutationNotSupported(syntax.keyword().span()));
        }

        let name = syntax.name();

        // if it's a function, we want to let that function know it's name.
        if let syntax::Expression::Function(f) = syntax.body() {
            self.function(f, Some(name), syntax.is_rec())?;
        } else if let Some(rec_span) = syntax.rec() {
            return Err(Error::RecNotFunction(rec_span, syntax.body().span()));
        } else {
            self.expression(syntax.body())?;
        }

        self.bind_local(name)?;

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
            syntax::Expression::EarlyExit(e) => self.early_exit(e),
            syntax::Expression::Function(f) => self.function(f, None, false),
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

            _ => Err(Error::UndefinedInfix(syntax.operator_span())),
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
                _ => Err(Error::UndefinedPostfix(span)),
            }
        } else {
            // None defined, yet.
            Err(Error::UndefinedPostfix(span))
        }
    }

    /// Compile a block expression.
    fn block(&mut self, syntax: &syntax::Block) -> Result<()> {
        self.with_scope(
            |compiler| compiler.statement_sequence(syntax),
            syntax.span(),
        )
    }

    /// Compile a function call.
    fn call(&mut self, syntax: &syntax::Call) -> Result<()> {
        self.expression(syntax.target())?;

        self.expression_sequence(syntax)?;

        let count = syntax.elements().len();
        if count >= Function::MAX_ARGUMENTS {
            let problem_arg = &syntax.elements()[u32::MAX as usize - 1];
            Err(Error::TooManyArguments(problem_arg.span()))
        } else {
            self.emit(Op::Call(count as u32), syntax.open() + syntax.close())
        }
    }

    /// Compile an early exit expression.
    fn early_exit(&mut self, syntax: &syntax::EarlyExit) -> Result<()> {
        match syntax.kind() {
            syntax::ExitKind::Return => {
                if let Some(expression) = syntax.expression() {
                    self.expression(expression)?;
                } else {
                    self.emit(Op::Unit, syntax.span())?;
                }

                self.emit(Op::Return, syntax.span())
            }
            _ => Err(Error::EarlyExitKindNotSupported(syntax.span())),
        }
    }

    /// Compile a function, including it's name if known.
    fn function(
        &mut self,
        syntax: &syntax::Function,
        name: Option<&syntax::Identifier>,
        recursive: bool,
    ) -> Result<()> {
        self.begin_function(syntax.span())?;

        // with that new function as the target of compilation
        {
            self.current_function_mut().set_recursive(recursive);

            let parameter_count = syntax.elements().len();

            if parameter_count > Function::MAX_PARAMETERS {
                let problem_element = &syntax.elements()[u32::MAX as usize];
                return Err(Error::TooManyParameters(problem_element.span()));
            }

            self.current_function_mut()
                .set_parameter_count(parameter_count as u32);

            if let Some(name) = name {
                let index = self.insert_constant(name.as_str());
                self.current_function_mut().set_name(index);
            }

            for parameter in syntax.elements() {
                self.bind_local(parameter.name())?;
            }

            self.expression(syntax.body())?;
            self.emit(Op::Return, syntax.body().span())?;
        }

        let i = self.end_function()?;

        self.emit(Op::LoadFunction(i), syntax.span())
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
        if let Some(index) = self.resolve_local(syntax.as_str()) {
            self.emit(Op::LoadLocal(index), syntax.span())
        } else if self.resolve_recursive(syntax) {
            self.emit(Op::LoadSelf, syntax.span())
        } else if let Some(index) = self.resolve_capture(syntax)? {
            self.emit(Op::LoadCapture(index), syntax.span())
        } else {
            Err(Error::UndefinedLocal(syntax.span()))
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

    /// Compile a subscript postfix
    fn subscript(&mut self, syntax: &syntax::Subscript) -> Result<()> {
        self.expression(syntax.target())?;
        self.expression(syntax.index())?;
        let span = syntax.open() + syntax.close();
        self.emit(Op::Index, span)
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
        let n = Constant::parse_radix(syntax.body(), 2)
            .map_err(|e| Error::ParseInt(syntax.span(), e))?;
        self.emit(Op::U48(n), syntax.span())
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
        let c = Constant::parse_char(syntax.body())
            .map_err(|_| Error::ParseChar(syntax.span()))?;
        let index = self
            .insert_constant(c)
            .ok_or_else(|| Error::TooManyConstants(syntax.span()))?;
        self.emit(Op::LoadConstant(index), syntax.span())
    }

    /// Compile a numeric literal
    fn decimal(&mut self, syntax: &syntax::Literal) -> Result<()> {
        let n = Constant::parse_int(syntax.body())
            .map_err(|e| Error::ParseInt(syntax.span(), e))?;
        self.emit(Op::U48(n), syntax.span())
    }

    fn float(&mut self, syntax: &syntax::Literal) -> Result<()> {
        let f = Constant::parse_float(syntax.body())
            .map_err(|_| Error::ParseFloat(syntax.span()))?;
        let index = self
            .insert_constant(f)
            .ok_or_else(|| Error::TooManyConstants(syntax.span()))?;
        self.emit(Op::LoadConstant(index), syntax.span())
    }

    /// Compile an octal numeric literal
    fn octal(&mut self, syntax: &syntax::Literal) -> Result<()> {
        let n = Constant::parse_radix(syntax.body(), 8)
            .map_err(|e| Error::ParseInt(syntax.span(), e))?;
        self.emit(Op::U48(n), syntax.span())
    }

    /// Compile a keyword literal
    fn keyword(&mut self, syntax: &syntax::Literal) -> Result<()> {
        let kw = Constant::parse_keyword(syntax.body());
        let index = self
            .insert_constant(kw)
            .ok_or_else(|| Error::TooManyConstants(syntax.span()))?;
        self.emit(Op::LoadConstant(index), syntax.span())
    }

    /// Compile a hexadecimal numeric literal
    fn hexadecimal(&mut self, syntax: &syntax::Literal) -> Result<()> {
        let n = Constant::parse_radix(syntax.body(), 16)
            .map_err(|e| Error::ParseInt(syntax.span(), e))?;
        self.emit(Op::U48(n), syntax.span())
    }

    fn string(&mut self, syntax: &syntax::Literal) -> Result<()> {
        let s = Constant::parse_string(syntax.body())?;
        let index = self
            .insert_constant(s)
            .ok_or_else(|| Error::TooManyConstants(syntax.span()))?;
        self.emit(Op::LoadConstant(index), syntax.span())
    }

    /// Compile a unit literal (i.e. `()`).
    fn unit(&mut self, syntax: &syntax::Literal) -> Result<()> {
        self.emit(Op::Unit, syntax.span())
    }
}
