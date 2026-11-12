//  MOD.rs
//    by Lut99
//
//  Description:
//!   Defines traits used for traversing a tree.
//!
//!   The traversion is done statically, but through dynamic objects instead of
//!   real objects. Why this is the case
//!   [is a long discussion](https://github.com/Lut99/ast-toolkit2-rs/issues/2),
//!   but suffice to say that it's the best solution.
//!
//!   Like other traits in this library, a
//!   [`Visit`](derive@Visit)/[`VisitMut`](derive@VisitMut)/
//!   [`VisitOwned`](derive@VisitOwned) derive macro exists.
//

// Modules
mod composable;
mod impls;
mod replace_with;

// Imports & Exports
use std::cell::{Ref, RefMut};
use std::rc::Rc;
use std::sync::{Arc, MutexGuard, RwLockReadGuard, RwLockWriteGuard};

pub use composable::{Composable, Initializable};
pub use replace_with::ReplaceWith;

use crate::tree::{Node, NodeMut};

/// Shorthand for including all the traits of this crate.
pub mod prelude {
    pub use super::{Composable, Initializable, ReplaceWith, Visit, VisitMut};
}


/***** HELPER MACROS *****/
macro_rules! visit_ptr_impl {
    (__impl) => {
        #[inline(always)]
        fn visit<V: Visitor>(&self, visitor: &mut V) -> V::Result { <T as Visit>::visit(self, visitor) }
    };

    ('a, $ty:ty) => {
        impl<'a, T: Visit> Visit for $ty {
            visit_ptr_impl!(__impl);
        }
    };
    ($ty:ty) => {
        impl<T: Visit> Visit for $ty {
            visit_ptr_impl!(__impl);
        }
    };
}

macro_rules! visit_mut_ptr_impl {
    (__impl) => {
        #[inline(always)]
        fn visit_mut<V: VisitorMut>(&mut self, visitor: &mut V) -> V::Result { <T as VisitMut>::visit_mut(self, visitor) }
    };

    ('a, $ty:ty) => {
        impl<'a, T: VisitMut> VisitMut for $ty {
            visit_mut_ptr_impl!(__impl);
        }
    };
    ($ty:ty) => {
        impl<T: VisitMut> VisitMut for $ty {
            visit_mut_ptr_impl!(__impl);
        }
    };
}





/***** INTERFACES *****/
/// Means that this node can be traversed and all of its children found.
///
/// Lovingly also called the "babysitter" trait.
///
/// This trait allows read-only, by-reference access to the children. See [`VisitMut`] for mutable,
/// by-reference access or [`VisitOwned`] for consuming, owned access.
///
/// # Generics
/// - `R`: Some [`Default`] and [`Composable`] type that is used to carry results of the visiting
///   process. A value of `()` means nothing is returned.
pub trait Visit: Node {
    /// Visits the node "standardly", i.e., traverses its children without visiting the node
    /// itself.
    ///
    /// This function should be implemented to call [`Visitor::visit()`] on each of its direct
    /// children. This makes every node responsible for traversing itself "by default".
    ///
    /// # Arguments
    /// - `visitor`: A [`Visitor`] that is programmed with traversal-specific visits for some
    ///   nodes. If it hasn't got one for the node you give it, it will instead defer to that
    ///   node's [`Visit`] implementation.
    ///
    /// # Returns
    fn visit<V: Visitor>(&self, visitor: &mut V) -> V::Result;
}

// Pointer-like impls
visit_ptr_impl!('a, &'a T);
visit_ptr_impl!('a, &'a mut T);
visit_ptr_impl!(Box<T>);
visit_ptr_impl!(Rc<T>);
visit_ptr_impl!(Arc<T>);
visit_ptr_impl!('a, Ref<'a, T>);
visit_ptr_impl!('a, RefMut<'a, T>);
visit_ptr_impl!('a, MutexGuard<'a, T>);
visit_ptr_impl!('a, RwLockReadGuard<'a, T>);
visit_ptr_impl!('a, RwLockWriteGuard<'a, T>);



/// Means that this node can be traversed and all of its children found.
///
/// This trait allows mutable, by-reference access to the children. See [`Visit`] for read-only,
/// by-reference access or [`VisitOwned`] for consuming, owned access.
///
/// # Generics
/// - `R`: Some [`Default`] and [`Composable`] type that is used to carry results of the visiting
///   process. A value of `()` means nothing is returned.
pub trait VisitMut: NodeMut + Visit {
    /// Visits the node "standardly", i.e., traverses its children without visiting the node
    /// itself.
    ///
    /// This function should be implemented to call [`Visitor::visit()`] on each of its direct
    /// children. This makes every node responsible for traversing itself "by default".
    ///
    /// # Arguments
    /// - `visitor`: A [`Visitor`] that is programmed with traversal-specific visits for some
    ///   nodes. If it hasn't got one for the node you give it, it will instead defer to that
    ///   node's [`Visit`] implementation.
    ///
    /// # Returns
    fn visit_mut<V: VisitorMut>(&mut self, visitor: &mut V) -> V::Result;
}

// Pointer-like impls
visit_mut_ptr_impl!('a, &'a mut T);
visit_mut_ptr_impl!(Box<T>);
visit_mut_ptr_impl!('a, RefMut<'a, T>);
visit_mut_ptr_impl!('a, MutexGuard<'a, T>);
visit_mut_ptr_impl!('a, RwLockWriteGuard<'a, T>);



/// A trait for implementing something that has custom visit functions for a [`Node`].
pub trait Visitor {
    /// A type returned by traversal functions of this visitor.
    ///
    /// [`Composable`] (and hence [`Default`]) means that you can join two answers of two visiting
    /// functions and still get a meaningful result. For example, sum two counts.
    type Result: Composable;

    /// Visits the given node, using the custom implementation if available or else defaulting to
    /// the node's [`Visit`] implementation.
    ///
    /// # Arguments
    /// - `node`: The [`Visit`]able [`Node`] to visit.
    ///
    /// # Returns
    fn visit<N: Visit>(&mut self, node: &N) -> Self::Result;
}



/// A trait for implementing something that has custom visit functions for a [`Node`].
pub trait VisitorMut {
    /// A type returned by traversal functions of this visitor.
    ///
    /// [`Composable`] (and hence [`Default`]) means that you can join two answers of two visiting
    /// functions and still get a meaningful result. For example, sum two counts.
    type Result: Composable;

    /// Visits the given node, using the custom implementation if available or else defaulting to
    /// the node's [`VisitMut`] implementation.
    ///
    /// # Arguments
    /// - `node`: The [`VisitMut`]able [`Node`] to visit.
    ///
    /// # Returns
    fn visit_mut<N: VisitMut>(&mut self, node: &mut N) -> Self::Result;
}
