//  TAG.rs
//    by Lut99
//
//  Description:
//!   Defines the [`Tag`], a subset of [`Term`]s that have lots of
//!   implementation done for them.
//

use super::Term;
use crate::loc::Loc;


/***** LIBRARY *****/
/// A more specific version of a [`Term`] that is a single sequence of parsable elements.
///
/// For example: this might be a keyword or specific punctuation.
///
/// Implementing this on your type will automatically some traits like parsers.
pub trait Tag<E: 'static>: Sized + Term {
    /// The literal that we parse to find this keyword.
    const TAG: &'static [E];

    /// Constructor for the Tag.
    ///
    /// The default implementation simply refers to [`Utf8Tag::with_loc()`] with a [`Loc::new()`].
    ///
    /// # Returns
    /// A new instance of Self that is not tied to any Loc.
    #[inline]
    fn new() -> Self { Self::with_loc(Loc::new()) }

    /// Constructor for the Tag from a (parsed) [`Loc`].
    ///
    /// # Arguments
    /// - `loc`: A [`Loc`] describing where we parsed it from.
    ///
    /// # Returns
    /// A new instance of Self that is parsed from `loc`.
    fn with_loc(loc: Loc) -> Self;
}
