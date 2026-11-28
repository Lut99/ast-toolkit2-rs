//  IMPLEMENTATIONS.rs
//    by Lut99
//
//  Description:
//!   Implements [`Visit`] and [`VisitMut`] for some [`std`] types.
//

use std::collections::{HashMap, HashSet, VecDeque};

use super::composable::{Composable, Initializable};
use super::{Visit, VisitMut, Visitor, VisitorMut};


/***** HELPER MACROS *****/
macro_rules! collection_impl {
    (__impl($ty:ty)) => {
        impl<T: 'static + Visit> Visit for $ty {
            fn visit<V: Visitor>(&self, visitor: &mut V) -> V::Result {
                let mut res: V::Result = Initializable::init();
                for node in self {
                    res.compose_with(visitor.visit(node));
                }
                res
            }
        }
    };
    (__impl_mut($ty:ty)) => {
        impl<T: 'static + VisitMut> VisitMut for $ty {
            #[inline]
            fn visit_mut<V: VisitorMut>(&mut self, visitor: &mut V) -> V::Result {
                let mut res: V::Result = Initializable::init();
                for node in self {
                    res.compose_with(visitor.visit_mut(node));
                }
                res
            }
        }
    };

    (mut $ty:ty) => {
        collection_impl!(__impl($ty));
        collection_impl!(__impl_mut($ty));
    };
    ($ty:ty) => {
        collection_impl!(__impl($ty));
    };
}





/***** IMPLS *****/
impl<T: 'static + Visit> Visit for Option<T> {
    fn visit<V: Visitor>(&self, visitor: &mut V) -> V::Result {
        match self {
            Some(node) => visitor.visit(node),
            None => Initializable::init(),
        }
    }
}
impl<T: 'static + VisitMut> VisitMut for Option<T> {
    #[inline]
    fn visit_mut<V: VisitorMut>(&mut self, visitor: &mut V) -> V::Result {
        match self {
            Some(node) => visitor.visit_mut(node),
            None => Initializable::init(),
        }
    }
}

collection_impl!(mut Vec<T>);
collection_impl!(mut VecDeque<T>);
collection_impl!(HashSet<T>);

impl<K: 'static + Visit, E: 'static + Visit> Visit for HashMap<K, E> {
    fn visit<V: Visitor>(&self, visitor: &mut V) -> V::Result {
        let mut res: V::Result = Initializable::init();
        for (key, elem) in self {
            res.compose_with(visitor.visit(key));
            res.compose_with(visitor.visit(elem));
        }
        res
    }
}
