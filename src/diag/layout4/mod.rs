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
mod formatter;
mod messages;
#[cfg(feature = "sources")]
pub mod renderers;

// Imports & Exports
use std::fmt;

pub use formatter::{main, replace, sourceless};
use messages::{Message, as_messages};

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



/// Interface that you can implement for a struct rendering source text.
pub trait Renderer<E> {
    /// Renders a [`MessageMain`] to the given `f`ormatter.
    ///
    /// # Arguments
    /// - `f`: A [`Formatter`] to write to.
    /// - `msg`: A single message to render. This one represents the main message and can have
    ///   zero or more _snippets_ of source text with zero or more _annotations_.
    ///
    /// # Errors
    /// This function should only error if it failed to interact with `f`.
    fn render_main<'d>(&self, f: main::Formatter<'d, '_>, msg: main::Message<'d>) -> fmt::Result;

    /// Renders a [`MessageReplace`] to the given `f`ormatter.
    ///
    /// # Arguments
    /// - `f`: A [`Formatter`] to write to.
    /// - `msg`: A single message to render. This one represents a single annotation showing a
    ///   suggested replacement.
    ///
    /// # Errors
    /// This function should only error if it failed to interact with `f`.
    fn render_replace<'d>(&self, f: replace::Formatter<'d, '_>, msg: replace::Message<'d, E>) -> fmt::Result;

    /// Renders a [`sourceless::Message`] to the given `f`ormatter.
    ///
    /// This function doesn't really require you to do anything except give the proper line width,
    /// if any.
    ///
    /// # Arguments
    /// - `f`: A [`sourceless::Formatter`] to write to.
    /// - `msg`: A single message to render. This one represents a single annotation showing a
    ///   suggested replacement.
    ///
    /// # Errors
    /// This function should only error if it failed to interact with `f`.
    fn render_sourceless<'d>(&self, f: sourceless::Formatter<'d, '_>, msg: sourceless::Message<'d>) -> fmt::Result;
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
