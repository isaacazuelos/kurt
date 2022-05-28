//! Diagnostics - user-readable messages

mod caret;
mod diagnostic;
mod diagnostic_coordinator;
mod emitter;
mod highlight;
mod input_coordinator;
mod level;
mod message;
mod span;

pub use self::{
    caret::Caret,
    diagnostic::Diagnostic,
    diagnostic_coordinator::DiagnosticCoordinator,
    input_coordinator::{InputCoordinator, InputId},
    level::Level,
    span::Span,
};

/// Convert a byte array into a `&str` we can use as input, creating a
/// reasonable [`Diagnostic`] if it's not UTF-8;
pub fn verify_utf8(input: &[u8]) -> Result<&str, Diagnostic> {
    std::str::from_utf8(input).map_err(|e| {
        let end = e.valid_up_to();
        let valid_bytes = &input[0..end];
        let valid = unsafe { std::str::from_utf8_unchecked(valid_bytes) };

        let mut location = Caret::default();
        for c in valid.chars() {
            location.increment(c);
        }

        Diagnostic::new("input was not valid UTF-8")
            .location(location)
            .info(format!("the input was valid UTF-8 until byte {end}"))
    })
}
