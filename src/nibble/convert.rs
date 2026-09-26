//  CONVERT.rs
//    by Lut99
//
//  Description:
//!   Plug for automatically translating values for literals.
//

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FResult};
use std::str::FromStr as _;


/***** HELPER MACROS *****/
macro_rules! convert_uint_impl {
    ($ty:ty) => {
        impl Convert for $ty {
            type Error = ParseIntError;

            #[inline]
            fn try_parse(slice: &[u8]) -> Result<Self, Self::Error> {
                let mut val: $ty = 0;
                for &b in slice {
                    // Convert to a value
                    if b < b'0' || b > b'9' {
                        return Err(ParseIntError::IllegalChar(b));
                    }
                    let i: $ty = (b - b'0') as $ty;

                    // Add to `val` with overflow checks
                    if val > <$ty>::MAX / 10 {
                        return Err(ParseIntError::Overflow(std::mem::size_of::<$ty>(), slice.into()));
                    }
                    val *= 10;
                    if val > <$ty>::MAX - i {
                        return Err(ParseIntError::Overflow(std::mem::size_of::<$ty>(), slice.into()));
                    }
                    val += i;
                }
                Ok(val)
            }
        }
    };
}

macro_rules! convert_sint_impl {
    ($ty:ty) => {
        impl Convert for $ty {
            type Error = ParseIntError;

            #[inline]
            fn try_parse(slice: &[u8]) -> Result<Self, Self::Error> {
                // Parse sign flips
                let mut i: usize = 0;
                let mut sign: $ty = 1;
                while i < slice.len() {
                    if slice[i] == b'-' {
                        sign *= -1;
                        i += 1;
                    } else {
                        break;
                    }
                }

                // Parse as before, keeping sign in mind
                let mut val: $ty = 0;
                for &b in slice {
                    // Sign

                    // Convert to a value
                    if b < b'0' || b > b'9' {
                        return Err(ParseIntError::IllegalChar(b));
                    }
                    let i: $ty = sign * (b - b'0') as $ty;

                    // Add to `val` with overflow checks
                    if sign > 0 {
                        if val > <$ty>::MAX / 10 {
                            return Err(ParseIntError::Overflow(std::mem::size_of::<$ty>(), slice.into()));
                        }
                    } else {
                        if val < <$ty>::MIN / 10 {
                            return Err(ParseIntError::Underflow(std::mem::size_of::<$ty>(), slice.into()));
                        }
                    }
                    val *= 10;

                    if sign > 0 {
                        if val > <$ty>::MAX - i {
                            return Err(ParseIntError::Overflow(std::mem::size_of::<$ty>(), slice.into()));
                        }
                    } else {
                        if val < <$ty>::MIN - i {
                            return Err(ParseIntError::Underflow(std::mem::size_of::<$ty>(), slice.into()));
                        }
                    }
                    val += i;
                }
                Ok(val)
            }
        }
    };
}

macro_rules! convert_float_impl {
    ($ty:ty) => {
        impl Convert for $ty {
            type Error = ParseFloatError;

            #[inline]
            fn try_parse(slice: &[u8]) -> Result<Self, Self::Error> {
                // Convert to UTF8 first (not that cheap, sadly)
                let slice: &str = std::str::from_utf8(slice)?;
                // Then handle the float parser
                <$ty>::from_str(slice).map_err(ParseFloatError::Float)
            }
        }
    };
}





/***** ERRORS *****/
/// Defines errors occurring when [`Convert`]ing unsigned- or signed integer primitives.
#[derive(Debug, Eq, PartialEq)]
pub enum ParseIntError {
    IllegalChar(u8),
    Overflow(usize, Vec<u8>),
    Underflow(usize, Vec<u8>),
}

impl Display for ParseIntError {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        match self {
            Self::IllegalChar(c) => {
                write!(f, "Illegal unsigned int character {:?}", char::from_u32(*c as u32).unwrap_or(char::REPLACEMENT_CHARACTER))
            },
            Self::Overflow(size, r) => {
                write!(f, "Unsigned int {:?} overflows for a {}-bit number", String::from_utf8_lossy(r), size * 8)
            },
            Self::Underflow(size, r) => {
                write!(f, "Unsigned int {:?} underflows for a {}-bit number", String::from_utf8_lossy(r), size * 8)
            },
        }
    }
}
impl Error for ParseIntError {}



/// Defines errors occurring when [`Convert`]ing floating-point primitives.
#[derive(Debug, Eq, PartialEq)]
pub enum ParseFloatError {
    Float(std::num::ParseFloatError),
    Utf8(std::str::Utf8Error),
}

impl Display for ParseFloatError {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        match self {
            Self::Float(e) => Display::fmt(e, f),
            Self::Utf8(e) => Display::fmt(e, f),
        }
    }
}
impl Error for ParseFloatError {
    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Float(e) => e.source(),
            Self::Utf8(e) => e.source(),
        }
    }
}

impl From<std::str::Utf8Error> for ParseFloatError {
    #[inline]
    fn from(value: std::str::Utf8Error) -> Self { Self::Utf8(value) }
}
impl From<std::num::ParseFloatError> for ParseFloatError {
    #[inline]
    fn from(value: std::num::ParseFloatError) -> Self { Self::Float(value) }
}





/***** LIBRARY *****/
/// A trait for parsing things from byte slices.
///
/// This is used when deriving [`Parsable`](super::Parsable) for literals.
pub trait Convert: Sized {
    type Error: Error;

    /// Attempts to parse Self from the given byte slice.
    ///
    /// # Arguments
    /// - `slice`: The slice to parse from. Note that it is expected your type can parse the
    ///   *whole* slice.
    ///
    /// # Returns
    /// A parsed Self.
    ///
    /// # Errors
    /// This function can error whenever `slice` is not valid `Self`.
    fn try_parse(slice: &[u8]) -> Result<Self, Self::Error>;
}

// Std impls
convert_uint_impl!(u8);
convert_uint_impl!(u16);
convert_uint_impl!(u32);
convert_uint_impl!(u64);
convert_uint_impl!(u128);
convert_uint_impl!(usize);
convert_sint_impl!(i8);
convert_sint_impl!(i16);
convert_sint_impl!(i32);
convert_sint_impl!(i64);
convert_sint_impl!(i128);
convert_sint_impl!(isize);
convert_float_impl!(f32);
convert_float_impl!(f64);
