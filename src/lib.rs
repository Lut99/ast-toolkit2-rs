//  LIB.rs
//    by Lut99
//
//  Description:
//!   TODO
//

// Declare the modules
#[cfg(feature = "init")]
pub mod init;
#[cfg(feature = "loc")]
pub mod loc;
#[cfg(feature = "nibble")]
pub mod nibble;
#[cfg(feature = "tree")]
pub mod tree;

// Aliases
#[cfg(feature = "nibble")]
pub use nibble as parse;

/// Represents all the interfaces that are often used when working with the AST-toolkit and are
/// thus useful to "import by default".
pub mod prelude {
    #[cfg(feature = "init")]
    pub use super::init::prelude::*;
    #[cfg(feature = "loc")]
    pub use super::loc::prelude::*;
    #[cfg(feature = "nibble")]
    pub use super::nibble::prelude::*;
    #[cfg(feature = "tree")]
    pub use super::tree::prelude::*;
}

/// Represents re-exports of macros.
#[cfg(feature = "macros")]
pub mod macros {
    #[cfg(feature = "loc")]
    pub use ast_toolkit2_macros::Located;
    #[cfg(feature = "tree")]
    pub use ast_toolkit2_macros::Node;
}
