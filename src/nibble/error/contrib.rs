//  CONTRIBUTED.rs
//    by Lut99
//
//  Description:
//!   Defines standard errors contributed by the library.
//!
//!   Some of these are used when using macros.
//

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FResult};


/***** LIBRARY *****/
/// Defines errors when using `#[parse(try_as = ...)]` in the
/// [`Parsable`](derive@super::super::Parsable) derivation macro.
#[cfg(feature = "proc-macros")]
#[derive(Debug, Eq, PartialEq)]
pub enum TryAsError<E1, E2> {
    /// The parsing failed.
    Parse(E1),
    /// The conversion failed.
    TryFrom(E2),
}

// Traits
impl<E1: Display, E2: Display> Display for TryAsError<E1, E2> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        match self {
            Self::Parse(err) => Display::fmt(err, f),
            Self::TryFrom(err) => Display::fmt(err, f),
        }
    }
}
impl<E1: Error, E2: Error> Error for TryAsError<E1, E2> {
    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Parse(err) => Error::source(err),
            Self::TryFrom(err) => Error::source(err),
        }
    }
}
