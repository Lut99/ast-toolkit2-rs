#![allow(unused)]
//  DERIVE TREE.rs
//    by Lut99
//
//  Description:
//!   Showcases the use of the [`Node`](derive@Node)-,
//!   [`NonTerm`](derive@NonTerm)- and [`Term`](derive@Term)-macros.
//

use ast_toolkit2::loc::{Loc, Located};
use ast_toolkit2::tree::{Node, NonTerm, Term};


/***** HELPER FUNCTIONS *****/
/// Asserts whether a type implements [`Node`].
fn assert_node<T: Node>() {}

/// Asserts whether a type implements [`NonTerm`].
fn assert_nonterm<T: NonTerm>() {}

/// Asserts whether a type implements [`Term`].
fn assert_term<T: Term>() {}





/***** TESTS *****/
#[test]
fn test_derive_node_structs() {
    #[derive(Located, Node)]
    pub struct StructSimple {
        foo: String,
        loc: Loc,
    }

    #[derive(Located, Node)]
    pub struct TupleSimple(String, #[loc] Loc);

    #[derive(Located, Node)]
    pub struct TupleGen<T>(T);


    assert_node::<StructSimple>();
    assert_node::<TupleSimple>();
    assert_node::<TupleGen<TupleSimple>>();
}

#[test]
fn test_derive_node_enums() {
    #[derive(Located, Node)]
    pub enum EnumSimple {
        Foo { foo: String, loc: Loc },
        Bar(String, #[loc] Loc),
    }

    assert_node::<EnumSimple>();
}



#[test]
fn test_derive_nonterm_structs() {
    #[derive(Located, Node, NonTerm)]
    pub struct StructSimple {
        foo: String,
        loc: Loc,
    }

    #[derive(Located, Node, NonTerm)]
    pub struct TupleSimple(String, #[loc] Loc);

    #[derive(Located, Node, NonTerm)]
    pub struct TupleGen<T>(T);


    assert_nonterm::<StructSimple>();
    assert_nonterm::<TupleSimple>();
    assert_nonterm::<TupleGen<TupleSimple>>();
}

#[test]
fn test_derive_nonterm_enums() {
    #[derive(Located, Node, NonTerm)]
    pub enum EnumSimple {
        Foo { foo: String, loc: Loc },
        Bar(String, #[loc] Loc),
    }

    assert_nonterm::<EnumSimple>();
}



#[test]
fn test_derive_term_structs() {
    #[derive(Located, Node, Term)]
    pub struct StructSimple {
        foo: String,
        loc: Loc,
    }

    #[derive(Located, Node, Term)]
    pub struct TupleSimple(String, #[loc] Loc);

    #[derive(Located, Node, Term)]
    pub struct TupleGen<T>(T);


    assert_term::<StructSimple>();
    assert_term::<TupleSimple>();
    assert_term::<TupleGen<TupleSimple>>();
}

#[test]
fn test_derive_term_enums() {
    #[derive(Located, Node, Term)]
    pub enum EnumSimple {
        Foo { foo: String, loc: Loc },
        Bar(String, #[loc] Loc),
    }

    assert_term::<EnumSimple>();
}
