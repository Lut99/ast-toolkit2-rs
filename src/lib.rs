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
#[cfg(feature = "print")]
pub mod print;
#[cfg(feature = "punct")]
pub mod punct;
#[cfg(feature = "tree")]
pub mod tree;

// Aliases
#[cfg(feature = "parse")]
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
    #[cfg(feature = "print")]
    pub use super::print::prelude::*;
    #[cfg(feature = "punct")]
    pub use super::punct::prelude::*;
    #[cfg(feature = "tree")]
    pub use super::tree::prelude::*;
}

/// Represents re-exports of macros.
#[cfg(feature = "macros")]
pub mod macros {
    #[cfg(all(feature = "proc-macros", feature = "loc"))]
    pub use ast_toolkit2_proc_macros::Located;
    #[cfg(all(feature = "proc-macros", feature = "tree"))]
    pub use ast_toolkit2_proc_macros::{Node, NonTerm, Term};
}
