//  RENDERERS.rs
//    by Lut99
//
//  Description:
//!   Defines the builtin renderers.
//

// Modules
#[cfg(feature = "sources")]
mod hex;
#[cfg(feature = "sources")]
mod utf8;

// Imports & Re-exports
use std::cell::{Ref, RefMut};
use std::fmt;
use std::rc::Rc;
use std::sync::{Arc, MutexGuard, RwLockReadGuard, RwLockWriteGuard};

#[cfg(feature = "sources")]
pub use hex::HexRenderer;
#[cfg(feature = "sources")]
pub use utf8::Utf8Renderer;

use super::formatter::{main, replace, sourceless};


/***** HELPER MACROS *****/
macro_rules! renderer_ptr_impl {
    (__impl) => {
        fn render_main<'d>(&self, f: main::Formatter<'d, '_>, msg: main::Message<'d>) -> fmt::Result { <T as Renderer<E>>::render_main(self, f, msg) }

        fn render_replace<'d>(&self, f: replace::Formatter<'d, '_>, msg: replace::Message<'d, E>) -> fmt::Result {
            <T as Renderer<E>>::render_replace(self, f, msg)
        }

        fn render_sourceless<'d>(&self, f: sourceless::Formatter<'d, '_>, msg: sourceless::Message<'d>) -> fmt::Result {
            <T as Renderer<E>>::render_sourceless(self, f, msg)
        }
    };

    ('a, $ty:ty) => {
        impl<'a, E, T: ?Sized + Renderer<E>> Renderer<E> for $ty {
            renderer_ptr_impl!(__impl);
        }
    };
    ($ty:ty) => {
        impl<E, T: ?Sized + Renderer<E>> Renderer<E> for $ty {
            renderer_ptr_impl!(__impl);
        }
    };
}





/***** INTERFACES *****/
/// Interface that you can implement for a struct rendering source text.
pub trait Renderer<E> {
    /// Renders a [`MessageMain`] to the given `f`ormatter.
    ///
    /// # Arguments
    /// - `f`: A [`Formatter`] to write to.
    /// - `msg`: A single message to render. This one represents the main message and can have
    ///   zero or more _snippets_ of source text with zero or more _annotations_.
    ///
    /// # Errors
    /// This function should only error if it failed to interact with `f`.
    fn render_main<'d>(&self, f: main::Formatter<'d, '_>, msg: main::Message<'d>) -> fmt::Result;

    /// Renders a [`MessageReplace`] to the given `f`ormatter.
    ///
    /// # Arguments
    /// - `f`: A [`Formatter`] to write to.
    /// - `msg`: A single message to render. This one represents a single annotation showing a
    ///   suggested replacement.
    ///
    /// # Errors
    /// This function should only error if it failed to interact with `f`.
    fn render_replace<'d>(&self, f: replace::Formatter<'d, '_>, msg: replace::Message<'d, E>) -> fmt::Result;

    /// Renders a [`sourceless::Message`] to the given `f`ormatter.
    ///
    /// This function doesn't really require you to do anything except give the proper line width,
    /// if any.
    ///
    /// # Arguments
    /// - `f`: A [`sourceless::Formatter`] to write to.
    /// - `msg`: A single message to render. This one represents a single annotation showing a
    ///   suggested replacement.
    ///
    /// # Errors
    /// This function should only error if it failed to interact with `f`.
    fn render_sourceless<'d>(&self, f: sourceless::Formatter<'d, '_>, msg: sourceless::Message<'d>) -> fmt::Result;
}

// Pointer-like impls
renderer_ptr_impl!('a, &'a T);
renderer_ptr_impl!('a, &'a mut T);
renderer_ptr_impl!(Box<T>);
renderer_ptr_impl!(Rc<T>);
renderer_ptr_impl!(Arc<T>);
renderer_ptr_impl!('a, Ref<'a, T>);
renderer_ptr_impl!('a, RefMut<'a, T>);
renderer_ptr_impl!('a, MutexGuard<'a, T>);
renderer_ptr_impl!('a, RwLockReadGuard<'a, T>);
renderer_ptr_impl!('a, RwLockWriteGuard<'a, T>);
