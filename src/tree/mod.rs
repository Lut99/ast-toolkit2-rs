//  MOD.rs
//    by Lut99
//
//  Description:
//!   Defines the absolute core interfaces for the AST-toolkit.
//!   
//!   Essentially, this gives you the handles to talk about AST's generically:
//!   - The [`Node`] defines any node in the tree, be it branches or leaves;
//!   - The [`NonTerm`] defines branches in the tree, i.e., nodes made up of
//!     other nodes. Very usually, these are extremely derivable; and
//!   - The [`Term`] defines leaves in the tree, i.e., elementary concepts
//!     (think literals or identifiers). Very usually, these require a bit more
//!     detailled work.
//

// Modules
mod tag;

// Re-export some node macros
#[cfg(feature = "macros")]
pub use ast_toolkit2_macros::{Node, NonTerm, Term};
pub use tag::Tag;

use crate::loc::Located;

/// Shorthand for including all the traits of this crate.
pub mod prelude {
    pub use super::*;
}


/***** INTERFACE *****/
/// Defines a generic node in your AST.
///
/// Note that nodes, in general, come in two flavours:
/// - [`NonTerm`]inals represent "branches" in your tree. They tend to be less concerned with
///   specific syntax but rather with order, count, etc. I.e., you can imagine that a more abstract
///   parser than a lexer parses [`Term`]inals.
///
///   Non-terminals explicitly _always_ have children.
/// - [`Term`]inals represent "leafs" in your tree. They often have a concrete, meaningful syntax
///   (e.g., they represent keywords, identifiers or literal values) and tend to be whitespace-
///   sensitive. You can imagine a lexer is used to parse these.
///
///   Terminals explicitly _don't_ have any children.
///
/// Despite this difference, this trait represents the general part of the two.
pub trait Node: Located {}



/// Represents a "branch" [`Node`] in your AST.
///
/// Non-terminals are characterized by having children. They also tend to be agnostic to specific
/// syntax; rather, they tend to treat syntax as being tokenized, i.e., concerned with the count
/// and order of specific, already parsed, constructs rather than with e.g. whitespace. You can
/// think of them as an understanding of a stream of [`Term`]inals.
pub trait NonTerm: Node {}



/// Represents a "leaf" [`Node`] in your AST.
///
/// Terminals are characterized by _not_ having children. They tend to relate very specifically to
/// syntax, and parsing them requires worrying about encodings, whitespaces, etc. You can think of
/// them forming a stream of the input, and [`NonTerm`]inals an understanding of that stream.
pub trait Term: Node {}
