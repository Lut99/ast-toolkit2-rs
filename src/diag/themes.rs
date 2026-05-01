//  THEMES.rs
//    by Lut99
//
//  Description:
//!   Defines the base [`Theme`] trait, as well as a few default themes coming
//!   with the library.
//

use std::fmt::{Debug, Formatter, Result as FResult};


/***** LIBRARY *****/
/// Defines a theme for customizing layouting of source snippets with their annotations.
///
/// To build your own, simply create this struct yourself or start with a provided theme (one of
/// the constants attached to this type) and go from there.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Theme {}

// Provided themes
impl Theme {
    /// The plain theme does not color anything, and is hence the most basic theme you can find.
    pub const PLAIN: Self = Self {};

    /// Simulates colors as used by the Rust compiler.
    pub const RUST: Self = Self {};
}

// Ops
impl Debug for Theme {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        if self == &Self::PLAIN {
            write!(f, "Theme::PLAIN")
        } else if self == &Self::RUST {
            write!(f, "Theme::RUST")
        } else {
            let Self {} = self;
            let mut fmt = f.debug_struct("Theme::CUSTOM");
            fmt.finish()
        }
    }
}
