//  LIB.rs
//    by Lut99
//
//  Description:
//!   TODO
//

// Declare the modules
#[cfg(feature = "loc")]
pub mod loc;
#[cfg(feature = "tree")]
pub mod tree;

/// Represents all the interfaces that are often used when working with the AST-toolkit and are
/// thus useful to "import by default".
pub mod prelude {
    #[cfg(feature = "loc")]
    pub use super::loc::prelude::*;
    #[cfg(feature = "tree")]
    pub use super::tree::prelude::*;
}
