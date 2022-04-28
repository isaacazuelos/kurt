use crate::caret::Caret;
use crate::highlight::Highlight;
use crate::input_coordinator::InputId;
use crate::level::Level;
use crate::message::Message;
use crate::Span;

/// Diagnostic messages, with a lot of trimmings.
///
/// The ultimate purpose of these is to be shown to the programmer at some
/// point. To that end, the `Display` implementation here just dumps un-wrapped
/// plain text.
#[derive(Debug)]
pub struct Diagnostic {
    /// The name of the place where the source code comes from.
    input_id: Option<InputId>,

    /// Where in the source file the error begins.
    ///
    /// Not all errors have a location, for instance "file not found" can't.
    location: Option<Caret>,

    /// The highlighted regions relevant to this diagnostic.
    highlights: Vec<Highlight>,

    /// This is the primary message of the diagnostic.
    message: Message,
}

impl Diagnostic {
    /// Create a new diagnostic message with only a simple description.
    ///
    /// Ideally this text would be sufficient for a familiar user to correct the
    /// issue, when combined with the source name and location.
    pub fn new(text: String) -> Self {
        Diagnostic {
            input_id: None,
            location: None,
            message: Message::new(Level::default(), text),
            highlights: Vec::new(),
        }
    }

    /// The id of the input that was being worked with when the issue arose.
    pub fn input_id(&self) -> Option<InputId> {
        self.input_id
    }

    /// Set the id of the input that caused this issue.
    pub fn set_input_id(&mut self, id: InputId) {
        self.input_id = Some(id);
    }

    /// The location where the issue started.
    ///
    /// Giving the issue a concrete starting location makes it easier for users
    /// to navigate their editor to a reasonable place to start investigating.
    pub fn location(&self) -> Option<Caret> {
        self.location
    }

    /// The location where the issue started.
    ///
    /// Giving the issue a concrete starting location makes it easier for users
    /// to navigate their editor to a reasonable place to start investigating.
    pub fn set_location(&mut self, location: Caret) {
        self.location = Some(location);
    }

    /// The primary message of this diagnostic.
    pub(crate) fn message(&self) -> &Message {
        &self.message
    }

    /// View the list of highlights.
    pub(crate) fn highlights(&self) -> &[Highlight] {
        &self.highlights
    }

    /// Add a highlight to this diagnostic message.
    pub fn add_highlight(&mut self, span: Span, note: String) {
        self.highlights.push(Highlight::new(span, note));
    }
}
