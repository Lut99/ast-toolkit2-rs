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
use crate::prelude::ResultExt as _;


/***** ERRORS *****/
/// Defines the error type yielded by parsing [`Vec`]s.
#[derive(Debug, Error)]
#[error("Failed to parse {pos}th element")]
pub struct Error<E> {
    /// The position of the element that failed.
    pub pos: usize,
    /// The error that caused it to fail.
    #[source]
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
    fn parse<'s, I: ParseStream<Elem<'s> = E>>(input: &'s I) -> Result<Option<Self>, NibbleError<Self::Error, I::Error>> {
        // Read the input stream
        // NOTE: I suspect it's quite optimal to avoid allocating until the first element. This
        // because of the brute-force nature of the parser, and we'll probably see more failing
        // calls then successful calls.
        let mut res = Vec::new();
        while let Some(item) = input.parse::<T>().map_syntax(|err| Error { pos: res.len(), err })? {
            // Do some optimized scaling if necessary
            if res.is_empty() {
                res.reserve(4);
            } else if res.len() >= res.capacity() {
                res.reserve(res.len());
            }

            // Then push the item
            res.push(item);
        }

        // Shrink to something efficient
        res.shrink_to_fit();
        Ok(Some(res))
    }
}
