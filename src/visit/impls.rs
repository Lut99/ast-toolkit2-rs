//  IMPLEMENTATIONS.rs
//    by Lut99
//
//  Description:
//!   Implements [`Visit`] and [`VisitMut`] for some [`std`] types.
//

use super::visitor::Visitor;
use super::{Visit, VisitMut};


/***** ITERATORS *****/





/***** IMPLS *****/
impl<T: 'static + Visit> Visit for Vec<T> {
    fn visit(&self, visitor: &mut super::Visitor) {
        for node in self {
            visitor.visit(node);
        }
    }
}
impl<T: 'static + VisitMut> VisitMut for Vec<T> {
    #[inline]
    fn visit_mut(&mut self, visitor: &mut Visitor) {
        for node in self {
            visitor.visit_mut(node);
        }
    }
}
