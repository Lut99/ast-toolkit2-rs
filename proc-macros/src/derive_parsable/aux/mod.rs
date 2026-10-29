//  AUXILLARY.rs
//    by Lut99
//
//  Description:
//!   Defines auxillary types needed to generate `Parsable`.
//

// Modules
pub mod err;
pub mod fmt;
mod utils;

// Re-exports
pub use err::{Err, ErrInfo};
pub use fmt::{Fmt, FmtInfo};
pub use utils::*;
