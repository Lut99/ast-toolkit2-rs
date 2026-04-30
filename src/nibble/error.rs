//  ERROR.rs
//    by Lut99
//
//  Description:
//!   Defines errors for this crate.
//

use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FResult};

use crate::loc::test::TestLoc;
use crate::loc::{Loc, Located};


/***** PRELUDE *****/
/// Trait for conveniently calling [`NibbleError`]'s map functions through a [`Result`].
pub trait ResultExt<T, F, E> {
    /// Transforms any [`NibbleError::Unmatched()`] into an [`Option`] for convenience.
    ///
    /// # Returns
    /// The same but now the result is [`Some`] if it wasn't an error, or [`None`] if
    /// [`NibbleError::Unmatched`] or [`NibbleError::NotEnough`] was the error. It still returns
    /// [`Err`] if the error was [`NibbleError::Error`] (but now directly with the error).
    fn transpose(self) -> Result<Option<T>, E>;



    /// Allows one to call [`NibbleError::into_atleast()`] through a [`Result`].
    ///
    /// # Returns
    /// A new instance of `Self` with any internal [`Needed::Bounded`] mapped to a
    /// [`Needed::AtLeast`].
    fn into_atleast(self) -> Result<T, NibbleError<F, E>>;

    /// Allows one to call [`NibbleError::auto_map()`] through a [`Result`].
    ///
    /// # Returns
    /// An equivalent error but with a mapped `F`.
    fn auto_map<F2: From<F>, E2: From<E>>(self) -> Result<T, NibbleError<F2, E2>>;

    /// Allows one to call [`NibbleError::map_fmt()`] through a [`Result`].
    ///
    /// # Arguments
    /// - `map`: Some [`FnOnce`] doing the mapping.
    ///
    /// # Returns
    /// An equivalent error but with a mapped `F`.
    fn map_fmt<F2, E2: From<E>>(self, map: impl FnOnce(F) -> F2) -> Result<T, NibbleError<F2, E2>>;

    /// Allows one to call [`NibbleError::map_err()`] through a [`Result`].
    ///
    /// # Arguments
    /// - `map`: Some [`FnOnce`] doing the mapping.
    ///
    /// # Returns
    /// An equivalent error but with a mapped `E`.
    fn map_nerr<F2: From<F>, E2>(self, map: impl FnOnce(E) -> E2) -> Result<T, NibbleError<F2, E2>>;
}
impl<T, F, E> ResultExt<T, F, E> for Result<T, NibbleError<F, E>> {
    #[inline]
    fn transpose(self) -> Result<Option<T>, E> {
        match self {
            Ok(res) => Ok(Some(res)),
            Err(NibbleError::Unmatched(_, _)) => Ok(None),
            Err(NibbleError::NotEnough(_, _, _)) => Ok(None),
            Err(NibbleError::Error(err)) => Err(err),
        }
    }



    #[inline]
    fn into_atleast(self) -> Result<T, NibbleError<F, E>> {
        match self {
            Ok(res) => Ok(res),
            Err(err) => Err(err.into_atleast()),
        }
    }
    #[inline]
    fn auto_map<F2: From<F>, E2: From<E>>(self) -> Result<T, NibbleError<F2, E2>> {
        match self {
            Ok(res) => Ok(res),
            Err(err) => Err(err.auto_map()),
        }
    }

    #[inline]
    fn map_fmt<F2, E2: From<E>>(self, map: impl FnOnce(F) -> F2) -> Result<T, NibbleError<F2, E2>> {
        match self {
            Ok(res) => Ok(res),
            Err(err) => Err(err.map_fmt(map)),
        }
    }

    #[inline]
    fn map_nerr<F2: From<F>, E2>(self, map: impl FnOnce(E) -> E2) -> Result<T, NibbleError<F2, E2>> {
        match self {
            Ok(res) => Ok(res),
            Err(err) => Err(err.map_nerr(map)),
        }
    }
}





/***** AUXILLARY *****/
/// Describes that and, ideally, how many additional input can be given for an unmatched parser to
/// turn into a matched one.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Needed {
    /// The parser tells you that, for it to match the input, it will need between .0 and .1 (both
    /// **inclusive**) additional elements in the slice for it to match the input.
    ///
    /// You usually see this with exact parsers. In that case, it may even be the case that .0 ==
    /// .1 (i.e., the parser knows exactly how much more it will need).
    Bounded(usize, usize),
    /// The parser tells you that, for it to match the input, it will need at least this many more
    /// elements in the slice. But it may be more!
    ///
    /// You usually see this with "greedy" parsers like [`Vec`].
    AtLeast(usize),
    /// The parser tells you that it needs more elements in the slice to match the input, but it
    /// doesn't know how many.
    Unknown,
}

// Needed
impl Needed {
    /// Yields an [`Iterator::size_hint()`]-like size hint based on how many additional elements
    /// are needed.
    ///
    /// # Returns
    /// A tuple with the minimum amount of elements to add (defaults to `0` if we don't know) and,
    /// if known, a maximum amount of elements to add, respectively.
    #[inline]
    pub const fn size_hint(&self) -> (usize, Option<usize>) {
        match *self {
            Self::Bounded(min, max) => (min, Some(max)),
            Self::AtLeast(min) => (min, None),
            Self::Unknown => (0, None),
        }
    }
}





/***** LIBRARY *****/
/// Defines the error of all error types: a nibble error.
///
/// The nibble error essentially classifies errors occuring while parsing into three possibilities:
/// - [`NibbleError::Unmatched`] represents that whatever you were trying to parse, was not matched,
///   meaning that it may yet still be something else;
/// - [`NibbleError::NotEnough`] represents that whatever you were trying to parse _looks_ like it
///   is this thing, but we'd need more input to know for sure (implying that an end-of-input was
///   encountered before the match could be made); and
/// - [`NibbleError::Error`] represents that whatever you were trying to parse was matched but
///   illegal somehow, meaning that it _couldn't_ have been something else.
#[derive(Debug)]
pub enum NibbleError<F, E> {
    /// Represents that what you were trying to parse was not recognized.
    ///
    /// This implies something else might still parse this bit successfully. However, it typically
    /// implies that more input does _not_ change the outcome; i.e., there was something, but what
    /// there was, was wrong.
    ///
    /// Consider using [`NibbleError::NotEnough`] instead if you want to communicate that the
    /// problem was that there was not enough input, and more input may rectify the issue.
    Unmatched(F, Loc),
    /// Represents that what you were trying to parse wasn't recognized but not due to incorrect
    /// input, but rather a lack of it.
    ///
    /// It communicates to the user that more input may cause this branch to succeed instead.
    NotEnough(F, Needed, Loc),
    /// Represents that what you were trying to parse was recognized, but illegal.
    ///
    /// This implies something else won't parse this bit successfully either.
    ///
    /// The field is the nested error further describing what went wrong.
    Error(E),
}

// Mappers
impl<F, E> NibbleError<F, E> {
    /// If this NibbleError is a [`NibbleError::Unmatched`], then the [`Needed`] within it is
    /// transformed from a [`Needed::Bounded`] to a [`Needed::AtLeast`].
    ///
    /// This seems like a hyper-specific usecase and it is. However, it occurs quite a lot when
    /// using "fixed-size" parsers (e.g., parsing a tag) in greedy parsers (e.g., [`Vec`]).
    /// Especially when they are prefixed, it pays to parse the prefix as a tag (with fixed size)
    /// but report it as a minimum-size requirement.
    ///
    /// # Returns
    /// A new instance of `Self` with any internal [`Needed::Bounded`] mapped to a
    /// [`Needed::AtLeast`].
    #[inline]
    pub fn into_atleast(self) -> NibbleError<F, E> {
        match self {
            Self::NotEnough(fmt, Needed::Bounded(min, _), loc) => Self::NotEnough(fmt, Needed::AtLeast(min), loc),
            err => err,
        }
    }

    /// Powerful version of a map function that attempts to automatically convert based on
    /// available [`From`]-implementations.
    ///
    /// # Returns
    /// A new instance of `Self` with `F` and `E` mapped.
    #[inline]
    pub fn auto_map<F2: From<F>, E2: From<E>>(self) -> NibbleError<F2, E2> {
        match self {
            Self::Unmatched(fmt, loc) => NibbleError::Unmatched(fmt.into(), loc),
            Self::NotEnough(fmt, needed, loc) => NibbleError::NotEnough(fmt.into(), needed, loc),
            Self::Error(err) => NibbleError::Error(err.into()),
        }
    }

    /// Maps the `F`ormatter in this NibbleError to something else.
    ///
    /// Does nothing if this is not a [`NibbleError::Unmatched`].
    ///
    /// # Arguments
    /// - `map`: Some closure mapping `F` to something else.
    ///
    /// # Returns
    /// A new instance of `Self` with `F` mapped.
    ///
    /// Note that `E` is also implicitly converted with its [`From`]-implementation!
    #[inline]
    pub fn map_fmt<F2, E2: From<E>>(self, map: impl FnOnce(F) -> F2) -> NibbleError<F2, E2> {
        match self {
            Self::Unmatched(fmt, loc) => NibbleError::Unmatched(map(fmt), loc),
            Self::NotEnough(fmt, needed, loc) => NibbleError::NotEnough(map(fmt), needed, loc),
            Self::Error(err) => NibbleError::Error(err.into()),
        }
    }

    /// Maps the `E`rror in this NibbleError to something else.
    ///
    /// Does nothing if this is not a [`NibbleError::Error`].
    ///
    /// # Arguments
    /// - `map`: Some closure mapping `E` to something else.
    ///
    /// # Returns
    /// A new instance of `Self` with `E` mapped.
    ///
    /// Note that `F` is also implicitly converted with its [`From`]-implementation!
    #[inline]
    pub fn map_nerr<F2: From<F>, E2>(self, map: impl FnOnce(E) -> E2) -> NibbleError<F2, E2> {
        match self {
            Self::Unmatched(fmt, loc) => NibbleError::Unmatched(fmt.into(), loc),
            Self::NotEnough(fmt, needed, loc) => NibbleError::NotEnough(fmt.into(), needed, loc),
            Self::Error(err) => NibbleError::Error(map(err)),
        }
    }
}

// Error
impl<F: Display, E: Display> Display for NibbleError<F, E> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        match self {
            Self::Unmatched(fmt, _) => Display::fmt(fmt, f),
            Self::NotEnough(_, Needed::Bounded(min, max), _) if min == max => {
                write!(f, "Unexpected end of input (expected {min} elements more)")
            },
            Self::NotEnough(_, Needed::Bounded(min, max), _) => {
                write!(f, "Unexpected end of input (expected at least {min}, at most {max} elements more)")
            },
            Self::NotEnough(_, Needed::AtLeast(min), _) => write!(f, "Unexpected end of input (expected at least {min} elements more)"),
            Self::NotEnough(_, Needed::Unknown, _) => write!(f, "Unexpected end of input"),
            Self::Error(err) => Display::fmt(err, f),
        }
    }
}
impl<F: Debug + Display, E: Error> Error for NibbleError<F, E> {
    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Unmatched(_, _) => None,
            Self::NotEnough(_, _, _) => None,
            Self::Error(err) => err.source(),
        }
    }
}
impl<F, E> From<E> for NibbleError<F, E> {
    #[inline]
    fn from(value: E) -> Self { Self::Error(value) }
}

// Ops
impl<F: Eq, E: Eq> Eq for NibbleError<F, E> {}
impl<F: PartialEq, E: PartialEq> PartialEq for NibbleError<F, E> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Unmatched(fmt1, loc1), Self::Unmatched(fmt2, loc2)) => fmt1 == fmt2 && TestLoc(*loc1) == TestLoc(*loc2),
            (Self::NotEnough(fmt1, needed1, loc1), Self::NotEnough(fmt2, needed2, loc2)) => {
                fmt1 == fmt2 && needed1 == needed2 && TestLoc(*loc1) == TestLoc(*loc2)
            },
            (Self::Error(lhs), Self::Error(rhs)) => lhs == rhs,
            _ => false,
        }
    }
}

// Loc
impl<F, E: Located> Located for NibbleError<F, E> {
    #[inline]
    fn loc(&self) -> Loc {
        match self {
            Self::Unmatched(_, loc) => *loc,
            Self::NotEnough(_, _, loc) => *loc,
            Self::Error(err) => err.loc(),
        }
    }
}
