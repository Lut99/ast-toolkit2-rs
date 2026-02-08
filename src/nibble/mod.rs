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

pub mod fmt;
pub mod stream;
#[cfg(feature = "tree")]
mod utf8_tag;
mod vec;

use std::error::Error;
use std::fmt::{Formatter, Result as FResult};

pub use stream::ParseStream;
use thiserror::Error;

/// Shorthand for including all the traits of this crate.
pub mod prelude {
    pub use super::{Parsable, ParseStream};
    use crate::nibble::NibbleError;


    /// Trait for conveniently calling [`NibbleError::map_syntax()`] and
    /// [`NibbleError::map_stream()`] through a [`Result`].
    pub trait ResultExt<T, E1, E2> {
        /// Allows one to call [`NibbleError::map_syntax()`] through a [`Result`].
        ///
        /// # Arguments
        /// - `map`: Some [`FnOnce`] doing the mapping.
        ///
        /// # Returns
        /// An equivalent error but with a mapped `E1`.
        fn map_syntax<E>(self, map: impl FnOnce(E1) -> E) -> Result<T, NibbleError<E, E2>>;

        /// Allows one to call [`NibbleError::map_stream()`] through a [`Result`].
        ///
        /// # Arguments
        /// - `map`: Some [`FnOnce`] doing the mapping.
        ///
        /// # Returns
        /// An equivalent error but with a mapped `E2`.
        fn map_stream<E>(self, map: impl FnOnce(E2) -> E) -> Result<T, NibbleError<E1, E>>;
    }
    impl<T, E1, E2> ResultExt<T, E1, E2> for Result<T, NibbleError<E1, E2>> {
        #[inline]
        fn map_syntax<E>(self, map: impl FnOnce(E1) -> E) -> Result<T, NibbleError<E, E2>> {
            match self {
                Ok(res) => Ok(res),
                Err(err) => Err(err.map_syntax(map)),
            }
        }

        #[inline]
        fn map_stream<E>(self, map: impl FnOnce(E2) -> E) -> Result<T, NibbleError<E1, E>> {
            match self {
                Ok(res) => Ok(res),
                Err(err) => Err(err.map_stream(map)),
            }
        }
    }
}


/***** ERRORS *****/
/// Defines the error of all error types: a nibble error.
///
/// The nibble error essentially classifies errors occuring while parsing into three possibilities:
/// - [`Error::Syntax`] represents a syntax error. It's something the user should have done
///   differently in the input; and
/// - [`Error::Stream`] represents an error in the input stream. It's likely something like a file
///   being inaccessible or something.
#[derive(Debug, Error)]
pub enum NibbleError<E1, E2> {
    #[error("Syntax error")]
    Syntax(#[source] E1),
    #[error("Failed to read the next token in the input stream")]
    Stream(#[source] E2),
}
impl<E1, E2> NibbleError<E1, E2> {
    /// Map the [`Syntax`](NibbleError::Syntax) in this NibbleError.
    ///
    /// # Arguments
    /// - `map`: Some [`FnOnce`] doing the mapping.
    ///
    /// # Returns
    /// An equivalent error but with a mapped `E1`.
    #[inline]
    pub fn map_syntax<E>(self, map: impl FnOnce(E1) -> E) -> NibbleError<E, E2> {
        match self {
            Self::Syntax(err) => NibbleError::Syntax(map(err)),
            Self::Stream(err) => NibbleError::Stream(err),
        }
    }

    /// Map the [`Stream`](NibbleError::Stream) in this NibbleError.
    ///
    /// # Arguments
    /// - `map`: Some [`FnOnce`] doing the mapping.
    ///
    /// # Returns
    /// An equivalent error but with a mapped `E2`.
    #[inline]
    pub fn map_stream<E>(self, map: impl FnOnce(E2) -> E) -> NibbleError<E1, E> {
        match self {
            Self::Syntax(err) => NibbleError::Syntax(err),
            Self::Stream(err) => NibbleError::Stream(map(err)),
        }
    }
}
impl<E1: Eq, E2: Eq> Eq for NibbleError<E1, E2> {}
impl<E1: PartialEq, E2: PartialEq> PartialEq for NibbleError<E1, E2> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Syntax(lhs), Self::Syntax(rhs)) => lhs == rhs,
            (Self::Stream(lhs), Self::Stream(rhs)) => lhs == rhs,
            _ => false,
        }
    }
}





/***** LIBRARY *****/
/// The main trait of the nibble library: defines that a node is parsable.
pub trait Parsable<E>: Sized {
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
    /// # Arguments
    /// - `f`: Some [`Formatter`] to write the expects-strings to. Hence, implement this function
    ///   like you would e.g. [`Display`](std::fmt::Display).
    ///
    /// # Errors
    /// This function should error if it failed to write to the `f`ormatter.
    fn expects_fmt(f: &mut Formatter<'_>) -> FResult;


    /// The actual parsing function.
    ///
    /// TODO.
    fn parse<'s, I: ParseStream<Elem<'s> = E>>(input: &'s I) -> Result<Option<Self>, NibbleError<Self::Error, I::Error>>;
}
