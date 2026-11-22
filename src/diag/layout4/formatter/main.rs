//  MAIN.rs
//    by Lut99
//
//  Description:
//!   Implements formatting for main messages.
//

use std::fmt;

use super::super::super::annots::Severity;
use super::super::super::themes::Theme;
use super::super::StyledWrite;
pub use super::super::messages::{LineAnnot, MessageMain as Message};
use super::common::{Repeat, write_break_line, write_empty_source_line, write_header_main, write_linewrapped, write_loc, write_source_line_start};
use super::place::{Draw, place_annots};


/***** LIBRARY *****/
/// A formatter for writing [sourceless messages](MessageSourceless).
pub struct Formatter<'d, 'f> {
    /// A theme to apply to everything.
    theme: &'d Theme,
    /// The maximum line width to write.
    max_line_width: usize,
    /// The real formatter.
    f: &'d mut fmt::Formatter<'f>,
}

// Constructors
impl<'d, 'f> Formatter<'d, 'f> {
    #[inline(always)]
    pub(in super::super) const fn new(theme: &'d Theme, f: &'d mut fmt::Formatter<'f>) -> Self { Self { theme, max_line_width: usize::MAX, f } }
}

// Factory methods
impl<'d, 'f> Formatter<'d, 'f> {
    /// Sets the maximum line width for the Formatter to write to.
    ///
    /// # Arguments
    /// - `max_line_width`: The maximum width of the line. This goes in graphemes and ignores
    ///   styling ("perceived" line width).
    ///
    /// # Returns
    /// Self for chaining.
    #[inline(always)]
    pub fn set_max_line_width(&mut self, max_line_width: usize) -> &mut Self {
        self.max_line_width = max_line_width;
        self
    }
}

// Formatting
impl<'d, 'f> Formatter<'d, 'f> {
    /// Starts rendering a message.
    ///
    /// This will write the message header and the snippet loc string to the internal formatter.
    ///
    /// # Arguments
    /// - `sev`: A [`Severity`] detailling the severity of the error.
    /// - `code`: Any code to show to link the error.
    /// - `msg`: A string message describing what happened.
    /// - `notes`: Any notes to attach to the end of the message.
    ///
    /// # Returns
    /// A [`MessageFormatter`] that can be used to write the remainder of the message.
    ///
    /// # Errors
    /// This function fails if we failed to write to the internal writer.
    #[inline(always)]
    pub fn start_message(
        &mut self,
        sev: Severity,
        code: Option<&str>,
        msg: &str,
        notes: &'d [String],
    ) -> Result<MessageFormatter<'_, 'd, 'f>, fmt::Error> {
        // Write the error header first
        write_header_main(self.f, self.theme, self.max_line_width, sev, code, msg)?;

        // Continue with the nested struct
        Ok(MessageFormatter::new(notes, self))
    }
}



/// Formats the rest of a message.
pub struct MessageFormatter<'s, 'd, 'f> {
    max_line_no_width: usize,
    notes: &'d [String],
    f: &'s mut Formatter<'d, 'f>,
}

// Constructors
impl<'s, 'd, 'f> MessageFormatter<'s, 'd, 'f> {
    #[inline(always)]
    const fn new(notes: &'d [String], f: &'s mut Formatter<'d, 'f>) -> Self { Self { max_line_no_width: usize::MAX, notes, f } }
}

// Factory methods
impl<'s, 'd, 'f> MessageFormatter<'s, 'd, 'f> {
    /// Sets the maximum line width for the Formatter to write to.
    ///
    /// # Arguments
    /// - `max_line_width`: The maximum width of the line. This goes in graphemes and ignores
    ///   styling ("perceived" line width).
    ///
    /// # Returns
    /// Self for chaining.
    #[inline(always)]
    pub fn set_max_line_no_width(&mut self, max_line_no_width: usize) -> &mut Self {
        self.max_line_no_width = max_line_no_width;
        self
    }
}

// Formatting
impl<'s, 'd, 'f> MessageFormatter<'s, 'd, 'f> {
    /// Starts rendering a snippet.
    ///
    /// This will write the snippet header and first empty line preceding the source text.
    ///
    /// # Arguments
    /// - `loc`: A string message describing where whatever happened, happened.
    ///
    /// # Returns
    /// A [`SnippetFormatter`] that can be used to write the remainder of the snippet.
    ///
    /// # Errors
    /// This function fails if we failed to write to the internal writer.
    pub fn start_snippet(&mut self, loc: &str) -> Result<SnippetFormatter<'_, 'd, 'f>, fmt::Error> {
        write_loc(self.f.f, self.f.theme, self.max_line_no_width, loc)?;
        write_empty_source_line(self.f.f, self.f.theme, self.max_line_no_width)?;
        Ok(SnippetFormatter { max_line_no_width: self.max_line_no_width, f: self.f })
    }

    /// Writes a snippet that has no annotations, just a source text.
    ///
    /// # Arguments
    /// - `loc`: A string message describing where whatever happened, happened. But without
    ///   position information.
    ///
    /// # Errors
    /// This function fails if we failed to write to the internal writer.
    pub fn write_unannot_snippet(&mut self, loc: &str) -> fmt::Result { write_loc(self.f.f, self.f.theme, self.max_line_no_width, loc) }


    /// Finishes rendering this message.
    ///
    /// # Arguments
    /// - `extra_notes`: Additional notes generated while formatting, from eg placing annotations.
    ///
    /// # Errors
    /// This function fails if we failed to write to the internal writer.
    #[inline(always)]
    pub fn finish(self, extra_notes: &[String]) -> fmt::Result {
        // Do extra notes first, then the original notes
        for note in extra_notes.into_iter().chain(self.notes) {
            // Write the `=`
            write_source_line_start(self.f.f, self.f.theme, self.max_line_no_width, '=', "")?;
            write!(self.f.f, "{}", self.f.theme.footer_note.apply_to("note: "))?;
            write_linewrapped(self.f.f, self.f.max_line_width, 6, &self.f.theme.footer_note_msg, self.max_line_no_width + 3 + 6, note)?;
            writeln!(self.f.f)?;
        }
        Ok(())
    }
}



/// Writes the remainder of a snippet.
pub struct SnippetFormatter<'s, 'd, 'f> {
    max_line_no_width: usize,
    f: &'s mut Formatter<'d, 'f>,
}

// Formatting
impl<'s, 'd, 'f> SnippetFormatter<'s, 'd, 'f> {
    /// Starts rendering a source line.
    ///
    /// # Arguments
    /// - `l`: The line number to start with.
    ///
    /// # Returns
    /// A [`SourceLineFormatter`] that can be used to write the remainder of the snippet.
    ///
    /// # Errors
    /// This function fails if we failed to write to the internal writer.
    pub fn start_source_line(&mut self, l: &str) -> Result<SourceLineFormatter<'_, 'd, 'f>, fmt::Error> {
        write_source_line_start(self.f.f, self.f.theme, self.max_line_no_width, '|', l)?;
        Ok(SourceLineFormatter { f: self.f })
    }

    /// Starts rendering the annotation lines for a single source line.
    ///
    /// # Returns
    /// A [`AnnotLinesFormatter`] that can be used to write the remainder of the snippet.
    #[inline(always)]
    pub const fn start_annot_lines(&mut self) -> AnnotLinesFormatter<'_, 'd, 'f> {
        AnnotLinesFormatter { max_line_no_width: self.max_line_no_width, f: self.f }
    }

    /// Writes the "break" line in source text.
    ///
    /// # Errors
    /// This function fails if we failed to write to the internal writer.
    #[inline(always)]
    pub fn write_break_line(&mut self) -> fmt::Result { write_break_line(self.f.f, self.f.theme, self.max_line_no_width) }


    /// Finishes rendering this replace snippet part.
    ///
    /// # Errors
    /// This function fails if we failed to write to the internal writer.
    #[inline(always)]
    pub fn finish(self) -> fmt::Result { write_empty_source_line(self.f.f, self.f.theme, self.max_line_no_width) }
}



/// Writes the remainder of a line.
pub struct SourceLineFormatter<'s, 'd, 'f> {
    f: &'s mut Formatter<'d, 'f>,
}

// Formatting
impl<'s, 'd, 'f> fmt::Write for SourceLineFormatter<'s, 'd, 'f> {
    #[track_caller]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // Scan for newlines
        if s.chars().find(|c| *c == '\n').is_some() {
            panic!("Cannot write newlines to a SourceLineFormatter");
        }
        self.f.f.write_str(s)
    }
}
impl<'s, 'd, 'f> StyledWrite for SourceLineFormatter<'s, 'd, 'f> {
    #[inline]
    fn write_highlighted(&mut self, sev: Severity, target: impl fmt::Display) -> fmt::Result {
        write!(self.f.f, "{}", self.f.theme.source_highlight_style(sev).apply_to(target))
    }

    #[inline]
    fn write_base(&mut self, target: impl fmt::Display) -> fmt::Result { write!(self.f.f, "{}", self.f.theme.source_base.apply_to(target)) }
}
impl<'s, 'd, 'f> SourceLineFormatter<'s, 'd, 'f> {
    /// Finishes rendering this line.
    ///
    /// # Errors
    /// This function fails if we failed to write to the internal writer.
    #[inline(always)]
    pub fn finish(self) -> fmt::Result { writeln!(self.f.f) }
}



/// Writes the remainder of a line.
pub struct AnnotLinesFormatter<'s, 'd, 'f> {
    max_line_no_width: usize,
    f: &'s mut Formatter<'d, 'f>,
}

// Formatting
impl<'s, 'd, 'f> AnnotLinesFormatter<'s, 'd, 'f> {
    /// Renders the given [`LineAnnot`]s and places them on zero or more annotation lines.
    ///
    /// # Arguments
    /// - `no_extra_notes`: The number of extra notes already added. This to keep the numbering
    ///   consistent across snippets and annotations.
    /// - `annots`: The annotations to write.
    ///
    /// # Returns
    /// A list of extra annotations to add at the end of the message.
    ///
    /// # Errors
    /// This function fails if we failed to write to the internal writer.
    pub fn write_annots(&mut self, no_extra_notes: usize, annots: impl IntoIterator<Item = LineAnnot<'d>>) -> Result<Vec<String>, fmt::Error> {
        // Run the placement algorithm and write the lines!
        let (lines, notes) = place_annots(self.f.theme, self.f.max_line_width, no_extra_notes, annots);
        for mut line in lines {
            line.consolidate_free_space();
            if line.all_empty() {
                continue;
            }
            write_source_line_start(self.f.f, self.f.theme, self.max_line_no_width, '|', "")?;
            let mut iter = line.draw.into_iter().peekable();
            while let Some(draw) = iter.next() {
                match draw {
                    Draw::Empty(n) => {
                        // Only draw the empty space if there is something else to come.
                        // Only works if free space is already consolidated
                        if iter.peek().is_some() {
                            write!(self.f.f, "{}", Repeat(' ', n))?
                        }
                    },
                    Draw::Marker(marker, style, n) => write!(self.f.f, "{}", style.apply_to(Repeat(marker, n)))?,
                    Draw::Text(text, style, _) => write!(self.f.f, "{}", style.apply_to(text))?,
                }
            }
            writeln!(self.f.f)?;
        }

        // Return the notes
        Ok(notes)
    }


    /// Finishes rendering this line.
    ///
    /// # Errors
    /// This function fails if we failed to write to the internal writer.
    #[inline(always)]
    pub const fn finish(self) {}
}
