//  FORMATTER.rs
//    by Lut99
//
//  Description:
//!   Implements the standardized part across renderers.
//

// Modules
mod sourceless;

// Imports
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
