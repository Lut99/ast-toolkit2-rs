#![allow(unused)]
//  DERIVE TREE.rs
//    by Lut99
//
//  Description:
//!   Showcases the use of the [`Node`](derive@Node)-,
//!   [`NonTerm`](derive@NonTerm)- and [`Term`](derive@Term)-macros.
//

use ast_toolkit2::loc::{Loc, Located};
use ast_toolkit2::tree::{Node, NodeMut};


/***** HELPER FUNCTIONS *****/
/// Asserts whether a type implements [`Node`].
fn assert_node<T: Node>() {}

/// Asserts whether a type implements [`NodeMut`].
fn assert_node_mut<T: NodeMut>() {}





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
    assert_node_mut::<StructSimple>();
    assert_node::<TupleSimple>();
    assert_node_mut::<StructSimple>();
    assert_node::<TupleGen<TupleSimple>>();
    assert_node_mut::<StructSimple>();
}

#[test]
fn test_derive_node_enums() {
    #[derive(Located, Node)]
    pub enum EnumSimple {
        Foo { foo: String, loc: Loc },
        Bar(String, #[loc] Loc),
    }

    assert_node::<EnumSimple>();
    assert_node_mut::<EnumSimple>();
}
