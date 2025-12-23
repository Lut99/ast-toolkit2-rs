//  VEC.rs
//    by Lut99
//
//  Description:
//!   Implements [`Parsable`] for [`Vec`]s of [`Parsable`] things.
//

use std::fmt::{Formatter, Result as FResult};

use thiserror::Error;

use super::{NibbleError, ParseStream};
use crate::nibble::Parsable;


/***** ERRORS *****/
/// Defines the error type yielded by parsing [`Vec`]s.
#[derive(Debug, Error)]
#[error("Failed to parse {pos}th element")]
pub struct Error<E> {
    /// The position of the element that failed.
    pub pos: usize,
    /// The error that caused it to fail.
    pub err: E,
}





/***** IMPL *****/
impl<E, T: Parsable<E>> Parsable<E> for Vec<T> {
    type Error = Error<T::Error>;

    #[inline]
    fn expects_fmt(f: &mut Formatter<'_>) -> FResult {
        write!(f, "zero or more occurrences of ")?;
        T::expects_fmt(f)
    }

    #[inline]
    fn parse<'s, I: ParseStream<Elem<'s> = E>>(input: &'s mut I) -> Result<Self, NibbleError<Self::Error, I::Error>> {
        /* TODO */
    }
}
