//  IMPLEMENTATIONS.rs
//    by Lut99
//
//  Description:
//!   Implements [`Visit`] and [`VisitMut`] for some [`std`] types.
//

use super::composable::{Composable, Initializable};
use super::{Visit, VisitMut, Visitor, VisitorMut};


/***** ITERATORS *****/





/***** IMPLS *****/
impl<T: 'static + Visit> Visit for Vec<T> {
    fn visit<V: Visitor>(&self, visitor: &mut V) -> V::Result {
        let mut res: V::Result = Initializable::init();
        for node in self {
            res.compose_with(visitor.visit(node));
        }
        res
    }
}
impl<T: 'static + VisitMut> VisitMut for Vec<T> {
    #[inline]
    fn visit_mut<V: VisitorMut>(&mut self, visitor: &mut V) -> V::Result {
        let mut res: V::Result = Initializable::init();
        for node in self {
            res.compose_with(visitor.visit_mut(node));
        }
        res
    }
}
