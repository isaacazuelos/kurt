//! Operator fixity and precedence definitions.
//!
//! ## Precedence of infix and postfix operators
//!
//! Note that [`Precedence`] is only used with infix operators. Both pre- and
//! post-fix operators always bind more tightly than infix, which is to say
//! they're higher precedence. The ranking is as follows:
//!
//! 1. Any postfix operators
//! 2. Any prefix operators
//! 3. Infix operators, according to [`Precedence`]
//!
//! This isn't really _Truth_, it's just that other options lead to results that
//! feel [surprising] compared to how were using to reading math.
//!
//! [surprising]: https://stackoverflow.com/questions/21973537/
//!
//! Allowing infix operators to have higher precedence that prefix would allow
//! `•a ◊ b` to mean `•(a ◊ b)`, which most people wouldn't expect even though
//! those symbols don't have any commonly understood meaning here.

use std::collections::HashMap;

/// The associativity of an operator.
///
/// Operators 'lean' to a side when multiple operators with the same precedence
/// occur in a sequence. For example, addition is usually left-associative,
/// which means that `a + b + c` is the same as `(a + b) + c`. Since the left
/// one binds first, it's 'left' associative.
///
/// An example of a right-associative operator is the `->` used for functions
/// types. A type like `a -> b -> c` is parsed as `a -> (b -> c)`, i.e. a
/// function which returns a function.
///
/// You can also [Disallow][Associativity::Disallow] associativity for an
/// operator. What this means is that operators of that precedence cannot be
/// used in a context where the associativity would be needed determine the
/// correct parse. This is useful for operators which are more about side
/// effect, such as assignment, where the order may not be clear. For example,
/// we don't want to allow multiple assignment, e.g. `a = b = c`.
///
/// Usually math operators are [`Left`][Associativity::Left] aligned. Examples
/// of right-associative operators are exponentiation, or the `->` in function
/// types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Associativity {
    Left,
    Right,
    Disallow,
}

/// The precedence of an operator.
///
/// When multiple operators are used, precedence is how we decide which one
/// 'happens first'. For example, `a + b * c` is read as the same as `a + (b *
/// c)` because the `*` has higher precedence
///
/// This is really the same thing as PEDMAS but extended to more symbols.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Precedence(u8);

impl Precedence {
    /// The highest possible precedence.
    pub const MAX: Precedence = Precedence(16);

    /// The lowest possible precedence.
    pub const MIN: Precedence = Precedence(0);

    /// The next higher precedence.
    pub fn next(self) -> Self {
        if self == Precedence::MAX {
            panic!("Precedence overflow!");
        }

        Precedence(self.0 + 1)
    }
}

/// Information about how an operator has been defined for use. Some operators
/// can be used in different ways, such as `-a` and `a - b`. Some are even all
/// three, such as [Rust's range expressions][range] `..`.
///
/// [range]: https://doc.rust-lang.org/reference/expressions/range-expr.html
#[derive(Debug, Default, Clone, Copy)]
struct Fixity {
    prefix: bool,
    postfix: bool,
    infix: Option<(Associativity, Precedence)>,
}

impl Fixity {
    /// Is the operator defined for use as prefix?
    #[inline]
    fn prefix(&self) -> bool {
        self.prefix
    }

    /// Is the operator defined for use as postfix?
    #[inline]
    fn postfix(&self) -> bool {
        self.postfix
    }

    /// Is the operator defined for use as postfix, and with what associativity
    /// and precedence?
    #[inline]
    fn infix(&self) -> Option<(Associativity, Precedence)> {
        self.infix
    }
}

/// A dictionary-like value for keeping track of which operators are defined,
/// and what kind of use they support (of infix, prefix, and postfix).
#[derive(Debug)]
pub struct DefinedOperators {
    defined: HashMap<String, Fixity>,
}

impl DefinedOperators {
    /// Is a specific operator defined for prefix use?
    pub fn is_prefix(&self, op: &str) -> bool {
        self.defined.get(op).map(Fixity::prefix).unwrap_or(false)
    }

    /// Define a new prefix operator. Returns whether the operator was already
    /// defined for prefix use.
    pub fn define_prefix(&mut self, op: &str) -> bool {
        let fixity = self.get_mut(op);
        let old = fixity.prefix;
        fixity.prefix = true;
        old
    }

    /// Is a specific operator defined for postfix use?
    pub fn is_postfix(&self, op: &str) -> bool {
        self.defined.get(op).map(Fixity::postfix).unwrap_or(false)
    }

    /// Define a new postfix operator. Returns whether the operator was already
    /// defined for postfix use.
    pub fn define_postfix(&mut self, op: &str) -> bool {
        let fixity = self.get_mut(op);
        let old = fixity.postfix;
        fixity.postfix = true;
        old
    }

    /// Is a specific operator defined for infix use at any precedence level and
    /// associativity?
    pub fn is_infix(&self, op: &str) -> bool {
        self.get_infix(op).is_some()
    }

    /// The precedence level and associativity for an operator, if it's defined
    /// for infix use. If it's not defined for infix, `None` is returned.
    pub fn get_infix(&self, op: &str) -> Option<(Associativity, Precedence)> {
        self.defined.get(op).and_then(Fixity::infix)
    }

    /// Define a new infix operator using the given associativity and
    /// precedence. If one was already defined it is returned, otherwise `None`
    /// is returned.
    pub fn define_infix(
        &mut self,
        op: &str,
        associate: Associativity,
        precedence: Precedence,
    ) -> Option<(Associativity, Precedence)> {
        let fixity = self.get_mut(op);
        let old = fixity.infix;
        fixity.infix = Some((associate, precedence));
        old
    }

    /// Get a mutable reference fixity for an operator.
    ///
    /// This will creates an entry with the default not-defined-for-any-use
    /// fixity if one doesn't already exist.
    fn get_mut(&mut self, op: &str) -> &mut Fixity {
        if !self.defined.contains_key(op) {
            self.defined.insert(op.into(), Fixity::default());
        }

        self.defined.get_mut(op).unwrap()
    }
}

impl Default for DefinedOperators {
    fn default() -> Self {
        use Associativity::*;

        let mut op = DefinedOperators {
            defined: HashMap::default(),
        };

        // the comments are what I've seen them used as, not what this language
        // will necessarily use them as.

        op.define_postfix("!"); // unwrap
        op.define_postfix("?"); // try

        op.define_prefix("!"); // not
        op.define_prefix("+"); // positive
        op.define_prefix("-"); // unary negative
        op.define_prefix("~"); // not
        op.define_prefix("*"); // deref,

        // lets us move these around easier
        let mut p = Precedence::MIN;

        // exponentiation
        op.define_infix("^", Right, p); // pow

        // multiplication
        p = p.next();
        op.define_infix("*", Left, p); // mul
        op.define_infix("/", Left, p); // div
        op.define_infix("%", Left, p); // mod

        // addition
        p = p.next();
        op.define_infix("+", Left, p); // add
        op.define_infix("-", Left, p); // sub

        // comparison
        p = p.next();
        op.define_infix("<", Left, p); // less
        op.define_infix(">", Left, p); // greater
        op.define_infix("<=", Left, p); // leq
        op.define_infix(">=", Left, p); // geq
        op.define_infix("<>", Left, p); // diamond
        op.define_infix("><", Left, p); // duck

        // equality
        p = p.next();
        op.define_infix("==", Left, p); // eq
        op.define_infix("!=", Left, p); // neq

        // bits
        p = p.next();
        op.define_infix("&", Left, p); // bit and
        op.define_infix("|", Left, p); // bit or
        op.define_infix("⊕", Left, p); // bit xor

        // shifts
        p = p.next();
        op.define_infix("<<", Left, p); // sll
        op.define_infix(">>", Left, p); // srl
        op.define_infix(">>>", Left, p); // sra

        // error
        op.define_infix("??", Left, p); // coalescing

        // errors and pipes
        p = p.next();
        op.define_infix("|>", Left, p); // f pipe

        // functions
        p = p.next();
        op.define_infix("->", Right, p);

        // assignment
        p = p.next();
        op.define_infix("=", Disallow, p);
        op.define_infix("+=", Disallow, p);
        op.define_infix("-=", Disallow, p);
        op.define_infix("*=", Disallow, p);
        op.define_infix("/=", Disallow, p);
        op.define_infix("%=", Disallow, p);
        op.define_infix("&=", Disallow, p);
        op.define_infix("|=", Disallow, p);
        op.define_infix("^=", Disallow, p);

        op
    }
}

#[cfg(test)]
mod tests {
    // nothing here is complicated enough to bother testing, yet.
}
