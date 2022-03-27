//! The rules for walking a syntax tree.

use syntax::{ast, Syntax};

use crate::{
    constant::Constant,
    error::{Error, Result},
    opcode::Op,
    Compiler,
};

impl Compiler {
    /// Compile a module.
    ///
    /// Note this is one of the few `pub` entry points into the compiler for
    /// various syntax.
    pub fn module(&mut self, syntax: &syntax::Module) -> Result<()> {
        for (i, statement) in syntax.statements().iter().enumerate() {
            self.statement(statement)?;

            // If we have a statement after this one, we want to pop this result
            // off the stack.
            if i < syntax.semicolons().len() {
                let span = syntax.semicolons()[i];
                self.emit(Op::Pop, span)?;
            }
        }

        Ok(())
    }

    /// Compile a statement.
    fn statement(&mut self, syntax: &ast::Statement) -> Result<()> {
        match syntax {
            ast::Statement::Binding(b) => self.binding(b),
            ast::Statement::Empty(_) => self.empty_statement(),
            ast::Statement::Expression(e) => self.expression(e),
        }
    }

    /// Compile a binding statement, something like `let a = b` or `var x = y`.
    fn binding(&mut self, syntax: &ast::Binding) -> Result<()> {
        if syntax.is_var() {
            return Err(Error::MutationNotSupported);
        }

        self.expression(syntax.body())?;
        self.bind_local(syntax.name());

        // We're keeping this slot on the stack.
        self.emit(Op::Unit, syntax.span())?;

        Ok(())
    }

    /// Compiles an empty statement
    ///
    /// For now this does nothing, mostly for completeness.
    fn empty_statement(&mut self) -> Result<()> {
        Ok(())
    }

    /// Compile an expression
    fn expression(&mut self, syntax: &ast::Expression) -> Result<()> {
        match syntax {
            ast::Expression::Identifier(i) => self.identifier_expression(i),
            ast::Expression::Literal(l) => self.literal(l),
        }
    }

    /// Compile a identifier used as an expression.
    ///
    /// For now we only have local variables.
    fn identifier_expression(
        &mut self,
        syntax: &ast::Identifier,
    ) -> Result<()> {
        if let Some(index) = self.lookup_local(syntax) {
            self.emit(Op::LoadLocal(index), syntax.span())
        } else {
            Err(Error::UndefinedLocal)
        }
    }

    /// Compile a literal
    fn literal(&mut self, syntax: &ast::Literal) -> Result<()> {
        match syntax.kind() {
            ast::LiteralKind::Binary => self.binary(syntax),
            ast::LiteralKind::Bool => self.bool(syntax),
            ast::LiteralKind::Char => self.char(syntax),
            ast::LiteralKind::Decimal => self.decimal(syntax),
            ast::LiteralKind::Float => self.float(syntax),
            ast::LiteralKind::Hexadecimal => self.hexadecimal(syntax),
            ast::LiteralKind::Octal => self.octal(syntax),
            ast::LiteralKind::String => self.string(syntax),
            ast::LiteralKind::Unit => self.unit(syntax),
        }
    }

    /// Compile an binary numeric literal
    fn binary(&mut self, syntax: &ast::Literal) -> Result<()> {
        let n = Constant::parse_radix(syntax.body(), 2)?;
        let index = self.constants.insert(n)?;
        self.emit(Op::LoadConstant(index), syntax.span())
    }

    /// Compile a boolean literal.
    fn bool(&mut self, syntax: &ast::Literal) -> Result<()> {
        let op = if syntax.body() == "true" {
            Op::True
        } else {
            Op::False
        };

        self.emit(op, syntax.span())
    }

    /// Compile a character literal.
    fn char(&mut self, syntax: &ast::Literal) -> Result<()> {
        let c = Constant::parse_char(syntax.body())?;
        let index = self.constants.insert(c)?;
        self.emit(Op::LoadConstant(index), syntax.span())
    }

    /// Compile a numeric literal
    fn decimal(&mut self, syntax: &ast::Literal) -> Result<()> {
        let n = Constant::parse_int(syntax.body())?;
        let index = self.constants.insert(n)?;
        self.emit(Op::LoadConstant(index), syntax.span())
    }

    fn float(&mut self, syntax: &ast::Literal) -> Result<()> {
        let f = Constant::parse_float(syntax.body())?;
        let index = self.constants.insert(f)?;
        self.emit(Op::LoadConstant(index), syntax.span())
    }

    /// Compile an octal numeric literal
    fn octal(&mut self, syntax: &ast::Literal) -> Result<()> {
        let n = Constant::parse_radix(syntax.body(), 8)?;
        let index = self.constants.insert(n)?;
        self.emit(Op::LoadConstant(index), syntax.span())
    }

    /// Compile a hexadecimal numeric literal
    fn hexadecimal(&mut self, syntax: &ast::Literal) -> Result<()> {
        let n = Constant::parse_radix(syntax.body(), 16)?;
        let index = self.constants.insert(n)?;
        self.emit(Op::LoadConstant(index), syntax.span())
    }

    fn string(&mut self, syntax: &ast::Literal) -> Result<()> {
        let s = Constant::parse_string(syntax.body())?;
        let index = self.constants.insert(s)?;
        self.emit(Op::LoadConstant(index), syntax.span())
    }

    /// Compile a unit literal (i.e. `()`).
    fn unit(&mut self, syntax: &ast::Literal) -> Result<()> {
        self.emit(Op::Unit, syntax.span())
    }
}

#[cfg(test)]
mod tests {
    // Not really anything to test as it's all just tree walking at this point.
}
