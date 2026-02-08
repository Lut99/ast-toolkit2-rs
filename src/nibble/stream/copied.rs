//  COPIED.rs
//    by Lut99
//
//  Description:
//!   Implements a [`ParseStream`] for slices where the element is [`Copy`].
//!   
//!   This differs from a [`SliceStream`](super::slice::SliceStream) by
//!   yielding the elements as-is instead of by reference.
//!
//!   Note that the [`ByteStream`](super::byte::ByteStream) is actually a type
//!   alias for this stream over raw bytes.
//

use std::cell::Cell;
use std::convert::Infallible;

use super::ParseStream;
use crate::loc::Loc;


/***** LIBRARY *****/
/// A stream over a slice yielding copies of the elements.
///
/// If your type is not cheaply copyable, you probably don't want to use this type. Instead, see
/// the [`SliceStream`](super::slice::SliceStream) that yields element by reference.
#[derive(Debug)]
pub struct CopiedStream<'a, T> {
    /// The slice of elements we are referring to.
    slice: &'a [T],
    /// The current position in the slice above.
    pos:   Cell<usize>,
}

// Constructors
impl<'a, T> CopiedStream<'a, T> {
    /// Constructor for the CopiedStream that initializes it from your slice.
    ///
    /// You can use [`CopiedStream::from()`] as well to do the same thing.
    ///
    /// # Arguments
    /// - `slice`: A slice to wrap this parser around.
    ///
    /// # Returns
    /// A new CopiedStream that implements [`ParseStream`].
    #[inline]
    pub const fn new(slice: &'a [T]) -> Self { Self { slice, pos: Cell::new(0) } }
}

// Ops
// NOTE: Necessary because we don't need the `Clone`-bound on `T`
impl<'a, T> Clone for CopiedStream<'a, T> {
    #[inline]
    fn clone(&self) -> Self { Self { slice: self.slice, pos: self.pos.clone() } }
}

// ParseStream
impl<'a, T: Clone> ParseStream for CopiedStream<'a, T> {
    type Elem<'s>
        = T
    where
        Self: 's;
    type Error = Infallible;

    #[inline]
    fn next<'s>(&'s self) -> Result<Option<(Self::Elem<'s>, Loc)>, Self::Error> {
        let pos: usize = self.pos.get();
        if pos >= self.slice.len() {
            return Ok(None);
        }
        self.pos.set(pos + 1);
        Ok(Some((self.slice[pos].clone(), Loc::encapsulate_range(self.slice.as_ptr() as u64, pos..=pos))))
    }

    #[inline]
    fn commit(&self) {
        /* A no-op, as we never need to drop memory. */
    }
}

// Conversion
impl<'a, T> From<&'a [T]> for CopiedStream<'a, T> {
    #[inline]
    fn from(value: &'a [T]) -> Self { Self::new(value) }
}
impl<'a> From<&'a str> for CopiedStream<'a, u8> {
    #[inline]
    fn from(value: &'a str) -> Self { Self::new(value.as_bytes()) }
}
