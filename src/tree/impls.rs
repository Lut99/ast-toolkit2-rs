//  IMPLEMENTATIONS.rs
//    by Lut99
//
//  Description:
//!   Implements [`Node`] and [`NodeMut`] for various [`std`] library types.
//

use std::any::Any;
use std::collections::{HashMap, HashSet, VecDeque};

use super::{Node, NodeMut};
use crate::loc::Located;


/***** HELPER MACROS *****/
macro_rules! node_impl {
    ($ty:ident<$($gens:ident),* $(,)?>) => {
        impl<$($gens: 'static + Located),*> Node for $ty<$($gens),*> {
            #[inline(always)]
            fn as_any(&self) -> &dyn Any { self as &dyn Any }
        }
    };
}

macro_rules! node_mut_impl {
    ($ty:ident<$($gens:ident),* $(,)?>) => {
        impl<$($gens: 'static + Located),*> NodeMut for $ty<$($gens),*> {
            #[inline(always)]
            fn as_any_mut(&mut self) -> &mut dyn Any { self as &mut dyn Any }
        }
    };
}





/***** IMPLS *****/
node_impl!(Option<T>);
node_mut_impl!(Option<T>);
node_impl!(Vec<T>);
node_mut_impl!(Vec<T>);
node_impl!(VecDeque<T>);
node_mut_impl!(VecDeque<T>);
node_impl!(HashSet<T>);
node_impl!(HashMap<K, V>);
