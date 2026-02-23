#![allow(unused)]
//  DERIVE NODE.rs
//    by Lut99
//
//  Description:
//!   Showcases the use of the [`Node`](derive@Node)-macro.
//

use ast_toolkit2::loc::{Loc, Located};
use ast_toolkit2::tree::Node;


/***** HELPER FUNCTIONS *****/
/// Asserts whether a type implements [`Node`].
fn assert_node<T: Node>() {}





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


    assert_node::<StructSimple>();
    assert_node::<TupleSimple>();
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
