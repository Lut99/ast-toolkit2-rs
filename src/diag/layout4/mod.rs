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

pub use formatter::sourceless;
use messages::{Message, as_messages};

use super::Diag;


/***** INTERFACES *****/
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
    fn render_main(&self, f: (), msg: ()) -> fmt::Result;

    /// Renders a [`MessageReplace`] to the given `f`ormatter.
    ///
    /// # Arguments
    /// - `f`: A [`Formatter`] to write to.
    /// - `msg`: A single message to render. This one represents a single annotation showing a
    ///   suggested replacement.
    ///
    /// # Errors
    /// This function should only error if it failed to interact with `f`.
    fn render_replace(&self, f: (), msg: ()) -> fmt::Result;

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
    fn render_sourceless(&self, f: sourceless::Formatter<'_, '_>, msg: sourceless::Message<'_>) -> fmt::Result;
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
                Message::Main(m) => self.renderer.render_main((), ())?,
                Message::Replace(m) => self.renderer.render_replace((), ())?,
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
