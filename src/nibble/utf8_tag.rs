//  UTF-8 TAG.rs
//    by Lut99
//
//  Description:
//!   Implements [`Parsable`] for [`Utf8Tag`]s.
//

use std::convert::Infallible;
use std::fmt::{Formatter, Result as FResult};

use super::{NibbleError, Parsable, ParseStream};
use crate::loc::Loc;
use crate::tree::Utf8Tag;


/***** IMPL *****/
impl<T: Utf8Tag> Parsable<u8> for T {
    type Error = Infallible;

    #[inline]
    fn expects_fmt(f: &mut Formatter<'_>) -> FResult { write!(f, "{:?}", Self::TAG) }

    #[inline]
    fn parse<'s, I: ParseStream<Elem<'s> = u8>>(input: &'s I) -> Result<Option<Self>, NibbleError<Self::Error, I::Error>> {
        // Parse them all in-sequence or die trying
        let mut loc: Option<Loc> = None;
        for tag_b in Self::TAG.bytes() {
            // Get the next byte and ensure it's correct
            let Some((b, l)) = input.next().map_err(NibbleError::Stream)? else { return Ok(None) };
            if tag_b != b {
                return Ok(None);
            }

            // Build the loc while at it
            if let Some(loc) = &mut loc {
                loc.extend(l);
            } else {
                loc = Some(l);
            }
        }
        Ok(Some(Self::with_loc(loc.unwrap_or(Loc::new()))))
    }
}





/***** TESTS *****/
#[cfg(test)]
mod tests {
    use super::super::stream::copied::CopiedStream;
    use super::*;
    use crate::loc::Located;
    use crate::loc::test::TestLoc;
    use crate::tree::Node;

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
        impl Utf8Tag for Hello {
            const TAG: &'static str = "Hello";

            #[inline]
            fn new() -> Self { Self(TestLoc::new()) }

            #[inline]
            fn with_loc(loc: Loc) -> Self { Self(TestLoc(loc)) }
        }


        // Define test inputs
        let input1 = CopiedStream::from("Hello");
        let input2 = CopiedStream::from("Hello, world!");
        let input3 = CopiedStream::from("Hell");
        let input4 = CopiedStream::from("foo");
        let input5 = CopiedStream::from("");

        // Attempt to parse it
        assert_eq!(Hello::parse(&input1), Ok(Some(Hello(TestLoc(Loc::encapsulate_range((b"Hello" as *const u8) as u64, ..5))))));
        assert_eq!(Hello::parse(&input2), Ok(Some(Hello(TestLoc(Loc::encapsulate_range((b"Hello, world!" as *const u8) as u64, ..5))))));
        assert_eq!(Hello::parse(&input3), Ok(None));
        assert_eq!(Hello::parse(&input4), Ok(None));
        assert_eq!(Hello::parse(&input5), Ok(None));
    }
}
