//  LIB.rs
//    by Lut99
//
//  Description:
//!   TODO
//

// Declare the modules
#[cfg(feature = "loc")]
pub mod loc;
#[cfg(feature = "nibble")]
pub mod nibble;
#[cfg(feature = "tree")]
pub mod tree;

// Aliases
pub use nibble as parse;

/// Represents all the interfaces that are often used when working with the AST-toolkit and are
/// thus useful to "import by default".
pub mod prelude {
    #[cfg(feature = "loc")]
    pub use super::loc::prelude::*;
    #[cfg(feature = "nibble")]
    pub use super::nibble::prelude::*;
    #[cfg(feature = "tree")]
    pub use super::tree::prelude::*;
}
