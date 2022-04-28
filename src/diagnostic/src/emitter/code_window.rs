//! Code windows are previews into the source code presented when displaying
//! diagnostic messages to help the user locate problems.

use crate::{
    caret::Caret,
    emitter::terminal::FancyEmitter,
    highlight::{self, Highlight},
    span::Span,
};

/// A preview into the source code with highlights and notes in the margins to
/// help the user locate what a diagnostic is referring to.
///
/// Note that the code window doesn't know the name of the source file, where
/// the owning diagnostic begins, or other more general diagnostic information.
#[derive(Debug)]
pub(crate) struct CodeWindow<'i, 'd> {
    /// Any highlighted regions and margin notes to show along with the source
    /// code.
    highlights: &'d [Highlight],
    /// The lines of input from `starting_line` and of length `lines`.
    lines: Vec<&'i str>,
}

// Builder pattern
impl<'i, 'd> CodeWindow<'i, 'd> {
    /// Crate a new empty code window. It automatically knows what lines to save
    /// based on what's highlighted.
    pub fn new(highlights: &'d [Highlight], input: &'i str) -> Self {
        // Note that this constructor guarantees that at least one highlight
        // exists. This is needed so that `starting_line`, `line_count`, etc.
        // can not be `Option`s.
        let mut window = CodeWindow {
            highlights,
            lines: Vec::new(),
        };

        // TODO use offsets.

        window.save_lines_offset(input, 0);

        window
    }

    /// Set the lines of input shown in the window. This is typically only done
    /// right before printing by the presenter, as the source code files could
    /// be quite large.
    ///
    /// The `offset` is for working with input which does not begin at line 0.
    /// This would typically be needed for the REPL, between 'lines' of user
    /// input.
    pub fn save_lines_offset(&mut self, input: &'i str, offset: usize) {
        self.lines = input
            .lines()
            .skip(self.starting_line() as usize - offset)
            .take(self.line_count() as _)
            .collect::<Vec<_>>();

        debug_assert_eq!(self.lines.len(), self.line_count() as usize);
    }
}

// Getters
impl<'i, 'd> CodeWindow<'i, 'd> {
    /// The line number the code window starts at.
    pub fn starting_line(&self) -> u32 {
        self.highlights().first().unwrap().span().start().line()
    }

    /// The line number the code windows ends at.
    pub fn ending_line(&self) -> u32 {
        self.highlights().last().unwrap().span().end().line()
    }

    /// How many lines are included in the code window. This is a max count,
    /// some lines may be omitted during rendering if they're not the first of
    /// last line and do not contain any highlighted sequences.
    pub fn line_count(&self) -> u32 {
        self.ending_line() - self.starting_line() + 1
    }

    /// Get a view into the highlighted regions. This array will always be
    /// sorted by the start of the span of the highlights.
    pub fn highlights(&self) -> &[Highlight] {
        &self.highlights
    }

    /// Returns an iterator of all highlights (in order) which intersect a span,
    /// If the intersection is the end of teh highlight, the highlight's note is
    /// included if any.
    ///
    /// This is weird, but it's exactly what we need when deciding to show a
    /// note for some highlighted span for line-wrapped source code.
    pub(crate) fn highlights_intersecting(
        &self,
        span: Span,
    ) -> impl Iterator<Item = (Span, Option<&str>)> {
        self.highlights().iter().filter_map(move |h| {
            h.span().intersection(span).map(|s| {
                (
                    s,
                    if s.end() == h.span().end() {
                        h.note()
                    } else {
                        None
                    },
                )
            })
        })
    }

    /// Return an iterator over the line number and lines in the code window.
    pub fn lines(&self) -> impl Iterator<Item = (u32, &str)> {
        let offset = self.starting_line();

        self.lines
            .iter()
            .enumerate()
            .map(move |(i, s)| (i as u32 + offset, *s))
    }
}

/// emit code windows with fancy
impl<'i, 'd> CodeWindow<'i, 'd> {
    pub(crate) fn print(
        &self,
        e: &mut FancyEmitter,
        label: &str,
    ) -> std::io::Result<()> {
        self.header(e, label)?;

        for (number, line) in self.lines() {
            self.line(e, number, line)?;
        }

        writeln!(e.out())
    }

    /// Print a code window header with the right line art, right aligning the
    /// label.
    fn header(&self, e: &mut FancyEmitter, label: &str) -> std::io::Result<()> {
        let label_length = e.presentation_width(label);
        let code_width = self.code_width(e.width());

        // Copy these out since we'll need the &mut for getting `out`.
        let h = e.line_art().horizontal;
        let t = e.line_art().tee;

        e.dim_spec()?;
        e.pad(h, self.gutter_width())?;
        write!(e.out(), "{}", t)?;
        e.pad(h, code_width - label_length - 1)?; // for the space
        e.reset_spec()?;

        writeln!(e.out(), " {}", label)
    }

    /// Print the line `number` and any subsequent highlights and notes.
    fn line(
        &self,
        p: &mut FancyEmitter,
        number: u32,
        line: &str,
    ) -> std::io::Result<()> {
        let mut start = 0;

        // We print the gutter with the number out here, and print the '...'
        // ones inside.
        self.gutter(p, number)?;

        for (n, line) in p
            .code_wrap(line, self.code_width(p.width()))
            .iter()
            .enumerate()
        {
            if n != 0 {
                self.more_gutter(p)?;
            };

            writeln!(p.out(), "{}", line)?;

            // We don't use unicode length here, but count chars instead,
            // because that's what `Caret::increment` uses.
            let len = line.chars().count() as u32;
            let span = Span::new(
                Caret::new(number, start),
                Caret::new(number, start + len),
            );
            self.highlight_lines(p, span)?;
            start += len;
        }

        Ok(())
    }

    /// Draw the highlights (and notes if any) that apply to the `line` within
    /// span.
    fn highlight_lines(
        &self,
        p: &mut FancyEmitter,
        span: Span,
    ) -> std::io::Result<()> {
        debug_assert_eq!(span.start().line(), span.end().line());

        for (sp, note) in self.highlights_intersecting(span) {
            // We do some math to know how to align things for more lines.
            let left = (sp.start().column() - span.start().column()) as usize;
            let underline = (sp.end().column() - sp.start().column()) as usize;
            let right = self.code_width(p.width()) - (left + underline);

            if let Some(note) = note {
                if right >= FancyEmitter::MIN_NOTE_MARGIN {
                    self.underline(p, left, underline)?;
                    let indent = (left + underline + 1) as usize;
                    self.right_margin_note(p, note, indent)?;
                } else if left >= FancyEmitter::MIN_NOTE_MARGIN {
                    self.left_margin_note(p, note, left, underline)?;
                } else {
                    self.underline(p, left, underline)?;
                    writeln!(p.out())?;
                    self.below_note(p, note)?;
                }
            } else {
                self.underline(p, left, underline)?;
                writeln!(p.out())?;
            }
        }

        Ok(())
    }

    /// Write an empty gutter, pad `left` spaces, then run the line art up for
    /// `length`.
    fn underline(
        &self,
        p: &mut FancyEmitter,
        left: usize,
        length: usize,
    ) -> std::io::Result<()> {
        self.empty_gutter(p)?;
        p.pad(' ', left as usize)?;
        p.highlight_spec()?;
        p.pad(p.line_art().up, length as usize)?;
        p.reset_spec()
    }

    /// Write a note below the highlight line, since it didn't fit in the left
    /// or right margin.
    fn below_note(
        &self,
        p: &mut FancyEmitter,
        note: &str,
    ) -> std::io::Result<()> {
        for line in p.wrap(note, self.code_width(p.width())) {
            self.empty_gutter(p)?;
            p.note_spec()?;
            writeln!(p.out(), "{}", line)?;
            p.reset_spec()?;
        }
        Ok(())
    }
    /// Write a note in the right margin
    fn right_margin_note(
        &self,
        p: &mut FancyEmitter,
        note: &str,
        indent: usize,
    ) -> std::io::Result<()> {
        let lines = p.wrap(note, self.code_width(p.width()) - indent);

        p.note_spec()?;
        writeln!(p.out(), " {}", lines[0])?;
        p.reset_spec()?;

        for line in &lines[1..] {
            self.empty_gutter(p)?;
            p.pad(' ', indent)?;
            p.note_spec()?;
            writeln!(p.out(), "{}", line)?;
            p.reset_spec()?;
        }

        Ok(())
    }

    fn left_margin_note(
        &self,
        p: &mut FancyEmitter,
        note: &str,
        left: usize,
        underline: usize,
    ) -> std::io::Result<()> {
        let note_width = p.presentation_width(note);
        if note_width < left {
            // If we can, we right-align the message.
            self.empty_gutter(p)?;
            p.pad(' ', left - note_width - 1)?;
            p.note_spec()?;
            write!(p.out(), "{} ", note)?;
            p.reset_spec()?;

            p.highlight_spec()?;
            p.pad(p.line_art().up, underline)?;
            writeln!(p.out())?;
            p.reset_spec()?;
        } else {
            self.underline(p, left, underline)?;
            writeln!(p.out())?;
            self.below_note(p, note)?;
        }

        Ok(())
    }

    /// Print the gutter for the left hand side of the gutter, filling it with
    /// some `Display`able content `d`.
    ///
    /// This wil right pad with spaces, and ensures there's at least enough
    /// space for the line art `more` .
    fn gutter(
        &self,
        e: &mut FancyEmitter,
        content: impl std::fmt::Display,
    ) -> std::io::Result<()> {
        e.dim_spec()?;
        let vertical = e.line_art().vertical;
        let more_width = e.line_art().more.len();
        write!(
            e.out(),
            "{: >gutter_width$}{} ",
            content,
            vertical,
            gutter_width = self.gutter_width().min(more_width)
        )?;
        e.reset_spec()?;
        Ok(())
    }

    /// A helper for printing an empty gutter, used by highlight and note lines.
    fn empty_gutter(&self, p: &mut FancyEmitter) -> std::io::Result<()> {
        self.gutter(p, "")
    }

    /// A helper, for an gutter which shows the line are `more` string. Used by
    /// for line wrapping code
    fn more_gutter(&self, p: &mut FancyEmitter) -> std::io::Result<()> {
        self.gutter(p, p.line_art().more)
    }

    /// The width of the left column in the gutter, not including the vertical
    /// line or space after it.
    fn gutter_width(&self) -> usize {
        let n = self.ending_line();
        // first +1 is because Caret is zero-indexed, but presented as
        // 1-indexed. The second is part of the equation.
        (((n + 1) as f64).log10().floor() as usize) + 1
    }

    /// The width of the code window, which is the `max_width` minus the gutter
    /// and spacing needed for the boarder.
    fn code_width(&self, max_width: usize) -> usize {
        // The -2 is for the "| " between the code and gutter.
        max_width - self.gutter_width() - 2
    }
}
