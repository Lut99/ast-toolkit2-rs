//  MOD.rs
//    by Lut99
//
//  Description:
//!   Defines the absolute core interfaces for the AST-toolkit.
//

// We secretly divide this module
mod node;
mod span;

// Pretend everything's here
pub use node::{Node, NonTerm, Term};
pub use span::{Span, Spanning};
