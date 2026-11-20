//  FORMATTER.rs
//    by Lut99
//
//  Description:
//!   Implements the standardized part across renderers.
//

use std::cell::Cell;
use std::fmt::{self, Display};

use console::{Style, StyledObject};
use unicode_segmentation::UnicodeSegmentation as _;

use super::super::themes::Theme;
use super::messages::MessageSourceless;


/***** HELPERS *****/
/// Allows one to apply one style to something written to multiple times.
struct FmtMultiplier<F>(Cell<Option<F>>);
impl<F: for<'a, 'b> FnOnce(&'a mut fmt::Formatter<'b>) -> fmt::Result> Display for FmtMultiplier<F> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Some(call) = self.0.take().take() else { panic!("Cannot call StyleMultiplier::fmt() twice") };

        // Run the call on the formatter
        call(f)
    }
}

trait StyleMultiplier {
    fn apply_to_all<F: for<'a, 'b> FnOnce(&'a mut fmt::Formatter<'b>) -> fmt::Result>(&self, call: F) -> StyledObject<FmtMultiplier<F>>;
}
impl StyleMultiplier for Style {
    #[inline(always)]
    fn apply_to_all<F: for<'a, 'b> FnOnce(&'a mut fmt::Formatter<'b>) -> fmt::Result>(&self, call: F) -> StyledObject<FmtMultiplier<F>> {
        self.apply_to(FmtMultiplier(Cell::new(Some(call))))
    }
}





/***** LIBRARY *****/
pub struct Formatter<'a> {
    /// A theme to apply to everything.
    theme: &'a Theme,
    /// The real formatter.
    f:     &'a mut fmt::Formatter<'a>,
}

// Formatting
impl<'a> Formatter<'a> {
    /// Renders the given [`MessageSourceless`] to the internal formatter.
    ///
    /// # Arguments
    /// - `max_line_width`: The maximum line width to render. This counts "graphemes" and ignores
    ///   any styling characters. Think of it as "perceived" width.
    /// - `msg`: The [`MessageSourceless`] to render (i.e., one without source, i.e., requiring no
    ///   input from you!).
    ///
    /// # Errors
    /// This function fails if we failed to write to the internal writer.
    pub fn render_sourceless<'d>(&mut self, max_line_width: usize, msg: MessageSourceless<'d>) -> fmt::Result {
        let sev_style = self.theme.header_severity_style(msg.sev);
        let msg_style = &self.theme.header_extra_msg;

        // Write the initial severity
        let ssev: &str = msg.sev.as_str();
        writeln!(self.f, "{}{}", sev_style.apply_to(ssev), msg_style.apply_to(": "))?;

        // Write any other text, cutting off at words
        let mut col: usize = ssev.len() + 2;
        for (word_pos, word) in msg.msg.split_word_bound_indices() {
            // If the word would go out-of-bounds for the line, linewrap
            let mut letters = word.grapheme_indices(true).collect::<Vec<(usize, &str)>>();
            if col + letters.len() > max_line_width {
                writeln!(self.f)?;
                col = 0;
            }

            // However, if the word as a whole is too long, write chunks of it
            let mut start_pos = 0;
            while letters.len() > max_line_width {
                let term_pos = letters[max_line_width].0;
                writeln!(self.f, "{}", msg_style.apply_to(&word[start_pos..term_pos]));
                start_pos = term_pos;
                letters = letters.split_off(term_pos);
            }
        }
        writeln!(self.f)?;
    }
}
