//  NIBBLE.rs
//    by Lut99
//
//  Description:
//!   Defines the `nibble` parse library of the `ast-toolkit2`.
//!   
//!   It is the spiritual successor of the
//!   [`ast-toolkit-snack`](https://github.com/Lut99/ast-toolkit-rs/tree/main/lib/ast-toolkit-snack)-library,
//!   which in turn is a conceptual fork of [`nom`](https://github.com/rust-bakery/nom).
//!   However, instead of being combinator-based, it takes the AST-approach and
//!   defines a (derivable!) [`Parse`]-trait that you can implement on
//!   individual nodes.
//!
//!   In short, it is a much more classic parsing library. However, its focus
//!   has not changed: make robust, error-friendly parsers.
//

// Modules
mod error;
mod impls;
mod slice;

// Imports
use std::error::Error;
use std::fmt::Display;

pub use error::NibbleError;
pub use slice::Slice;

/// Shorthand for including all the traits of this crate.
pub mod prelude {
    pub use super::error::ResultExt;
    pub use super::{Parsable, Slice};
}


/***** LIBRARY *****/
/// The main trait of the nibble library: defines that a node is parsable.
pub trait Parsable<E>: Sized {
    /// The type that formats what we expected if we didn't match the parent item.
    type Formatter: Display;
    /// The type of error returned when parsing fails.
    type Error: 'static + Error;


    /// Generates a description of what is expected when we try to parse this.
    ///
    /// This needn't be complex. If you're parsing an expression, have it generate "an expression";
    /// whatever it is, it should populate `X` in `Expected X`.
    ///
    /// The power comes of making this meaningful to the user. Seeing this word should tell them
    /// what the parser attempted to parse and roughly what they should've given.
    ///
    /// # Returns
    /// An [`Parsable::ExpectsFormatter`] implementing [`Display`] that can format the expected
    /// string.
    fn expects() -> Self::Formatter;

    /// The actual parsing function.
    ///
    /// TODO.
    fn parse(input: Slice<E>) -> Result<(Self, Slice<E>), NibbleError<Self::Formatter, Self::Error>>;
}
