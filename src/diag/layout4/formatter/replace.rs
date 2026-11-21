//  REPLACE.rs
//    by Lut99
//
//  Description:
//!   Implements formatting for replacing messages.
//

use std::fmt;

#[cfg(feature = "decl-macros")]
pub use ast_toolkit2_decl_macros::write_highlighted;

use super::super::super::themes::Theme;
use super::super::StyledWrite;
pub use super::super::messages::MessageReplace as Message;
use super::common::{write_empty_source_line, write_header, write_loc};
use crate::diag::Severity;
use crate::diag::layout4::formatter::common::write_source_line_start;


/***** LIBRARY *****/
/// A formatter for writing [sourceless messages](MessageSourceless).
pub struct Formatter<'a, 'f> {
    /// A theme to apply to everything.
    theme: &'a Theme,
    /// The maximum line width to write.
    max_line_width: usize,
    /// The real formatter.
    f: &'a mut fmt::Formatter<'f>,
}

// Constructors
impl<'a, 'f> Formatter<'a, 'f> {
    #[inline(always)]
    pub(in super::super) const fn new(theme: &'a Theme, f: &'a mut fmt::Formatter<'f>) -> Self { Self { theme, max_line_width: usize::MAX, f } }
}

// Factory methods
impl<'a, 'f> Formatter<'a, 'f> {
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
impl<'a, 'f> Formatter<'a, 'f> {
    /// Starts rendering a message.
    ///
    /// This will write the message header and the snippet loc string to the internal formatter.
    ///
    /// # Arguments
    /// - `sev`: A [`Severity`] detailling the severity of the error.
    /// - `msg`: A string message describing what happened.
    ///
    /// # Returns
    /// A [`MessageFormatter`] that can be used to write the remainder of the message.
    ///
    /// # Errors
    /// This function fails if we failed to write to the internal writer.
    #[inline(always)]
    pub fn start_message(&mut self, sev: Severity, msg: &str) -> Result<MessageFormatter<'_, 'a, 'f>, fmt::Error> {
        // Write the error header first
        write_header(self.f, self.theme, self.max_line_width, sev, msg)?;

        // Continue with the nested struct
        Ok(MessageFormatter::new(self))
    }
}



/// Writes the remainder of a message.
pub struct MessageFormatter<'s, 'a, 'f> {
    max_line_no_width: usize,
    f: &'s mut Formatter<'a, 'f>,
}

// Constructors
impl<'s, 'a, 'f> MessageFormatter<'s, 'a, 'f> {
    #[inline(always)]
    const fn new(f: &'s mut Formatter<'a, 'f>) -> Self { Self { max_line_no_width: 0, f } }
}

// Factory methods
impl<'s, 'a, 'f> MessageFormatter<'s, 'a, 'f> {
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
impl<'s, 'a, 'f> MessageFormatter<'s, 'a, 'f> {
    /// Starts rendering a (the only) snippet.
    ///
    /// This will write the snippet header and first empty line preceding the source text.
    ///
    /// # Arguments
    /// - `max_line_no_width`: The width of the widest line number that will be displayed.
    /// - `loc`: A string message describing where whatever happened, happened.
    ///
    /// # Returns
    /// A [`SnippetFormatter`] that can be used to write the remainder of the snippet.
    ///
    /// # Errors
    /// This function fails if we failed to write to the internal writer.
    pub fn start_snippet(&mut self, max_line_no_width: usize, loc: &str) -> Result<SnippetFormatter<'_, 'a, 'f>, fmt::Error> {
        write_loc(self.f.f, self.f.theme, max_line_no_width, loc)?;
        write_empty_source_line(self.f.f, self.f.theme, max_line_no_width)?;
        Ok(SnippetFormatter { max_line_no_width, f: self.f })
    }


    /// Finishes rendering this message.
    #[inline(always)]
    pub fn finish(self) {}
}



/// Writes the remainder of a snippet.
pub struct SnippetFormatter<'s, 'a, 'f> {
    max_line_no_width: usize,
    f: &'s mut Formatter<'a, 'f>,
}

// Formatting
impl<'s, 'a, 'f> SnippetFormatter<'s, 'a, 'f> {
    /// Starts rendering pre-replace source lines.
    ///
    /// # Returns
    /// A [`PreReplaceFormatter`] that can be used to write the remainder of the snippet.
    #[inline(always)]
    pub const fn start_pre_replace(&mut self) -> PreReplaceFormatter<'_, 'a, 'f> {
        PreReplaceFormatter { max_line_no_width: self.max_line_no_width, f: self.f }
    }

    /// Starts rendering aft-replace source lines.
    ///
    /// # Returns
    /// A [`AftReplaceFormatter`] that can be used to write the remainder of the snippet.
    #[inline(always)]
    pub const fn start_aft_replace(&mut self) -> AftReplaceFormatter<'_, 'a, 'f> {
        AftReplaceFormatter { max_line_no_width: self.max_line_no_width, f: self.f }
    }


    /// Finishes rendering this replace snippet part.
    ///
    /// # Errors
    /// This function fails if we failed to write to the internal writer.
    #[inline(always)]
    pub fn finish(self) -> fmt::Result { write_empty_source_line(self.f.f, self.f.theme, self.max_line_no_width) }
}



/// Allows one to write replacing source lines.
pub struct PreReplaceFormatter<'s, 'a, 'f> {
    max_line_no_width: usize,
    f: &'s mut Formatter<'a, 'f>,
}

// Formatting
impl<'s, 'a, 'f> PreReplaceFormatter<'s, 'a, 'f> {
    /// Starts rendering a single pre replace source line.
    ///
    /// # Arguments
    /// - `l`: The line number to write at the start of the line.
    ///
    /// # Returns
    /// A [`PreReplaceLineFormatter`] that can be used to write the remainder of the snippet.
    ///
    /// # Errors
    /// This function fails if we failed to write to the internal writer.
    #[inline(always)]
    pub fn start_line(&mut self, l: &str) -> Result<PreReplaceLineFormatter<'_, 'a, 'f>, fmt::Error> {
        write_source_line_start(self.f.f, self.f.theme, self.max_line_no_width, '-', l)?;
        Ok(PreReplaceLineFormatter { f: self.f })
    }


    /// Finishes rendering this replace snippet part.
    #[inline(always)]
    pub fn finish(self) {}
}



/// Allows one to write a replacing source line.
pub struct PreReplaceLineFormatter<'s, 'a, 'f> {
    f: &'s mut Formatter<'a, 'f>,
}

// Formatting
impl<'s, 'a, 'f> fmt::Write for PreReplaceLineFormatter<'s, 'a, 'f> {
    #[track_caller]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // Scan for newlines
        if s.chars().find(|c| *c == '\n').is_some() {
            panic!("Cannot write newlines to a PreReplaceLineFormatter");
        }
        self.f.f.write_str(s)
    }
}
impl<'s, 'a, 'f> StyledWrite for PreReplaceLineFormatter<'s, 'a, 'f> {
    #[inline]
    fn write_highlighted(&mut self, sev: Severity, target: impl fmt::Display) -> fmt::Result {
        write!(self.f.f, "{}", self.f.theme.source_highlight_style(sev).apply_to(target))
    }

    #[inline]
    fn write_base(&mut self, target: impl fmt::Display) -> fmt::Result { write!(self.f.f, "{}", self.f.theme.source_base.apply_to(target)) }
}
impl<'s, 'a, 'f> PreReplaceLineFormatter<'s, 'a, 'f> {
    /// Finishes rendering this line.
    ///
    /// # Errors
    /// This function fails if we failed to write to the internal writer.
    #[inline(always)]
    pub fn finish(self) -> fmt::Result { writeln!(self.f.f) }
}



/// Allows one to write replacing source lines.
pub struct AftReplaceFormatter<'s, 'a, 'f> {
    max_line_no_width: usize,
    f: &'s mut Formatter<'a, 'f>,
}

// Formatting
impl<'s, 'a, 'f> AftReplaceFormatter<'s, 'a, 'f> {
    /// Starts rendering a single aft replace source line.
    ///
    /// # Arguments
    /// - `l`: The line number to write at the start of the line.
    ///
    /// # Returns
    /// A [`AftReplaceLineFormatter`] that can be used to write the remainder of the snippet.
    ///
    /// # Errors
    /// This function fails if we failed to write to the internal writer.
    #[inline(always)]
    pub fn start_line(&mut self, l: &str) -> Result<AftReplaceLineFormatter<'_, 'a, 'f>, fmt::Error> {
        write_source_line_start(self.f.f, self.f.theme, self.max_line_no_width, '+', l)?;
        Ok(AftReplaceLineFormatter { f: self.f })
    }


    /// Finishes rendering this replace snippet part.
    #[inline(always)]
    pub fn finish(self) {}
}



/// Allows one to write a replacing source line.
pub struct AftReplaceLineFormatter<'s, 'a, 'f> {
    f: &'s mut Formatter<'a, 'f>,
}

// Formatting
impl<'s, 'a, 'f> fmt::Write for AftReplaceLineFormatter<'s, 'a, 'f> {
    #[track_caller]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // Scan for newlines
        if s.chars().find(|c| *c == '\n').is_some() {
            panic!("Cannot write newlines to a AftReplaceLineFormatter");
        }
        self.f.f.write_str(s)
    }
}
impl<'s, 'a, 'f> StyledWrite for AftReplaceLineFormatter<'s, 'a, 'f> {
    #[inline]
    fn write_highlighted(&mut self, sev: Severity, target: impl fmt::Display) -> fmt::Result {
        write!(self.f.f, "{}", self.f.theme.source_highlight_style(sev).apply_to(target))
    }

    #[inline]
    fn write_base(&mut self, target: impl fmt::Display) -> fmt::Result { write!(self.f.f, "{}", self.f.theme.source_base.apply_to(target)) }
}
impl<'s, 'a, 'f> AftReplaceLineFormatter<'s, 'a, 'f> {
    /// Finishes rendering this line.
    ///
    /// # Errors
    /// This function fails if we failed to write to the internal writer.
    #[inline(always)]
    pub fn finish(self) -> fmt::Result { writeln!(self.f.f) }
}
