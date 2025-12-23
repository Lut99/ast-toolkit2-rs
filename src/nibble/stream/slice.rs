//  SLICE.rs
//    by Lut99
//
//  Description:
//!   Provides a [`ParseStream`] implementation that parses completely in-memory slices.
//

use std::convert::Infallible;

use super::ParseStream;
use crate::loc::Loc;


/***** LIBRARY *****/
/// A [`ParseStream`] implementation that can be used to parse completely in-memory slices.
#[derive(Debug)]
pub struct SliceParser<'a, T> {
    /// The slice being parsed.
    slice: &'a [T],
    /// The current position in the input stream.
    pos:   usize,
}

// Constructors
impl<'a, T> SliceParser<'a, T> {
    /// Constructor for the SliceParser that initializes it from your slice.
    ///
    /// You can use [`SliceParser::from()`] as well to do the same thing.
    ///
    /// # Arguments
    /// - `slice`: A slice to wrap this parser around.
    ///
    /// # Returns
    /// A new SliceParser that implements [`ParseStream`].
    #[inline]
    pub const fn new(slice: &'a [T]) -> Self { Self { slice, pos: 0 } }
}

// Ops
// NOTE: Necessary because we don't need the `Clone`-bound on `T`
impl<'a, T> Clone for SliceParser<'a, T> {
    #[inline]
    fn clone(&self) -> Self { Self { slice: self.slice, pos: self.pos } }
}
// NOTE: Necessary because we don't need the `Clone`-bound on `T`
impl<'a, T> Copy for SliceParser<'a, T> {}

// ParseStream
impl<'a, T> ParseStream for SliceParser<'a, T> {
    type Elem<'s>
        = &'s T
    where
        Self: 's;
    type Error = Infallible;

    #[inline]
    fn next<'s>(&'s mut self) -> Result<Option<(Self::Elem<'s>, Loc)>, Self::Error> {
        if self.pos >= self.slice.len() {
            return Ok(None);
        }
        let pos: usize = self.pos;
        self.pos += 1;
        Ok(Some((&self.slice[pos], Loc::encapsulate_range(self.slice.as_ptr() as u64, pos..=pos))))
    }

    #[inline]
    fn commit(&mut self) {
        /* A no-op, as we never need to drop memory. */
    }
}

// Conversion
impl<'a, T> From<&'a [T]> for SliceParser<'a, T> {
    #[inline]
    fn from(value: &'a [T]) -> Self { Self::new(value) }
}
impl<'a> From<&'a str> for SliceParser<'a, u8> {
    #[inline]
    fn from(value: &'a str) -> Self { Self::new(value.as_bytes()) }
}
