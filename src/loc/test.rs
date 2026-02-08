//  TEST.rs
//    by Lut99
//
//  Description:
//!   Defines test helpers for the [`Loc`].
//!
//!   Specifically, contributes [`TestLoc`] which, unlike the normal [`Loc`],
//!   implements `eq` etc strictly.
//

use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};

use super::Loc;


/***** LIBRARY *****/
/// A wrapper around a normal [`Loc`] that implements [`Eq`], [`Hash`], and [`PartialEq`] strictly.
///
/// That is, where the normal [`Loc`]'s assume they are all the same, this one writes custom
/// operators that don't. This is mostly useful for checking if you parsed a Loc correctly, for
/// example.
#[derive(Clone, Copy, Debug)]
pub struct TestLoc(pub Loc);

// Constructors
impl TestLoc {
    /// Initializes a TestLoc around an empty [`Loc`].
    ///
    /// # Returns
    /// A TestLoc with the result of [`Loc::new()`] in it.
    #[inline]
    pub const fn new() -> Self { Self(Loc::new()) }
}

// Ops
impl Eq for TestLoc {}
impl Hash for TestLoc {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        let Loc { source, range } = &self.0;
        source.hash(state);
        range.hash(state);
    }
}
impl PartialEq for TestLoc {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let Loc { source, range } = &self.0;
        source == &other.0.source && range == &other.0.range
    }
}

// Conversion
impl Deref for TestLoc {
    type Target = Loc;

    #[inline]
    fn deref(&self) -> &Self::Target { &self.0 }
}
impl DerefMut for TestLoc {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}
impl AsRef<Loc> for TestLoc {
    #[inline]
    fn as_ref(&self) -> &Loc { &self.0 }
}
impl From<Loc> for TestLoc {
    #[inline]
    fn from(value: Loc) -> Self { Self(value) }
}
impl From<TestLoc> for Loc {
    #[inline]
    fn from(value: TestLoc) -> Self { value.0 }
}
