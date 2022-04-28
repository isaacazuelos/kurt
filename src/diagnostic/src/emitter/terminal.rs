//! Pretty printing of diagnostic messages.
//!
//! This module handles all the external libraries we need to do this (mostly)
//! right, and wraps them up in a single configurable printer.

#![allow(unused)]

use std::io::Result;
use std::{borrow::Cow, usize};

use hyphenation::{Language, Load, Standard};
use term_size::dimensions_stderr;
use termcolor::{
    BufferedStandardStream, Color, ColorChoice, ColorSpec, WriteColor,
};
use textwrap::{Options, WordSplitter};
use unicode_width::UnicodeWidthStr;

use crate::message::Message;
use crate::{emitter::line_art::LineArt, level::Level};
use crate::{Diagnostic, InputCoordinator};

use super::code_window::CodeWindow;
use super::Emitter;

// TODO: We could probably do a lot here to abstract this into a more general
//       trait that's more user-configurable.

// TODO: How should we handle aligning highlights with source code across weird
//       unicode lengths?

/// A printer.
pub struct FancyEmitter {
    /// Output stream.
    out: Box<dyn WriteColor>,
    /// The set of line art characters to use.
    line_art: LineArt,
    /// The dictionary used for hyphenation rules, used for messages. We cache
    /// this because loading it isn't trivial.
    dictionary: Option<Standard>,
    /// The max width of the output
    width: usize,
}

impl FancyEmitter {
    /// The default terminal width used if the actual terminal is below
    /// `MIN_WIDTH`.
    pub const DEFAULT_WIDTH: usize = 80;

    /// The narrowest allowed terminal size that things will be wrapped to, any
    /// smaller and we use `DEFAULT_WIDTH` instead to maintain readability.
    pub const MIN_WIDTH: usize = 40;

    /// Notes in the margins must be at least this wide.
    pub const MIN_NOTE_MARGIN: usize = 25;

    pub fn simpler() -> Self {
        FancyEmitter {
            out: Box::new(BufferedStandardStream::stderr(ColorChoice::Never)),
            line_art: LineArt::ASCII,
            dictionary: None,
            width: FancyEmitter::DEFAULT_WIDTH,
        }
    }

    /// Prints to stderr, using all the fancy features.
    pub fn full() -> Self {
        let width = if let Some((w, _)) = dimensions_stderr() {
            Self::MIN_WIDTH.max(w)
        } else {
            Self::DEFAULT_WIDTH
        };

        let dictionary = match Standard::from_embedded(Language::EnglishUS) {
            Ok(s) => Some(s),
            Err(_) => None,
        };

        FancyEmitter {
            out: Box::new(BufferedStandardStream::stderr(ColorChoice::Auto)),
            line_art: LineArt::UNICODE,
            width,
            dictionary,
        }
    }

    /// The width of line-wrapped output.
    pub(crate) fn width(&self) -> usize {
        self.width
    }

    /// The line art used by the printer
    pub(crate) fn line_art(&self) -> LineArt {
        self.line_art
    }

    /// A handle on the output stream.
    pub(crate) fn out(&mut self) -> &mut dyn std::io::Write {
        &mut self.out
    }

    /// How wide is a string when printed?
    ///
    /// This isn't the same as the string's `s.len()` which counts bytes, or the
    /// `s.chars().count()` as some rendered characters are multiple code points
    /// (and some single code points may be double wide in a terminal).
    pub(crate) fn presentation_width(&self, s: &str) -> usize {
        // TODO: We should count the lossy ascii width if the line art is ascii?
        UnicodeWidthStr::width(s)
    }

    /// Line wrapping for English text hyphenation.
    pub(crate) fn wrap<'a>(
        &self,
        text: &'a str,
        width: usize,
    ) -> Vec<Cow<'a, str>> {
        textwrap::wrap(
            text,
            Options::new(width).word_splitter(match &self.dictionary {
                Some(splitter) => WordSplitter::Hyphenation(splitter.clone()),
                None => WordSplitter::NoHyphenation,
            }),
        )
    }

    /// Line wrapping options for code.
    ///
    /// Input can't be more than one line.
    pub(crate) fn code_wrap<'a>(
        &self,
        text: &'a str,
        width: usize,
    ) -> Vec<&'a str> {
        debug_assert_eq!(text.lines().count(), 1);

        let mut buf: Vec<&'a str> = Vec::new();
        let mut remaining = text;

        while !remaining.is_empty() {
            // `str::split_at` will panic if width is too long, or not at a
            // character boundary.
            let mut split_index = width.min(remaining.len());
            while !remaining.is_char_boundary(split_index) {
                split_index -= 1;
            }

            let (l, r) = remaining.split_at(split_index);
            buf.push(l);
            remaining = r;
        }

        buf
    }

    /// Prints `len` number of the `padding` character.
    pub(crate) fn pad(&mut self, padding: char, len: usize) -> Result<()> {
        for _ in 0..len {
            write!(self.out, "{}", padding)?;
        }
        Ok(())
    }

    /// Set the output to print dimmed text.
    pub(crate) fn dim_spec(&mut self) -> Result<()> {
        let mut spec = ColorSpec::new();
        spec.set_dimmed(true);
        self.out.set_color(&spec)
    }

    /// Set the output to highlight printed text.
    pub(crate) fn highlight_spec(&mut self) -> Result<()> {
        let mut spec = ColorSpec::new();
        spec.set_bold(true);
        spec.set_fg(Some(Color::Yellow));
        self.out.set_color(&spec)
    }

    /// Set the output style to the style used for notes.
    pub(crate) fn note_spec(&mut self) -> Result<()> {
        let mut spec = ColorSpec::new();
        spec.set_fg(Some(Color::Blue));
        self.out.set_color(&spec)
    }

    /// Reset the printed style to the default.
    pub(crate) fn reset_spec(&mut self) -> Result<()> {
        self.out.reset()
    }

    /// Set the spec of the settings for this level.
    pub(crate) fn set_level_spec(&mut self, level: Level) -> Result<()> {
        let mut spec = ColorSpec::new();
        spec.set_bold(true);
        let color = match level {
            Level::Error => Color::Red,
        };
        spec.set_fg(Some(color));
        self.out.set_color(&spec)
    }
}

impl Emitter for FancyEmitter {
    fn emit(
        &mut self,
        d: &Diagnostic,
        i: &InputCoordinator,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        self.emit_message(d.message())?;

        if !d.highlights().is_empty() && d.input_id().is_some() {
            let input = i.get_input_buffer(d.input_id().unwrap());
            let name = i.get_input_name(d.input_id().unwrap());
            let mut window = CodeWindow::new(d.highlights(), input);

            window.print(self, &name)?;
        }

        self.out().flush()?;

        Ok(())
    }
}

// For emitter
impl FancyEmitter {
    fn emit_message(&mut self, msg: &Message) -> Result<()> {
        // The coloured prefix also decides how much subsequent lines are
        // indented.
        let prefix_length = self.emit_message_level(msg)?;
        let wrap_width = self.width() - prefix_length;

        // Now for the body of the message
        let lines = self.wrap(&msg.text(), wrap_width);

        // First line doesn't have a prefix, since it comes after the level.
        writeln!(self.out(), "{}", lines[0])?;

        // Subsequent lines are indented by the length of the level name, to
        // align with the `:`.
        for line in &lines[1..] {
            self.pad(' ', prefix_length)?;
            writeln!(self.out(), "{}", line)?;
        }

        Ok(())
    }

    /// Emits the coloured prefix of the message, which is the level name with a
    /// `": "` at the end for spacing. This will set and reset the colour too.
    fn emit_message_level(&mut self, msg: &Message) -> Result<usize> {
        self.set_level_spec(msg.level())?;
        write!(self.out(), "{}: ", msg.level().name())?;
        self.reset_spec()?;
        Ok(msg.level().name().len() + ": ".len())
    }
}
