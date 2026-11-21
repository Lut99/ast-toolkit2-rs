//  SOURCELESS.rs
//    by Lut99
//
//  Description:
//!   Implements formatting for sourceless messages.
//

use std::fmt;

use super::super::super::themes::Theme;
pub use super::super::messages::MessageSourceless as Message;
use super::common::write_header;


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
    /// Renders the given [`MessageSourceless`] to the internal formatter.
    ///
    /// # Arguments
    /// - `msg`: The [`MessageSourceless`] to render (i.e., one without source, i.e., requiring no
    ///   input from you!).
    ///
    /// # Errors
    /// This function fails if we failed to write to the internal writer.
    #[inline(always)]
    pub fn write_sourceless(&mut self, msg: Message<'_>) -> fmt::Result {
        write_header(self.f, self.theme, self.max_line_width, false, msg.sev, msg.msg)
    }
}





/***** TESTS *****/
#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::super::super::super::annots::Severity;
    use super::super::super::super::themes::Theme;
    use super::*;

    struct Display<'d>(Cell<Option<Message<'d>>>);
    impl<'d> fmt::Display for Display<'d> {
        #[inline]
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let mut f = Formatter::new(&Theme::PLAIN, f);
            f.set_max_line_width(20);
            f.write_sourceless(self.0.take().take().unwrap())
        }
    }

    #[test]
    fn test_formatter_write_sourceless() {
        let msg = Message { sev: Severity::Error, msg: "Darth Plagueis the Wise was a Dark Lord of the Sith." };
        assert_eq!(
            Display(Cell::new(Some(msg))).to_string(),
            r#"error: Darth
Plagueis the Wise
was a Dark Lord of
the Sith.
"#
        );
    }
}
