//! Diagnostic messages.
//!
//! These are honestly a really hard to write well. The [rustc] book as some
//! useful tips.
//!
//! [rustc]: https://rustc-dev-guide.rust-lang.org/diagnostics.html

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
///
/// The interface is a little odd. Methods either use (possibly `mut`)
/// references and start with `get` or `set`, or consume `self` and return it.
/// The builder-style methods only allow setting, so they don't take an
/// [`Option`].
///
/// This is done for convenience because some information are typically added
/// when the [`Diagnostic`] is first created, and other times added later.
#[derive(Debug)]
pub struct Diagnostic {
    /// The name of the place where the source code comes from.
    input_id: Option<InputId>,

    /// Where in the source file the error begins.
    ///
    /// Not all errors have a location, for instance "file not found" can't.
    location: Option<Caret>,

    /// This is the primary message of the diagnostic.
    message: Message,

    /// The highlighted regions relevant to this diagnostic.
    highlights: Vec<Highlight>,

    /// A place to include extra information
    footers: Vec<Message>,
}

impl Diagnostic {
    /// Create a new diagnostic message with only a simple description.
    ///
    /// Ideally this text would be sufficient for a familiar user to correct the
    /// issue, when combined with the source name and location.
    ///
    /// The [`Level`]'s [`Default`] is used.
    pub fn new(text: impl Into<String>) -> Self {
        Diagnostic {
            input_id: None,
            location: None,
            message: Message::new(Level::default(), text.into()),
            highlights: Vec::new(),
            footers: Vec::new(),
        }
    }

    /// Add the id of the input that caused this issue.
    ///
    /// This is just an [`InputId`] that corresponds to an
    /// [`InputCoordinator`][ic], instead of say a reference, since as an
    /// optional field that can be specified later, we don't necessarily know
    /// the lifetime of the input when the Diagnostic is made.
    ///
    /// [id]: crate::InputCoordinator
    pub fn input(mut self, id: InputId) -> Self {
        self.input_id = Some(id);
        self
    }

    /// The id of the input that produced this issue.
    pub fn get_input(&self) -> Option<InputId> {
        self.input_id
    }

    /// Set the id of the input that caused this issue.
    pub fn set_input(&mut self, id: Option<InputId>) {
        self.input_id = id;
    }

    /// The location where the issue started.
    ///
    /// Giving the issue a concrete starting location makes it easier for users
    /// to navigate their editor to a reasonable place to start investigating.
    pub fn location(mut self, location: Caret) -> Self {
        self.location = Some(location);
        self
    }

    /// Get the location where the issue arose. This may be `None` if it's not
    /// known, or wouldn't be meaningful, such as if a file cannot be read.
    pub fn get_location(&self) -> Option<Caret> {
        self.location
    }

    /// Set the location where the issue arose.
    pub fn set_location(&mut self, location: Option<Caret>) {
        self.location = location;
    }

    /// Get the main diagnostic message.
    pub fn get_text(&self) -> &str {
        self.message.get_text()
    }

    /// Set the main message of this diagnostic message.
    ///
    /// This doesn't have a corresponding builder-style method as it's required
    /// by [`Diagnostic::new`].
    pub fn set_text(&mut self, text: String) {
        self.message.set_text(text);
    }

    /// The [`Level`] of this diagnostic.
    pub fn get_level(&self) -> Level {
        self.message.get_level()
    }

    /// Set the [`Level`] of this diagnostic.
    pub fn set_level(&mut self, level: Level) {
        self.message.set_level(level);
    }

    /// Add a highlight to this diagnostic message.
    pub fn highlight(mut self, span: Span, note: impl Into<String>) -> Self {
        self.highlights.push(Highlight::new(span, note.into()));
        self
    }

    /// View the list of highlights.
    pub(crate) fn get_highlights(&self) -> &[Highlight] {
        &self.highlights
    }

    /// Add an info footer
    pub fn info(mut self, text: impl Into<String>) -> Self {
        self.footers.push(Message::new(Level::Info, text.into()));
        self
    }

    /// Add an help footer
    pub fn help(mut self, text: impl Into<String>) -> Self {
        self.footers.push(Message::new(Level::Help, text.into()));
        self
    }

    /// The footers belonging to this diagnostic.
    pub(crate) fn get_footers(&self) -> &[Message] {
        &self.footers
    }
}
