//  LAYOUTING ALGORITHM - v4.rs
//    by Lut99
//
//  Description:
//!   Fourth iteration of the terribly infamous layouting algorithm for
//!   displaying [`Diag`]s.
//!
//!   # Algorithm
//!   Instead of having a main renderer drive the process and then use a
//!   layouter to fill in blanks, we turn the script around: in this version,
//!   a layouter drives the process and interfaces with a special formatter
//!   that handles the standardized parts.
//

// Modules
pub mod formatter;
mod messages;
pub mod renderers;

// Imports & Exports
use std::fmt;

use formatter::{main, replace, sourceless};
use messages::{Message, as_messages};
use renderers::Renderer;

use super::Diag;
use super::annots::Severity;


/***** INTERFACES *****/
/// Helper interface giving styling options to a [`fmt::Write`]r.
pub trait StyledWrite: fmt::Write {
    /// Applies the internal style for highlighted source text to your given
    /// [`Display`](fmt::Display).
    fn write_highlighted(&mut self, sev: Severity, target: impl fmt::Display) -> fmt::Result;

    /// Applies the internal style for unhighlighted source text to your given
    /// [`Display`](fmt::Display).
    fn write_base(&mut self, target: impl fmt::Display) -> fmt::Result;
}

// Pointer-like impls
impl<'a, T: StyledWrite> StyledWrite for &'a mut T {
    #[inline(always)]
    fn write_highlighted(&mut self, sev: Severity, target: impl fmt::Display) -> fmt::Result {
        <T as StyledWrite>::write_highlighted(self, sev, target)
    }

    #[inline(always)]
    fn write_base(&mut self, target: impl fmt::Display) -> fmt::Result { <T as StyledWrite>::write_base(self, target) }
}





/***** LIBRARY *****/
/// Helper struct that can be used to [`Display`](fmt::Display) a [`Diag`]nostic.
pub struct Display<'d, R, E> {
    renderer: R,
    diag:     &'d Diag<E>,
}

impl<'d, R: Renderer<E>, E> fmt::Display for Display<'d, R, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msgs = as_messages(self.diag);
        for msg in msgs {
            match msg {
                Message::Main(m) => self.renderer.render_main(main::Formatter::new(&self.diag.theme, f), m)?,
                Message::Replace(m) => self.renderer.render_replace(replace::Formatter::new(&self.diag.theme, f), m)?,
                Message::Sourceless(m) => self.renderer.render_sourceless(sourceless::Formatter::new(&self.diag.theme, f), m)?,
            }
        }
        Ok(())
    }
}





/***** IMPLS *****/
impl<E> Diag<E> {
    /// Allows you to [`Display`](fmt::Display) the Diag.
    ///
    /// This will render the error.
    ///
    /// # Arguments
    /// - `renderer`: A [`Renderer`] that decides how to show the original source text.
    ///
    /// # Returns
    /// A [`Display`] implementing [`Display`](fmt::Display).
    #[inline]
    pub const fn display<R: Renderer<E>>(&self, renderer: R) -> Display<'_, R, E> { Display { renderer, diag: self } }
}
