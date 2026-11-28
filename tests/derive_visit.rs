//  VISIT.rs
//    by Lut99
//
//  Description:
//!   Showcases the visiting pattern.
//

use ast_toolkit2::loc::{Loc, Located};
use ast_toolkit2::tree::Node;
use ast_toolkit2::visit::{Group, ReplaceWith as _, Visit, VisitMut, Visitor, VisitorMut, group, visitor_impl, visitor_mut_impl};


/***** AST *****/
#[derive(Debug, Eq, Located, Node, PartialEq, Visit, VisitMut)]
enum Lit {
    String(LitString),
    Int64(LitInt64),
    Int32(LitInt32),
}



#[derive(Debug, Eq, Located, Node, PartialEq, Visit, VisitMut)]
struct LitString {
    #[visit(skip)]
    value: String,
    loc:   Loc,
}



#[derive(Debug, Eq, Located, Node, PartialEq, Visit, VisitMut)]
struct LitInt64 {
    #[visit(skip)]
    value: u64,
    loc:   Loc,
}



#[derive(Debug, Eq, Located, Node, PartialEq, Visit, VisitMut)]
struct LitInt32 {
    #[visit(skip)]
    value: u32,
    loc:   Loc,
}





/***** GROUPS *****/
group! {
    /// Identifies all nodes carrying a numeric value.
    pub group LitNum<'a> {
        Int32(&'a LitInt32),
        Int64(&'a LitInt64),
    }
}
impl<'a> LitNum<'a> {
    fn value(&self) -> u64 {
        match self {
            Self::Int32(l) => l.value as u64,
            Self::Int64(l) => l.value,
        }
    }
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
visitor_mut_impl!(StringifyVisitor { Lit });



struct ValueVisitor;
impl ValueVisitor {
    fn visit_lit_int32(&mut self, node: &LitInt32) -> u64 { node.value as u64 }
    fn visit_lit_int64(&mut self, node: &LitInt64) -> u64 { node.value }
}
visitor_impl!(ValueVisitor as u64 {
    LitInt32 => visit_lit_int32,
    LitInt64,
});



struct GroupVisitor;
impl GroupVisitor {
    fn visit_lit_num(&mut self, node: LitNum<'_>) -> u64 { node.value() }
}
visitor_impl!(GroupVisitor as u64 {
    {group} LitNum,
});





/***** TESTS *****/
#[test]
fn test_derive_visit() {
    let mut ex1 = Lit::Int32(LitInt32 { value: 42, loc: Loc::new() });
    StringifyVisitor.visit_mut(&mut ex1);
    assert_eq!(ex1, Lit::String(LitString { value: "42".into(), loc: Loc::new() }));

    let ex2 = Lit::Int64(LitInt64 { value: 42, loc: Loc::new() });
    assert_eq!(ValueVisitor.visit(&ex2), 42);

    let ex3 = Lit::Int32(LitInt32 { value: 42, loc: Loc::new() });
    assert_eq!(GroupVisitor.visit(&ex3), 42);
}



#[test]
fn test_derive_visit_extra() {
    #[allow(dead_code)]
    #[derive(Node, Visit, VisitMut)]
    struct Empty;
    impl Located for Empty {
        #[inline]
        fn loc(&self) -> Loc { Loc::new() }
    }
}
