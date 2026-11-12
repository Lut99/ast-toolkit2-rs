//  VISIT.rs
//    by Lut99
//
//  Description:
//!   Showcases the visiting pattern.
//

use std::any::Any;

use ast_toolkit2::loc::{Loc, Located};
use ast_toolkit2::tree::{Node, NodeMut};
use ast_toolkit2::visit::{ReplaceWith as _, Visit, VisitMut, Visitor, VisitorMut};


/***** AST *****/
#[derive(Debug, Eq, PartialEq)]
enum Lit {
    String(LitString),
    Int64(LitInt64),
    Int32(LitInt32),
}
impl Located for Lit {
    #[inline]
    fn loc(&self) -> Loc {
        match self {
            Self::String(s) => s.loc(),
            Self::Int64(i) => i.loc(),
            Self::Int32(i) => i.loc(),
        }
    }
}
impl Node for Lit {
    #[inline]
    fn as_any(&self) -> &dyn Any { self as &dyn Any }
}
impl NodeMut for Lit {
    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn Any { self as &mut dyn Any }
}
impl Visit for Lit {
    #[inline(always)]
    fn visit<V: Visitor>(&self, visitor: &mut V) -> V::Result {
        match self {
            Self::String(s) => visitor.visit(s),
            Self::Int64(i) => visitor.visit(i),
            Self::Int32(i) => visitor.visit(i),
        }
    }
}
impl VisitMut for Lit {
    #[inline(always)]
    fn visit_mut<V: VisitorMut>(&mut self, visitor: &mut V) -> V::Result {
        match self {
            Self::String(s) => visitor.visit_mut(s),
            Self::Int64(i) => visitor.visit_mut(i),
            Self::Int32(i) => visitor.visit_mut(i),
        }
    }
}



#[derive(Debug, Eq, PartialEq)]
struct LitString {
    value: String,
    loc:   Loc,
}
impl Located for LitString {
    #[inline(always)]
    fn loc(&self) -> Loc { self.loc }
}
impl Node for LitString {
    #[inline]
    fn as_any(&self) -> &dyn Any { self as &dyn Any }
}
impl NodeMut for LitString {
    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn Any { self as &mut dyn Any }
}
impl Visit for LitString {
    #[inline(always)]
    fn visit<V: Visitor>(&self, visitor: &mut V) -> V::Result { visitor.visit(&self.loc) }
}
impl VisitMut for LitString {
    #[inline(always)]
    fn visit_mut<V: VisitorMut>(&mut self, visitor: &mut V) -> V::Result { visitor.visit_mut(&mut self.loc) }
}



#[derive(Debug, Eq, PartialEq)]
struct LitInt64 {
    value: u64,
    loc:   Loc,
}
impl Located for LitInt64 {
    #[inline(always)]
    fn loc(&self) -> Loc { self.loc }
}
impl Node for LitInt64 {
    #[inline]
    fn as_any(&self) -> &dyn Any { self as &dyn Any }
}
impl NodeMut for LitInt64 {
    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn Any { self as &mut dyn Any }
}
impl Visit for LitInt64 {
    #[inline(always)]
    fn visit<V: Visitor>(&self, visitor: &mut V) -> V::Result { visitor.visit(&self.loc) }
}
impl VisitMut for LitInt64 {
    #[inline(always)]
    fn visit_mut<V: VisitorMut>(&mut self, visitor: &mut V) -> V::Result { visitor.visit_mut(&mut self.loc) }
}



#[derive(Debug, Eq, PartialEq)]
struct LitInt32 {
    value: u32,
    loc:   Loc,
}
impl Located for LitInt32 {
    #[inline(always)]
    fn loc(&self) -> Loc { self.loc }
}
impl Node for LitInt32 {
    #[inline]
    fn as_any(&self) -> &dyn Any { self as &dyn Any }
}
impl NodeMut for LitInt32 {
    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn Any { self as &mut dyn Any }
}
impl Visit for LitInt32 {
    #[inline(always)]
    fn visit<V: Visitor>(&self, visitor: &mut V) -> V::Result { visitor.visit(&self.loc) }
}
impl VisitMut for LitInt32 {
    #[inline(always)]
    fn visit_mut<V: VisitorMut>(&mut self, visitor: &mut V) -> V::Result { visitor.visit_mut(&mut self.loc) }
}





/***** VISITORS *****/
struct StringifyVisitor;
impl StringifyVisitor {
    fn visit_lit_mut(&mut self, node: &mut Lit) {
        // Run the replace
        node.replace_with_abort(|node| match node {
            Lit::Int64(i) => Lit::String(LitString { value: i.value.to_string(), loc: i.loc }),
            Lit::Int32(i) => Lit::String(LitString { value: i.value.to_string(), loc: i.loc }),
            // Ignored
            node @ Lit::String(_) => node,
        });

        // Recurse as usual
        match node {
            Lit::String(s) => self.visit_mut(s),
            Lit::Int64(i) => self.visit_mut(i),
            Lit::Int32(i) => self.visit_mut(i),
        }
    }
}
impl VisitorMut for StringifyVisitor {
    type Result = ();

    #[inline]
    fn visit_mut<N: VisitMut>(&mut self, node: &mut N) {
        if let Some(node) = node.as_t_mut::<Lit>() {
            self.visit_lit_mut(node);
        } else {
            node.visit_mut(self);
        }
    }
}





/***** TESTS *****/
#[test]
fn test_visit() {
    let mut ex1 = Lit::Int32(LitInt32 { value: 42, loc: Loc::new() });
    StringifyVisitor.visit_lit_mut(&mut ex1);
    assert_eq!(ex1, Lit::String(LitString { value: "42".into(), loc: Loc::new() }));
}
