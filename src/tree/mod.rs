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

// Imports & Exports
use std::any::Any;
use std::cell::{Ref, RefMut};
use std::rc::Rc;
use std::sync::{Arc, MutexGuard, RwLockReadGuard, RwLockWriteGuard};

#[cfg(feature = "proc-macros")]
pub use ast_toolkit2_proc_macros::Node;

use crate::loc::Located;

/// Shorthand for including all the traits of this crate.
pub mod prelude {
    pub use super::{Node, NodeMut};
}


/***** HELPER MACROS *****/
/// Does pointer-like implementations for [`Node`].
macro_rules! node_ptr_impl {
    (__impl) => {
        #[inline(always)]
        fn as_any(&self) -> &dyn Any { <T as Node>::as_any(self) }
    };

    ('a, $ty:ty) => {
        impl<'a, T: Node> Node for $ty {
            node_ptr_impl!(__impl);
        }
    };
    ($ty:ty) => {
        impl<T: Node> Node for $ty {
            node_ptr_impl!(__impl);
        }
    };
}

/// Does pointer-like implementations for [`NodeMut`].
macro_rules! node_mut_ptr_impl {
    (__impl) => {
        #[inline(always)]
        fn as_any_mut(&mut self) -> &mut dyn Any { <T as NodeMut>::as_any_mut(self) }
    };

    ('a, $ty:ty) => {
        impl<'a, T: NodeMut> NodeMut for $ty {
            node_mut_ptr_impl!(__impl);
        }
    };
    ($ty:ty) => {
        impl<T: NodeMut> NodeMut for $ty {
            node_mut_ptr_impl!(__impl);
        }
    };
}





/***** INTERFACE *****/
/// Defines a generic node in your AST.
///
/// The purpose of this node is to, eventually, allow traversal using the
/// [`Visit`](crate::visit::Visit)-trait. However, concretely, it allows you to downcast a dynamic
/// trait object of it to a concrete object.
pub trait Node: Located {
    /// Returns this Node as an [`Any`].
    ///
    /// This will allow downcasting/upcasting later. Used internally when visiting.
    fn as_any(&self) -> &dyn Any;

    /// Convenience function that attempts to downcast `self` to `T` by going through [`Any`].
    ///
    /// # Generics
    /// - `T`: The type to attempt to downcast ourself as.
    ///
    /// # Returns
    /// Self downcasted if we were a supertype of `T`, or else [`None`].
    #[inline]
    fn as_t<T: 'static>(&self) -> Option<&T> { self.as_any().downcast_ref::<T>() }
}

// Std impls
impl<T: 'static + Located> Node for Vec<T> {
    #[inline(always)]
    fn as_any(&self) -> &dyn Any { self as &dyn Any }
}

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



/// Defines a generic node in your AST that can be mutable.
///
/// The purpose of this node is to, eventually, allow traversal using the
/// [`Visit`](crate::visit::Visit)-trait. However, concretely, it allows you to downcast a dynamic
/// trait object of it to a concrete object.
pub trait NodeMut: Node {
    /// Returns this Node as a mutable [`Any`].
    ///
    /// This will allow downcasting/upcasting later. Used internally when visiting.
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Convenience function that attempts to downcast `self` to `T` mutably by going through
    /// [`Any`].
    ///
    /// # Generics
    /// - `T`: The type to attempt to downcast ourself as.
    ///
    /// # Returns
    /// Self downcasted if we were a supertype of `T`, or else [`None`].
    #[inline]
    fn as_t_mut<T: 'static>(&mut self) -> Option<&mut T> { self.as_any_mut().downcast_mut::<T>() }
}

// Std impls
impl<T: 'static + Located> NodeMut for Vec<T> {
    #[inline(always)]
    fn as_any_mut(&mut self) -> &mut dyn Any { self as &mut dyn Any }
}

// Pointer-like impls
node_mut_ptr_impl!('a, &'a mut T);
node_mut_ptr_impl!(Box<T>);
node_mut_ptr_impl!('a, RefMut<'a, T>);
node_mut_ptr_impl!('a, MutexGuard<'a, T>);
node_mut_ptr_impl!('a, RwLockWriteGuard<'a, T>);
