//  DISPLAY.rs
//    by Lut99
//
//  Description:
//!   Defines the [`Display`] that implements [`Display`](std::fmt::Display)
//!   around a formatter.
//

use std::fmt::Result as FResult;

use crate::print::{Coloring, Formatter, PrettyPrint};


/***** LIBRARY *****/
/// Turns something that's [`PrettyPrint`]able into something implementing
/// [`Display`](std::fmt::Display).
#[derive(Clone, Copy)]
pub struct Display<'a, A>(pub(crate) &'a A, #[cfg(feature = "color")] pub(crate) Coloring);

// Formatting
impl<'a, A: PrettyPrint> std::fmt::Display for Display<'a, A> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> FResult {
        #[cfg(feature = "color")]
        let mut f = Formatter::with_color(f, self.1);
        #[cfg(not(feature = "color"))]
        let mut f = Formatter::new(f);
        PrettyPrint::fmt(&self.0, &mut f)
    }
}
