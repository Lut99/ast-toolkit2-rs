//  SPEC.rs
//    by Lut99
//
//  Description:
//!   Defines auxillary interfaces useful with [`Loc`]s.
//

use std::convert::Infallible;
use std::hint::unreachable_unchecked;

use super::Loc;


/***** LIBRARY *****/
/// Defines an object that is tied to a location (in a source text).
pub trait Located {
    /// Returns the [`Loc`] that links this _entire_ object to the source.
    ///
    /// The idea is that this function internally combines- or shrinks [`Loc`]s until it has one
    /// "representing it as a whole". I.e., the returned [`Loc`] would suffice to refer to "this"
    /// object in the source code.
    ///
    /// # Returns
    /// A representable [`Loc`] for this object.
    fn loc(&self) -> Loc;
}

// Std impls
impl Located for Infallible {
    /// # Safety
    /// This function, while defined, is complete unreachable! This because [`Infallible`] objects
    /// are impossible to construct, and hence, no reference to `self` can exist to call this
    /// function with.
    ///
    /// **Calling this function anyway is considered undefined behaviour.**
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    fn loc(&self) -> Loc {
        #[cfg(debug_assertions)]
        panic!(
            "Whoa there, tiger! You're doing something very hacky there! It should be impossible for you to actually call this function! Without \
             `debug_assertions`, this becomes UNDEFINED BEHAVIOUR!!"
        );
        #[cfg_attr(debug_assertions, allow(unreachable_code))]
        unsafe {
            unreachable_unchecked()
        }
    }
}
