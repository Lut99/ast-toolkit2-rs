//  LIB.rs
//    by Lut99
//
//  Description:
//!   TODO
//

// Declare the modules
pub mod spec;

/// Represents all the interfaces that are often used when working with the AST-toolkit and are
/// thus useful to "import by default".
pub mod prelude {
    pub use super::spec::*;
}
