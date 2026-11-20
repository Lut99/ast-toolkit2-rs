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

pub use formatter::Formatter;
pub use messages::{Message, MessageMain, MessageReplace, MessageSourceless, as_messages};

use super::Diag;


/***** INTERFACES *****/
pub trait Renderer<'d, E: 'd> {
    /// Renders a source array of `E`lements to the given [`Formatter`].
    ///
    /// Note that this is _not_ a [`std::fmt::Formatter`]. Instead, it is a custom type that writes
    /// the standardized parts to an internal output stream.
    ///
    /// # Arguments
    /// - `f`: A [`Formatter`] to write to.
    /// - `messages`: A stream of [`Message`]s to show. Each of these is a separate thing to
    ///   output, where each such they may be subdivided into multiple snippets.
    ///
    /// # Errors
    /// This function should only error if it failed to interact with `f`.
    fn render(&mut self, f: Formatter<'_>, messages: impl Iterator<Item = Message<'d, E>>) -> fmt::Result;
}





/***** LIBRARY *****/
/// Helper struct that can be used to [`Display`](fmt::Display) a [`Diag`]nostic.
pub struct Display<'d, R, E> {
    renderer: R,
    diag:     &'d Diag<E>,
}

impl<'d, R: Renderer<'d, E>, E> fmt::Display for Display<'d, R, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msgs = as_messages(self.diag);
        todo!()
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
    pub const fn display<'d, R: Renderer<'d, E>>(&'d self, renderer: R) -> Display<'d, R, E> { Display { renderer, diag: self } }
}
