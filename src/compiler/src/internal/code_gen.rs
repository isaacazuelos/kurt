//! The rules for walking a syntax tree.

use common::Index;
use diagnostic::Span;
use syntax::{self, Expression, Sequence, Statement, Syntax};

use crate::{
    constant::Constant,
    error::{Error, Result},
    internal::ModuleBuilder,
    opcode::Op,
    Function,
};

use super::module::PatchObligation;
impl ModuleBuilder {
    /// Compile a sequence of statements.
    ///
    /// If it's empty or if it has a trailing semicolon, a `()` is left on the
    /// stack, otherwise only the last result is.
    ///
    /// ```text
    ///   <s_1>
    ///   Pop
    ///   <s_2>
    ///   Pop
    ///   ...
    ///   <s_n>
    /// [ Pop  ]  // trailing semicolon
    /// [ Unit ]  // if trailing semicolon or no statements
    /// ```
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
    ///
    /// ```text
    ///   DefineLocal
    /// ```
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
    /// Empty statements have a value of `()`, so we need to push one to the
    /// stack.
    ///
    /// ``` text
    ///   Unit
    /// ```
    fn empty_statement(&mut self, span: Span) -> Result<()> {
        self.emit(Op::Unit, span)
    }

    /// Compile an `if` with no `else` as a statement.
    ///
    /// Compile an `if` with no `else` as a statement.
    ///
    /// ``` text
    ///   <condition>
    ///   BranchFalse(end)
    ///   <block>
    /// end:
    ///     ...
    /// ```
    fn if_only(&mut self, syntax: &syntax::IfOnly) -> Result<()> {
        self.expression(syntax.condition())?;

        let end_span = syntax.block().close();

        let jump_false = self.new_patch_obligation(end_span)?;
        self.block(syntax.block())?;

        let end = self.next_op(end_span)?;
        let to_end = jump_distance(jump_false, end, syntax.span())?;
        self.patch(jump_false, Op::BranchFalse(to_end));

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
            syntax::Expression::Loop(l) => self.loop_loop(l),
            syntax::Expression::Literal(l) => self.literal(l),
            syntax::Expression::Unary(u) => self.unary(u),
            syntax::Expression::While(w) => self.while_loop(w),
            syntax::Expression::Subscript(s) => self.subscript(s),
        }
    }

    /// Compile a sequence of expressions, leaving each on the stack.
    ///
    /// ```text
    ///     <e_1>
    ///     <e_2>
    ///     ...
    ///     <e_n>
    /// ```
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
    ///
    /// We have to special case `and` and `or`, but otherwise the byte code
    /// emitted is the following, when `op` is one of the built-in operations
    ///
    /// ``` text
    ///   <left>
    ///   <right>
    ///   <op>
    /// ```
    fn binary(&mut self, syntax: &syntax::Binary) -> Result<()> {
        if matches!(syntax.operator(), "and" | "or") {
            return self.short_circuiting(syntax);
        }

        if matches!(syntax.operator(), "=") {
            return self.assignment(syntax);
        }

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

    /// Compiles an assignment expression.
    ///
    /// ```text
    ///   <left>
    ///   <right>
    ///   <set_op>
    /// ```
    fn assignment(&mut self, syntax: &syntax::Binary) -> Result<()> {
        let set_op = self.assignment_target(syntax.left())?;
        self.expression(syntax.right())?;
        self.emit(set_op, syntax.operator_span())
    }

    /// Compiles the left hand side of an assignment
    fn assignment_target(&mut self, syntax: &syntax::Expression) -> Result<Op> {
        match syntax {
            Expression::Subscript(s) => self.assignment_target_subscript(s),
            Expression::Identifier(i) => self.assignment_target_identifier(i),

            Expression::Binary(_)
            | Expression::Block(_)
            | Expression::Call(_)
            | Expression::EarlyExit(_)
            | Expression::Function(_)
            | Expression::Grouping(_)
            | Expression::If(_)
            | Expression::List(_)
            | Expression::Literal(_)
            | Expression::Loop(_)
            | Expression::Unary(_)
            | Expression::While(_) => {
                Err(Error::NotALegalAssignmentTarget(syntax.span()))
            }
        }
    }

    /// Assignment to an identifier
    ///
    /// ```text
    ///   SetLocal
    /// ```
    fn assignment_target_identifier(
        &mut self,
        syntax: &syntax::Identifier,
    ) -> Result<Op> {
        if let Some(local) = self.resolve_local(syntax.as_str()) {
            Ok(Op::SetLocal(local))
        } else if let Some(capture) = self.resolve_capture(syntax)? {
            Ok(Op::SetCapture(capture))
        } else {
            Err(Error::UndefinedLocal(syntax.span()))
        }
    }

    /// Assignment to an identifier
    ///
    /// ```text
    ///   <target>
    ///   <key>
    ///   <new value>
    ///   SetIndex
    /// ```
    fn assignment_target_subscript(
        &mut self,
        syntax: &syntax::Subscript,
    ) -> Result<Op> {
        self.expression(syntax.target())?;
        self.expression(syntax.index())?;
        Ok(Op::SetIndex)
    }

    /// An `and` or `or` infix operator.
    ///
    /// ``` text
    ///   <left>
    /// [ Branch(end)] // if `or`
    /// [ BranchFalse(end)] // if `and`
    ///   <right>
    /// end:
    ///   ...
    /// ```
    fn short_circuiting(&mut self, syntax: &syntax::Binary) -> Result<()> {
        let op_span = syntax.operator_span();
        self.expression(syntax.left())?;

        let branch = self.new_patch_obligation(op_span)?;

        self.expression(syntax.right())?;

        let end = self.next_op(op_span)?;

        let distance = jump_distance(branch, end, syntax.span())?;

        let op = if syntax.operator() == "and" {
            Op::BranchFalse(distance)
        } else {
            Op::Branch(distance)
        };

        self.patch(branch, op);
        Ok(())
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
        if let Some(expression) = syntax.expression() {
            self.expression(expression)?;
        } else {
            self.emit(Op::Unit, syntax.span())?;
        }

        match syntax.kind() {
            syntax::ExitKind::Return => self.emit(Op::Return, syntax.span()),

            syntax::ExitKind::Break => {
                let jump = self.new_patch_obligation(syntax.span())?;
                self.current_function_mut().register_break(jump)
            }

            syntax::ExitKind::Continue => {
                // We allow this in the parser so it matches the other early
                // exits which can have values, but it's not legal -- what would
                // it mean?
                if let Some(invalid) = syntax.expression() {
                    return Err(Error::ContinueWithValue(invalid.span()));
                }

                let jump = self.new_patch_obligation(syntax.span())?;
                self.current_function_mut().register_continue(jump)
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
    ///
    /// ``` text
    ///   <condition>
    ///   BranchFalse(start_of_else_block)
    ///   <true_block>
    ///   Jump(end)
    /// start_of_else_block:
    ///   Pop
    ///   <false branch>;
    /// end:
    ///    ...
    /// ```
    fn if_else(&mut self, syntax: &syntax::IfElse) -> Result<()> {
        self.expression(syntax.condition())?;

        let branch_false =
            self.new_patch_obligation(syntax.false_block().span())?;

        self.block(syntax.true_block())?;

        let jump_to_end =
            self.new_patch_obligation(syntax.true_block().close())?;

        let start_of_else_block = self.next_op(syntax.false_block().open())?;

        self.emit(Op::Pop, syntax.false_block().open())?;
        self.block(syntax.false_block())?;

        let end = self.next_op(syntax.false_block().close())?;

        let to_else =
            jump_distance(branch_false, start_of_else_block, syntax.span())?;
        self.patch(branch_false, Op::BranchFalse(to_else));

        let to_end = jump_distance(jump_to_end, end, syntax.span())?;
        self.patch(jump_to_end, Op::Jump(to_end));

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

    /// Compile a `loop` loop.
    ///
    /// ``` text
    /// top:
    ///   <body>
    ///   Jump(top)
    /// ```
    fn loop_loop(&mut self, syntax: &syntax::Loop) -> Result<()> {
        let top = self.next_op(syntax.loop_span())?;

        self.current_function_mut().begin_loop();
        self.block(syntax.body())?;

        let jump = self.new_patch_obligation(syntax.body().close())?;
        let distance = jump_distance(jump, top, syntax.span())?;
        self.patch(jump, Op::Jump(distance));

        let end = self.next_op(syntax.body().close())?;

        self.current_function_mut().end_loop(
            top,
            end,
            syntax.body().open(),
            syntax.body().close(),
        )
    }

    /// Compile a `while` loop.
    ///
    /// ``` text
    ///   Unit               // while's value is () if it don't loop
    /// top:
    ///   <condition>
    ///   BranchFalse(condition_false)
    ///   Pop                // remove last value
    ///   <body>
    ///   Jump(top)
    /// condition_false:
    ///   Pop                // remove the false that left the loop
    /// end:           //
    ///   ...
    /// ```
    fn while_loop(&mut self, syntax: &syntax::While) -> Result<()> {
        let condition_span = syntax.condition().span();

        self.emit(Op::Unit, condition_span)?;
        let top = self.next_op(condition_span)?;

        self.expression(syntax.condition())?;

        let condition_false_jump = self.new_patch_obligation(condition_span)?;

        self.emit(Op::Pop, condition_span)?;

        self.current_function_mut().begin_loop();
        self.block(syntax.body())?;

        let jump = self.new_patch_obligation(syntax.while_span())?;
        let to_top = jump_distance(jump, top, syntax.span())?;
        self.patch(jump, Op::Jump(to_top));

        let condition_false = self.next_op(syntax.body().close())?;
        let to_condition_false = jump_distance(
            condition_false_jump,
            condition_false,
            syntax.condition().span(),
        )?;

        self.patch(condition_false_jump, Op::BranchFalse(to_condition_false));

        self.emit(Op::Pop, condition_span)?;

        let end = self.next_op(syntax.body().close())?;

        // close off any break and continue statements
        self.current_function_mut().end_loop(
            top,
            end,
            syntax.body().open(),
            syntax.body().close(),
        )?;

        Ok(())
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

pub(crate) fn jump_distance(
    jump_instruction: Index<PatchObligation>,
    target: Index<Op>,
    span: Span,
) -> Result<i32> {
    let start = jump_instruction.as_usize() as isize;
    let end = target.as_usize() as isize;

    let distance = end - start;

    if distance <= i32::MAX as isize && distance >= i32::MIN as isize {
        Ok(distance as i32)
    } else {
        Err(Error::JumpTooFar(span))
    }
}
