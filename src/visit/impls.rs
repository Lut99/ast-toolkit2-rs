//  IMPLEMENTATIONS.rs
//    by Lut99
//
//  Description:
//!   Implements [`Visit`] and [`VisitMut`] for some [`std`] types.
//

use super::{Visit, VisitMut, Visitor, VisitorMut};


/***** ITERATORS *****/





/***** IMPLS *****/
impl<T: 'static + Visit> Visit for Vec<T> {
    fn visit<V: Visitor>(&self, visitor: &mut V) {
        for node in self {
            visitor.visit(node);
        }
    }
}
impl<T: 'static + VisitMut> VisitMut for Vec<T> {
    #[inline]
    fn visit_mut<V: VisitorMut>(&mut self, visitor: &mut V) {
        for node in self {
            visitor.visit_mut(node);
        }
    }
}
