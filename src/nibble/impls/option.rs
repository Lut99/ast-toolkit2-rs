//  OPTION.rs
//    by Lut99
//
//  Description:
//!   Provides a [`Parsable`] implementation for an [`Option`].
//

use std::fmt::{Display, Formatter, Result as FResult};

use super::super::slice::Slice;
use super::super::{NibbleError, Parsable};


/***** FORMATTERS *****/
/// Formatter for [`Option::expects()`].
#[derive(Debug, Eq, PartialEq)]
pub struct OptionFormatter<F> {
    fmt: F,
}
impl<F: Display> Display for OptionFormatter<F> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        write!(f, "optionally ")?;
        Display::fmt(&self.fmt, f)
    }
}
impl<F> From<F> for OptionFormatter<F> {
    #[inline]
    fn from(value: F) -> Self { Self { fmt: value } }
}





/***** IMPL *****/
impl<T: Parsable<E>, E> Parsable<E> for Option<T> {
    type Formatter = OptionFormatter<T::Formatter>;
    type Error = T::Error;

    #[inline]
    fn expects() -> Self::Formatter { OptionFormatter { fmt: T::expects() } }

    /// NOTE: This parser can never be [`NibbleError::Unmatched`] (as it will simply return
    /// [`None`] then).
    #[inline]
    fn parse(input: Slice<E>) -> Result<(Self, Slice<E>), NibbleError<Self::Formatter, Self::Error>> {
        match T::parse(input) {
            Ok((res, rem)) => Ok((Some(res), rem)),
            Err(NibbleError::Unmatched(_, _)) => Ok((None, input)),
            Err(NibbleError::Error(err)) => Err(NibbleError::Error(err)),
        }
    }
}
