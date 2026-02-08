//  MOD.rs
//    by Lut99
//
//  Description:
//!   Defines the [`ParseStream`], an abstraction over something that will
//!   yield inputs.
//!
//!   See it for more information.
//

// Define some provided impls
pub mod copied;
pub mod slice;

use std::error::Error;

use super::{NibbleError, Parsable};
use crate::loc::Loc;


/***** LIBRARY *****/
/// Defines an abstraction over something parsable by nibble.
///
/// In general, think of the ParseStream as an efficient iterator: every instance represents a
/// unique position in some input stream, yielding elements of some type. This can be e.g.
/// individual bits, bytes, unicode characters or already parsed terminals. What matters is that
/// elements are sequential and parse to zero or more semantic ASTs.
///
/// # Assumptions
/// Users of the trait can assume, and implementors must make sure, that:
/// 1. Every instance of a `ParseStream` represents a specific position in the stream. Hence,
///    [`Clone`] one can be used to implement e.g. branching; and
/// 2. (For this reason) [`Clone`]ing should be relatively cheap, as it may happen often.
///
/// For the combination of these reasons, implementations will typically have some backing store
/// (e.g., an in-memory array or a buffered file) that each `ParseStream` provides a view on. To
/// avoid them having to always see the entire contents, the [`ParseStream::commit()`]-function
/// allows implementations to signal to the underlying backend that, up to a certain point, they
/// are confident that they won't have to read again. Think of this as the "front" of the buffer
/// being advanced.
pub trait ParseStream: Clone {
    /// The type of element that is yielded by the stream. Note that it may refer to the original
    /// input!
    type Elem;
    /// The type of error that can occur if [`ParseStream::next()`] fails.
    type Error: 'static + Error;


    // User-provided
    /// Get the next element in the stream.
    ///
    /// This is expected to yield the next item in this ParseStream's view. Note that it borrows
    /// from `self`, so likely, you won't want to refer to the returned element in any way in your
    /// AST. This is unencouraged anyway (trust me, plz), so instead just accept the (often
    /// negligible) penalty hit and clone the result. This is only yielded by reference as often,
    /// after parsing, some analysis has to be done to determine if what was parsed was useful.
    ///
    /// Note that `self` isn't mutable. This because else the lifetime management is impossible
    /// in parsing functions due to lifetimes become invariant or contravariant and such.
    ///
    /// # Returns
    /// An instance of [`ParseStream::Elem`] representing the yielded element, together with a
    /// [`Loc`] describing where in the input it was found; or [`None`] if the stream ended.
    ///
    /// # Errors
    /// This function may error if it failed to get the next item or information about whether it
    /// even exists. This can occur if the stream is backed by e.g. a file and the current position
    /// is not yet cached.
    fn next(self) -> Result<Option<(Self, Loc, Self::Elem)>, Self::Error>;

    /// Commits this current ParseStream's position as "the earliest you'll look."
    ///
    /// In essence, this allows any backing implementation to drop contents up to this
    /// ParseStream's point. Hence, it functions as a heuristic.
    ///
    /// For implementors: this means that, in the worst-case, **streams may still access elements
    /// before in the stream even after calling commit.** Well-behaved implementations are robust
    /// to this mistake, while well-behaved users avoid doing that.
    fn commit(self) -> Self;


    // Derived
    /// Provides a convenience function for getting something [`Parsable`] off of the input stream.
    ///
    /// This will simply attempt to advance the stream up to the point where the given [`Parsable`]
    /// is successfully parsed. If it fails, then this function fails, too.
    ///
    /// # Returns
    /// An instance of `P`, representing the [`Parsable`] object we successfully parsed.
    ///
    /// # Errors
    /// This function can error if we failed to parse `P`, for whatever reason it thinks.
    #[inline]
    fn parse<'s, P: Parsable<Self::Elem>>(self) -> Result<Option<(Self, P)>, NibbleError<P::Error, Self::Error>> { P::parse(self) }
}
