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
use std::cell::{Ref, RefMut};
use std::rc::Rc;
use std::sync::{Arc, MutexGuard, RwLockReadGuard, RwLockWriteGuard};

// Re-export some node macros
#[cfg(feature = "proc-macros")]
pub use ast_toolkit2_proc_macros::{Node, NonTerm, Term};

use crate::loc::Located;

/// Shorthand for including all the traits of this crate.
pub mod prelude {
    pub use super::*;
}


/***** HELPER MACROS *****/
/// Does pointer-like implementations for [`Node`].
macro_rules! node_ptr_impl {
    ('a, $ty:ty) => {
        impl<'a, T: Node> Node for $ty {}
    };
    ($ty:ty) => {
        impl<T: Node> Node for $ty {}
    };
}

/// Does pointer-like implementations for [`NonTerm`].
macro_rules! nonterm_ptr_impl {
    ('a, $ty:ty) => {
        impl<'a, T: NonTerm> NonTerm for $ty {}
    };
    ($ty:ty) => {
        impl<T: NonTerm> NonTerm for $ty {}
    };
}

/// Does pointer-like implementations for [`Term`].
macro_rules! term_ptr_impl {
    ('a, $ty:ty) => {
        impl<'a, T: Term> Term for $ty {}
    };
    ($ty:ty) => {
        impl<T: Term> Term for $ty {}
    };
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

// Pointer-like impls
node_ptr_impl!('a, &'a T);
node_ptr_impl!('a, &'a mut T);
node_ptr_impl!(Box<T>);
node_ptr_impl!(Rc<T>);
node_ptr_impl!(Arc<T>);
node_ptr_impl!('a, Ref<'a, T>);
node_ptr_impl!('a, RefMut<'a, T>);
node_ptr_impl!('a, MutexGuard<'a, T>);
node_ptr_impl!('a, RwLockReadGuard<'a, T>);
node_ptr_impl!('a, RwLockWriteGuard<'a, T>);



/// Represents a "branch" [`Node`] in your AST.
///
/// Non-terminals are characterized by having children. They also tend to be agnostic to specific
/// syntax; rather, they tend to treat syntax as being tokenized, i.e., concerned with the count
/// and order of specific, already parsed, constructs rather than with e.g. whitespace. You can
/// think of them as an understanding of a stream of [`Term`]inals.
pub trait NonTerm: Node {}

// Pointer-like impls
nonterm_ptr_impl!('a, &'a T);
nonterm_ptr_impl!('a, &'a mut T);
nonterm_ptr_impl!(Box<T>);
nonterm_ptr_impl!(Rc<T>);
nonterm_ptr_impl!(Arc<T>);
nonterm_ptr_impl!('a, Ref<'a, T>);
nonterm_ptr_impl!('a, RefMut<'a, T>);
nonterm_ptr_impl!('a, MutexGuard<'a, T>);
nonterm_ptr_impl!('a, RwLockReadGuard<'a, T>);
nonterm_ptr_impl!('a, RwLockWriteGuard<'a, T>);



/// Represents a "leaf" [`Node`] in your AST.
///
/// Terminals are characterized by _not_ having children. They tend to relate very specifically to
/// syntax, and parsing them requires worrying about encodings, whitespaces, etc. You can think of
/// them forming a stream of the input, and [`NonTerm`]inals an understanding of that stream.
pub trait Term: Node {}

// Pointer-like impls
term_ptr_impl!('a, &'a T);
term_ptr_impl!('a, &'a mut T);
term_ptr_impl!(Box<T>);
term_ptr_impl!(Rc<T>);
term_ptr_impl!(Arc<T>);
term_ptr_impl!('a, Ref<'a, T>);
term_ptr_impl!('a, RefMut<'a, T>);
term_ptr_impl!('a, MutexGuard<'a, T>);
term_ptr_impl!('a, RwLockReadGuard<'a, T>);
term_ptr_impl!('a, RwLockWriteGuard<'a, T>);
