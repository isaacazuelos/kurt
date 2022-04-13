//! An intermediate representation which consists of only the 'core' parts of
//! the language.
//!
//! This lets implement some nicer parts of the syntax in a way that doesn't
//! cause the compiler to become more complex.
//!
//! # A note on `⟦ ⟧` notation
//!
//! In a few places I've (very informally) used `⟦ e ⟧ = ...` syntax to mean
//! 'translate `e`' by rewriting as `...`. If you see the `⟦ ⟧` brackets on the
//! right of the `=`, it means we continue recursively translating there.
//!
//! I haven't used this notation everywhere, just where there's something
//! complicated going on.

use std::{fmt, ops::Deref};

use syntax::*;

mod constant;
mod id;

use crate::constant::Constant;
use crate::id::Id;

/// An intermediate representation used to simplify compilation.
#[derive(Debug)]
pub enum Core {
    /// `let id = body in expr`
    ///
    /// Bind a value to a name, and every binding is scoped!
    Bind(Id, Box<(Core, Core)>),

    /// `{ c₁; c₂; ...; cₙ }`
    ///
    /// Core expressions in sequence
    Block(Vec<Core>),

    /// `f(a, b, c)`
    ///
    /// A function call.
    Call(Box<Core>, Vec<Core>),

    /// `1` or `"hello"` or `()`
    Constant(Constant),

    /// `x`
    ///
    /// Some name for a thing, not yet resolved.
    Id(Id),

    /// `(a, b, c) => expr`
    ///
    /// A function with a fixed number of positional parameters with names.
    ///
    /// There's no var-arg, no keyword arguments, no optional arguments, etc.
    Lambda(Vec<Id>, Box<Core>),
}

impl fmt::Display for Core {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Core::Bind(id, ref b) => {
                let (body, scope) = b.deref();
                write!(f, "let {} = {} in {}", id, body, scope)
            }
            Core::Block(ss) => {
                write!(f, "{{ ")?;
                for s in ss {
                    write!(f, "{s}; ")?;
                }
                write!(f, "}}")
            }
            Core::Call(func, args) => {
                write!(f, "{}( ", func.as_ref())?;
                for arg in args {
                    write!(f, "{arg}, ")?;
                }
                write!(f, ")")
            }
            Core::Constant(c) => write!(f, "{c}"),
            Core::Id(id) => write!(f, "{id}"),
            Core::Lambda(params, body) => {
                write!(f, "( ")?;
                for p in params {
                    write!(f, "{p}, ")?;
                }
                write!(f, ") => {body}")
            }
        }
    }
}

impl Core {
    /// Each statement gets translated in order
    ///
    /// ⟦ s₁; s₂; ..., sₙ ⟧ = { ⟦ s₁ ⟧; ⟦ s₂; ...; sₙ; ⟧ }
    ///
    /// Empty statements get ignored if there's stuff after
    ///
    /// ⟦ ; ... ⟧ = ⟦ ... ⟧
    ///
    /// Empty statements at the end get turned into unit
    ///
    /// ⟦ ; ⟧ = ()
    ///
    /// Let statements eat up everything after them, bringing it into their scope.
    ///
    /// ⟦ let x = e; s₁; ... sₙ; ⟧ = let x = ⟦ e ⟧ in { ⟦ s₁ ⟧; ...; ⟦ sₙ ⟧; }
    pub fn statements(ss: &[Statement]) -> Core {
        if ss.is_empty() {
            // We literally replace `{;}` with `()`.
            return Core::Constant(Constant::Unit);
        }

        let mut cs = Vec::new();

        for i in 0..ss.len() {
            match &ss[i] {
                Statement::Empty(_) => {}
                Statement::Expression(e) => cs.push(e.into()),
                Statement::Binding(b) => {
                    // in the case of bindings, the binding takes over the rest
                    // of the statements since they're now after it's `in` part.
                    let rest = &ss[i + 1..];
                    cs.push(Core::binding(b, rest));
                    break;
                }
            }
        }

        Core::Block(cs)
    }

    /// See [Core::statements] for how this is translated.
    ///
    /// We can't translate a binding without knowing the statements which come
    /// after it where it's in scope.
    fn binding(b: &Binding, rest: &[Statement]) -> Core {
        let id = b.name().into();
        let body = b.body().into();
        let expr = Core::statements(rest);

        Core::Bind(id, Box::new((body, expr)))
    }
}

impl<'a> From<&Expression<'a>> for Core {
    fn from(expression: &Expression<'a>) -> Core {
        match expression {
            Expression::Block(b) => b.into(),
            Expression::Call(c) => c.into(),
            Expression::Function(f) => f.into(),
            Expression::Grouping(g) => g.into(),
            Expression::Identifier(id) => Core::Id(id.into()),
            Expression::List(l) => l.into(),
            Expression::Literal(l) => l.into(),
        }
    }
}

impl<'a> From<&Block<'a>> for Core {
    /// ⟦ { s₁; ...; sₙ } ⟧ = { ⟦ s₁; ...; sₙ ⟧ }  
    fn from(syntax: &Block<'a>) -> Core {
        Core::statements(syntax.statements().as_slice())
    }
}

impl<'a> From<&Call<'a>> for Core {
    /// ⟦ f( a₁, ..., aₙ ) ⟧ =  ⟦ f ⟧ ( ⟦ a₁ ⟧, ..., ⟦ aₙ ⟧ )
    fn from(syntax: &Call<'a>) -> Core {
        let func = Box::new(syntax.target().into());
        let args = syntax.arguments().iter().map(Core::from).collect();

        Core::Call(func, args)
    }
}

impl<'a> From<Constant> for Core {
    /// This one's a doozy.
    ///
    /// ⟦ c ⟧ = c
    fn from(c: Constant) -> Core {
        Core::Constant(c)
    }
}

impl<'a> From<&Function<'a>> for Core {
    /// ⟦ ( p₁, ..., pₙ ) => e ⟧ = ( ⟦ p₁ ⟧, ..., ⟦ pₙ ⟧ ) => ⟦ e ⟧
    fn from(syntax: &Function<'a>) -> Core {
        let params = syntax
            .parameters()
            .iter()
            .map(Parameter::name)
            .map(Id::from)
            .collect();
        let body = Box::new(syntax.body().into());
        Core::Lambda(params, body)
    }
}

impl<'a> From<&Grouping<'a>> for Core {
    fn from(syntax: &Grouping<'a>) -> Core {
        syntax.body().into()
    }
}

impl<'a> From<Id> for Core {
    /// ⟦ id ⟧ = id
    fn from(id: Id) -> Core {
        Core::Id(id)
    }
}

impl<'a> From<&Module<'a>> for Core {
    /// See [`Core::statements`] for how this is translated into core.
    fn from(syntax: &Module<'a>) -> Core {
        Core::statements(syntax.statements().as_slice())
    }
}

impl<'a> From<&TopLevel<'a>> for Core {
    /// See [`Core::statements`] for how this is translated into core.
    fn from(syntax: &TopLevel<'a>) -> Core {
        Core::statements(syntax.statements().as_slice())
    }
}

impl<'a> From<&List<'a>> for Core {
    /// We translate a list by making an empty mutable list and pushing elements
    /// into it.
    ///
    /// ⟦ [ e₁, ... eₙ ] ⟧ = { var list = []; ⟦ e₁, ... eₙ ⟧; list }
    ///
    /// Inside, we expand elements like this:
    ///
    /// ⟦ e ⟧ = List.push(list, ⟦ e ⟧) ⟦ ...e ⟧ = List.extend(list, ⟦ e ⟧)
    fn from(_syntax: &List<'a>) -> Core {
        unimplemented!(
            "cannot lower lists to core until we have mutation, \
            built-in list methods, etc."
        )
    }
}

impl<'a> From<&Literal<'a>> for Core {
    /// ⟦ 1 ⟧ = 1
    fn from(syntax: &Literal<'a>) -> Core {
        Core::Constant(syntax.into())
    }
}
