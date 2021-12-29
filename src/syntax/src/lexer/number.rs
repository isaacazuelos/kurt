//! Numeric literals lexing.
//!
//! What's supported should _mostly_ be unsurprising.
//!
//! There are a few supported types:
//!
//! 1. Base-10 numeric literals.
//! 2. Floating point literals mostly modeled after those in [JSON][json].
//! 3. Other common radix literals, with bases '0b' for base 2, '0o' for base 8,
//!    and '0x' for base 16.
//!
//! [json]: https://www.json.org/json-en.html
//!
//! # Notes
//!
//! There are a few caveats here worth pointing out, since they might seem
//! unintuitive to some.
//!
//! You cannot omit whole part of a floating point number and use just `.5`, you
//! have to specify `0.5`.
//!
//! Any leading '-' or '+' would be lexed as an operator and later parsed as a
//! unary operator (hopefully). A string like "-10" is two tokens.
//!
//! The actual value isn't interpreted at this stage, since we want to be able
//! to move on with parsing even if a numeric literal represents a value too
//! large for whatever the underlying storage is.
//!
//! We allow the digits after the first to include an underscore, so `10_000` is
//! allowed. We don't restrict where these occur or how many of them and they
//! can appear at the end as well. `12__34_5___` is the same number as `12345`.
//! We do require at least one digit at the start of any sequence though, so
//! `_0` isn't valid as an integer and likewise `0._` isn't a valid floating
//! point number.
//!
//! Note that the `.` in a floating point literal _must_ have a base-10 digit
//! after it. Otherwise it's lexed as a `.` in a method call. Something like `0
//! . 0` is an (invalid) method call.

// TODO: I'd like to support literal suffixes the way rust does with `47u32` but
//       I think it's wiser to wait until we have a better idea of what the type
//       names will be.

use crate::lexer::{Error, Lexer, TokenKind};

impl Lexer<'_> {
    /// The entry point for numeric literals.
    pub(crate) fn number(&mut self) -> Result<TokenKind, Error> {
        if self.peek() == Some('0') {
            match self.peek_nth(1) {
                Some('x') | Some('X') => self.radix_literal("xX", 16),
                Some('o') | Some('O') => self.radix_literal("oO", 8),
                Some('b') | Some('B') => self.radix_literal("bB", 2),
                _ => self.float_or_integer(),
            }
        } else {
            self.float_or_integer()
        }
    }

    /// Consume a number that _isn't_ one of the non-standard radix literals
    /// that start with '0' then a letter. This would be either an integer
    /// literal or a floating point number.
    ///
    /// ```text
    /// float_or_integer := digits
    ///                     ('.' digits)?
    ///                     (("e" | "E") ("+" | "-")? digits)?
    /// ```
    fn float_or_integer(&mut self) -> Result<TokenKind, Error> {
        self.consume_digits(10)
            .expect("Lexer::float_or_int expected at least 1 base-10 digit");

        let next = self.peek();

        if next == Some('e') || next == Some('E') {
            self.float()
        } else if next == Some('.')
            && self.peek_nth(1).map(|c| c.is_digit(10)).unwrap_or(false)
        {
            self.float()
        } else {
            Ok(TokenKind::Int)
        }
    }

    /// Floating point literals.
    ///
    /// This assumes the caller has already consumed the whole part of the
    /// floating point value.
    fn float(&mut self) -> Result<TokenKind, Error> {
        if let Some('.') = self.char('.') {
            let location = self.location;
            self.consume_digits(10)
                .ok_or_else(|| Error::InvalidFloatFractional(location))?;
        }

        if let Some(_) = self.one_of("eE") {
            // if this returns `None`, it's fine as the sign is optional.
            self.one_of("+-");

            let location = self.location;
            self.consume_digits(10)
                .ok_or_else(|| Error::InvalidFloatExponent(location))?;
        }

        Ok(TokenKind::Float)
    }

    /// Consume a radix literal like those used for hexadecimal, octal and
    /// binary literals.
    ///
    /// ```text
    /// radix_literal := "0" one_of(letters) digit(radix) digit_or_underscore(radix)*
    /// ```
    ///
    /// # Panics
    ///
    /// This will panic if the input does not start with '0' or '0', and then
    /// one of the characters in `letters`. It is the caller's responsibility to
    /// check for this.
    fn radix_literal(
        &mut self,
        letters: &'static str,
        radix: u32,
    ) -> Result<TokenKind, Error> {
        self.char('0')
            .expect("Lexer::radix_literal expected a leading 0");

        self.one_of(letters)
            .expect("Lexer::radix_literal expected specific letters after a 0");

        match self.consume_digits(radix) {
            Some(_) => Ok(TokenKind::Hex),
            None => Err(Error::EmptyRadixLiteral(self.location, radix)),
        }
    }

    /// Consumes one or digits in a specified radix, which must be less than or
    /// equal to 36 for [`char::is_digit`][std::char::is_digit] to function.
    ///
    /// After the first digit, we can have underscores for spacing.
    ///
    /// This returns an [`Option`] like the other combinators so that the caller
    /// can turn it into an appropriate [`Error`].
    fn consume_digits(&mut self, radix: u32) -> Option<&str> {
        match self.peek() {
            Some(c) if c.is_digit(radix as _) => {
                Some(self.consume_while(|c| c == '_' || c.is_digit(radix as _)))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero() {
        let mut lexer = Lexer::new("0");
        assert_eq!(lexer.token().unwrap().kind(), TokenKind::Int);
        assert!(lexer.is_empty());
    }

    #[test]
    fn integer() {
        let mut lexer = Lexer::new("12341");
        assert_eq!(lexer.token().unwrap().kind(), TokenKind::Int);
        assert!(lexer.is_empty());
    }

    #[test]
    fn signed() {
        let mut lexer = Lexer::new("-1");
        assert_eq!(lexer.token().unwrap().kind(), TokenKind::Operator);
        assert_eq!(lexer.token().unwrap().kind(), TokenKind::Int);
        assert!(lexer.is_empty());
    }

    #[test]
    fn really_big_integer() {
        // Since we don't convert the _value_ of the token here, it's fine for
        // now if the backing representation size can't support the number. This
        // is 2^512, which should almost certainly not fit by default.
        let mut lexer = Lexer::new("13_407_807_929_942_597_099_574_024_998\
            _205_846_127_479_365_820_592_393_377_723_561_443_721_764_030_073_546\
            _976_801_874_298_166_903_427_690_031_858_186_486_050_853_753_882_811\
            _946_569_946_433_649_006_084_096");
        assert_eq!(lexer.token().unwrap().kind(), TokenKind::Int);
        assert!(lexer.is_empty());
    }

    #[test]
    fn fractional() {
        let mut lexer = Lexer::new("0.0");
        assert_eq!(lexer.token().unwrap().kind(), TokenKind::Float);
        assert!(lexer.is_empty());
    }

    #[test]
    fn exponent() {
        // This almost looks like a base-e radix literal. :p
        let mut lexer = Lexer::new("0e1");
        assert_eq!(lexer.token().unwrap().kind(), TokenKind::Float);
        assert!(lexer.is_empty());
    }

    #[test]
    fn exponent_with_separators() {
        // yes this is weird
        let mut lexer = Lexer::new("0_e1_");
        assert_eq!(lexer.token().unwrap().kind(), TokenKind::Float);
        assert!(lexer.is_empty());
    }

    #[test]
    fn pathological_float() {
        let mut lexer = Lexer::new("0_.0_E-0_");
        assert_eq!(lexer.token().unwrap().kind(), TokenKind::Float);
        assert!(lexer.is_empty());
    }

    #[test]
    fn exponent_sign() {
        let mut lexer = Lexer::new("0e+1");
        assert_eq!(lexer.token().unwrap().kind(), TokenKind::Float);
        assert!(lexer.is_empty());
    }

    #[test]
    fn exponent_sign_no_digits() {
        let mut lexer = Lexer::new("0e-");
        assert!(lexer.token().is_err());
    }

    #[test]
    fn dot_underscore() {
        let mut lexer = Lexer::new("0._0");
        assert_eq!(lexer.token().unwrap().kind(), TokenKind::Int);
        assert_eq!(lexer.token().unwrap().kind(), TokenKind::Dot);
        assert_eq!(lexer.token().unwrap().kind(), TokenKind::Identifier);
        assert!(lexer.is_empty());
    }

    #[test]
    fn dot_letter() {
        let mut lexer = Lexer::new("0.a");
        assert_eq!(lexer.token().unwrap().kind(), TokenKind::Int);
        assert_eq!(lexer.token().unwrap().kind(), TokenKind::Dot);
        assert_eq!(lexer.token().unwrap().kind(), TokenKind::Identifier);
        assert!(lexer.is_empty());
    }

    #[test]
    fn no_leading_zero_in_float() {
        let mut lexer = Lexer::new(".5");
        assert_eq!(lexer.token().unwrap().kind(), TokenKind::Dot);
        assert!(!lexer.is_empty());
    }
}
