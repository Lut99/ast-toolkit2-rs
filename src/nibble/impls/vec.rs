//  VEC.rs
//    by Lut99
//
//  Description:
//!   Implements [`Parsable`] for [`Vec`]s of [`Parsable`] things.
//

use std::fmt::{Display, Formatter, Result as FResult};

use super::super::error::ResultExt;
use super::super::{NibbleError, Parsable, Slice};


/***** FORMATTERS *****/
#[derive(Debug)]
pub struct VecFormatter<F> {
    /// The inner formatter
    fmt: F,
}
impl<F: Display> Display for VecFormatter<F> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        write!(f, "zero or more occurrences of ")?;
        Display::fmt(&self.fmt, f)
    }
}
impl<F> From<F> for VecFormatter<F> {
    #[inline]
    fn from(value: F) -> Self { Self { fmt: value } }
}





/***** IMPL *****/
impl<E, T: Parsable<E>> Parsable<E> for Vec<T> {
    type Formatter = VecFormatter<T::Formatter>;
    type Error = T::Error;

    #[inline]
    fn expects() -> Self::Formatter { VecFormatter { fmt: T::expects() } }

    #[inline]
    fn parse(mut input: Slice<E>) -> Result<(Self, Slice<E>), NibbleError<Self::Formatter, Self::Error>> {
        // Read the input stream
        // NOTE: I suspect it's quite optimal to avoid allocating until the first element. This
        // because of the brute-force nature of the parser, and we'll probably see more failing
        // calls then successful calls.
        let mut res = Vec::new();
        while let Some((item, rem)) = input.parse::<T>().transpose().auto_map()? {
            // Do some optimized scaling if necessary
            if res.is_empty() {
                res.reserve(4);
            } else if res.len() >= res.capacity() {
                res.reserve(res.len());
            }

            // Then push the item
            res.push(item);
            input = rem;
        }

        // Shrink to something efficient
        res.shrink_to_fit();
        Ok((res, input))
    }
}
