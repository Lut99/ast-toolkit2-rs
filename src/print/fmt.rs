//  FORMATTER.rs
//    by Lut99
//
//  Description:
//!   Defines a custom [`Formatter`] that's more elaborate than the standard one.
//

use std::fmt::{Result as FResult, Write};

use crate::print::Coloring;


/***** LIBRARY *****/
pub struct Formatter<'w> {
    /// The writer we're a-writin' to
    writer:   &'w mut dyn Write,
    #[cfg(feature = "color")]
    coloring: Coloring,
}

// Constructors
impl<'w> Formatter<'w> {
    /// Constructor for the Formatter.
    ///
    /// # Arguments
    /// - `writer`: A [`Write`]r that this Formatter writes to.
    ///
    /// # Returns
    /// A new Formatter ready to format.
    #[inline]
    pub const fn new(writer: &'w mut dyn Write) -> Self { Self { writer, coloring: Coloring::Never } }

    /// Constructor for the Formatter that takes your color preference into account.
    ///
    /// Only available with the `color`-feature; use [`Formatter::new()`] otherwise.
    ///
    /// # Arguments
    /// - `writer`: A [`Write`]r that this Formatter writes to.
    /// - `coloring`: Whether to respect [`Formatter::style()`] choices and such.
    ///
    /// # Returns
    /// A new Formatter ready to format with your chosen color settings.
    #[cfg(feature = "color")]
    #[inline]
    pub const fn with_color(writer: &'w mut dyn Write, coloring: Coloring) -> Self { Self { writer, coloring } }
}

// Options

// Write
impl<'w> Write for Formatter<'w> {
    #[inline]
    fn write_str(&mut self, s: &str) -> FResult { todo!() }
}
