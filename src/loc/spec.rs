//  SPEC.rs
//    by Lut99
//
//  Description:
//!   Defines auxillary interfaces useful with [`Loc`]s.
//

use std::cell::{Ref, RefMut};
use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::hint::unreachable_unchecked;
use std::rc::Rc;
use std::sync::{Arc, MutexGuard, RwLockReadGuard, RwLockWriteGuard};

use super::Loc;


/***** HELPER MACROS *****/
macro_rules! located_tuple_impl {
    /* Private methods */
    (__gen() => ($($running:ident),*)) => {};
    (__gen($head:ident $(, $rem:ident)*) => ($($running:ident),*)) => {
        located_tuple_impl!(__impl($($running,)* $head));
        located_tuple_impl!(__gen($($rem),*) => ($($running,)* $head));
    };
    (__impl($fty:ident $(, $rty:ident)*)) => {
        impl<$fty: Located $(,$rty: Located)*> Located for ($fty, $($rty,)*) {
            #[inline]
            fn loc(&self) -> Loc {
                #[allow(non_snake_case)]
                let ($fty, $($rty,)*) = self;
                #[allow(unused_mut)]
                let mut res: Loc = <$fty as Located>::loc($fty);
                $(res.extend(<$rty as Located>::loc($rty));)*
                res
            }
        }
    };

    /* Public interface */
    ($($ty:ident),* $(,)?) => {
        located_tuple_impl!(__gen($($ty),*) => ());
    };
}

macro_rules! located_collection_impl {
    ($ty:ty) => {
        impl<T: Located> Located for $ty {
            /// Iterates over this type to create one [`Loc`] [`Loc::extend()`]ed over all of the
            /// elements.
            ///
            /// If there are none, then [`Loc::new()`] is returned.
            #[inline]
            fn loc(&self) -> Loc { self.into_iter().map(Located::loc).collect() }
        }
    };
}

macro_rules! located_ptr_impl {
    ('a, $ty:ty) => {
        impl<'a, T: Located> Located for $ty {
            #[inline(always)]
            fn loc(&self) -> Loc { <T as Located>::loc(self) }
        }
    };
    ($ty:ty) => {
        impl<T: Located> Located for $ty {
            #[inline(always)]
            fn loc(&self) -> Loc { <T as Located>::loc(self) }
        }
    };
}





/***** LIBRARY *****/
/// Defines an object that is tied to a location (in a source text).
pub trait Located {
    /// Returns the [`Loc`] that links this _entire_ object to the source.
    ///
    /// The idea is that this function internally combines- or shrinks [`Loc`]s until it has one
    /// "representing it as a whole". I.e., the returned [`Loc`] would suffice to refer to "this"
    /// object in the source code.
    ///
    /// # Returns
    /// A representable [`Loc`] for this object.
    fn loc(&self) -> Loc;
}

// Std impls
impl Located for Infallible {
    /// # Safety
    /// This function, while defined, is complete unreachable! This because [`Infallible`] objects
    /// are impossible to construct, and hence, no reference to `self` can exist to call this
    /// function with.
    ///
    /// **Calling this function anyway is considered undefined behaviour.**
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    fn loc(&self) -> Loc {
        #[cfg(debug_assertions)]
        panic!(
            "Whoa there, tiger! You're doing something very hacky there! It should be impossible for you to actually call this function! Without \
             `debug_assertions`, this becomes UNDEFINED BEHAVIOUR!!"
        );
        #[cfg_attr(debug_assertions, allow(unreachable_code))]
        unsafe {
            unreachable_unchecked()
        }
    }
}
impl<T: Located> Located for Option<T> {
    /// Returns [`Located::loc()`] of the internal element if it's [`Some`], else returns
    /// [`Loc::new()`].
    #[inline(always)]
    fn loc(&self) -> Loc { self.as_ref().map(Located::loc).unwrap_or_else(Loc::new) }
}
located_collection_impl!([T]);
impl<const LEN: usize, T: Located> Located for [T; LEN] {
    /// Iterates over  this type to create one [`Loc`] [`Loc::extend()`]ed over all of the
    /// elements.
    ///
    /// If there are none, then [`Loc::new()`] is returned.
    ///
    /// (Aliases to `<[T] as Located>::loc()`)
    #[inline(always)]
    fn loc(&self) -> Loc { <[T] as Located>::loc(self.as_slice()) }
}
located_collection_impl!(Vec<T>);
located_collection_impl!(HashSet<T>);
impl<K, V: Located> Located for HashMap<K, V> {
    /// Iterates over the values in this type to create one [`Loc`] [`Loc::extend()`]ed
    /// over all of the elements.
    ///
    /// If there are none, then [`Loc::new()`] is returned.
    #[inline]
    fn loc(&self) -> Loc {
        let mut res: Option<Loc> = None;
        for elem in self.values() {
            let loc = <V as Located>::loc(elem);
            res.get_or_insert(loc).extend(loc);
        }
        res.unwrap_or_else(Loc::new)
    }
}

// Abstract impls
located_tuple_impl!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);

// Pointer-like impls
located_ptr_impl!('a, &'a T);
located_ptr_impl!('a, &'a mut T);
located_ptr_impl!(Box<T>);
located_ptr_impl!(Rc<T>);
located_ptr_impl!(Arc<T>);
located_ptr_impl!('a, Ref<'a, T>);
located_ptr_impl!('a, RefMut<'a, T>);
located_ptr_impl!('a, RwLockReadGuard<'a, T>);
located_ptr_impl!('a, RwLockWriteGuard<'a, T>);
located_ptr_impl!('a, MutexGuard<'a, T>);
