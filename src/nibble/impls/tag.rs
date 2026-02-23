//  UTF-8 TAG.rs
//    by Lut99
//
//  Description:
//!   Implements [`Parsable`] for [`Utf8Tag`]s.
//

use std::convert::Infallible;
use std::fmt::{Debug, Display, Formatter, Result as FResult};
use std::marker::PhantomData;

use super::super::error::Needed;
use super::super::{NibbleError, Parsable, Slice};
use crate::tree::Tag;


/***** FORMATTERS *****/
#[derive(Debug, Eq, PartialEq)]
pub struct TagFormatter<E, T> {
    /// The type to find.
    _t: PhantomData<(E, T)>,
}
impl<E, T> Display for TagFormatter<E, T>
where
    E: 'static,
    T: Tag<E>,
    &'static [E]: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult { write!(f, "{:?}", T::TAG) }
}





/***** IMPL *****/
impl<T, E> Parsable<E> for T
where
    T: Tag<E>,
    E: 'static + PartialEq,
    &'static [E]: Debug,
{
    type Formatter = TagFormatter<E, T>;
    type Error = Infallible;

    #[inline]
    fn expects() -> Self::Formatter { TagFormatter { _t: PhantomData } }

    #[inline]
    fn parse(input: Slice<E>) -> Result<(Self, Slice<E>), NibbleError<Self::Formatter, Self::Error>> {
        // Get a slice of bytes equal to (at most) the tag size
        let ((head, loc), rem) = input.head_slice_loc(Self::TAG.len());
        for (h, t) in head.into_iter().zip(Self::TAG.into_iter()) {
            if h != t {
                // Divirging bytes. More input can never fix this!
                return Err(NibbleError::Unmatched(Self::expects(), None));
            }
        }

        // Now it depends on whether the head is _all_ of TAG or whether it is a prefix of it
        if head.len() >= Self::TAG.len() {
            Ok((Self::with_loc(loc), rem))
        } else {
            let needed: usize = Self::TAG.len() - head.len();
            Err(NibbleError::Unmatched(Self::expects(), Some(Needed::Bounded(needed, needed))))
        }
    }
}





/***** TESTS *****/
#[cfg(test)]
mod tests {
    use super::*;
    use crate::loc::test::TestLoc;
    use crate::loc::{Loc, Located};
    use crate::tree::{Node, Term};

    #[test]
    fn test_utf8_tag() {
        /// Define a tag
        #[derive(Debug, Eq, PartialEq)]
        struct Hello(TestLoc);
        impl Located for Hello {
            #[inline]
            fn loc(&self) -> Loc { self.0.into() }
        }
        impl Node for Hello {}
        impl Term for Hello {}
        impl Tag<u8> for Hello {
            const TAG: &'static [u8] = b"Hello";

            #[inline]
            fn new() -> Self { Self(TestLoc::new()) }

            #[inline]
            fn with_loc(loc: Loc) -> Self { Self(TestLoc(loc)) }
        }


        // Define test inputs
        const ID: u64 = 0;
        let input1 = Slice::with_raw_id(ID, b"Hello".as_slice());
        let input2 = Slice::with_raw_id(ID, b"Hello, world!".as_slice());
        let input3 = Slice::with_raw_id(ID, b"Hell".as_slice());
        let input4 = Slice::with_raw_id(ID, b"foo".as_slice());
        let input5 = Slice::with_raw_id(ID, b"".as_slice());

        // Attempt to parse it
        assert_eq!(Hello::parse(input1), Ok((Hello(TestLoc(Loc::encapsulate_range(ID, ..5))), input1.slice(5..))));
        assert_eq!(Hello::parse(input2), Ok((Hello(TestLoc(Loc::encapsulate_range(ID, ..5))), input2.slice(5..))));
        assert_eq!(Hello::parse(input3), Err(NibbleError::Unmatched(TagFormatter { _t: PhantomData }, Some(Needed::Bounded(1, 1)))));
        assert_eq!(Hello::parse(input4), Err(NibbleError::Unmatched(TagFormatter { _t: PhantomData }, None)));
        assert_eq!(Hello::parse(input5), Err(NibbleError::Unmatched(TagFormatter { _t: PhantomData }, Some(Needed::Bounded(5, 5)))));
    }
}
