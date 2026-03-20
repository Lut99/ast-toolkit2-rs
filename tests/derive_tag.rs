#![allow(unused)]
//  DERIVE TAG.rs
//    by Lut99
//
//  Description:
//!   Showcases the use of the [`Tag`](derive@Tag)-macro.
//

use ast_toolkit2::loc::test::TestLoc;
use ast_toolkit2::loc::{Loc, Located};
use ast_toolkit2::tree::{Node, Tag, Term};


#[test]
fn test_derive_tag() {
    #[derive(Debug, Default, Eq, Located, Node, PartialEq, Tag, Term)]
    #[tag(u8, b"foo")]
    struct NamedField {
        loc: TestLoc,
    }
    #[derive(Debug, Default, Eq, Located, Node, PartialEq, Tag, Term)]
    #[tag(u8, b"bar")]
    struct NamedFieldMulti {
        #[loc]
        loc1: TestLoc,
        #[loc]
        loc2: TestLoc,
    }

    #[derive(Debug, Default, Eq, Located, Node, PartialEq, Tag, Term)]
    #[tag(u8, b"baz")]
    struct UnnamedField(TestLoc);

    #[derive(Debug, Default, Eq, Located, Node, PartialEq, Tag, Term)]
    #[tag(u8, b"quz")]
    #[loc(all)]
    struct UnnamedFieldMulti(TestLoc, TestLoc);

    // NOT POSSIBLE
    // No loc exist to derive `Located` on. Neat.
    // #[derive(Debug, Default, Eq, Located, Node, PartialEq, Tag, Term)]
    // #[tag(u8, b"foo")]
    // struct NoField;


    assert_eq!(NamedField::new(), NamedField { loc: TestLoc(Loc::new()) });
    assert_eq!(NamedField::with_loc(Loc::encapsulate(0)), NamedField { loc: TestLoc(Loc::encapsulate(0)) });

    assert_eq!(NamedFieldMulti::new(), NamedFieldMulti { loc1: TestLoc(Loc::new()), loc2: TestLoc(Loc::new()) });
    assert_eq!(NamedFieldMulti::with_loc(Loc::encapsulate(0)), NamedFieldMulti {
        loc1: TestLoc(Loc::encapsulate(0)),
        loc2: TestLoc(Loc::encapsulate(0)),
    });

    assert_eq!(UnnamedField::new(), UnnamedField(TestLoc(Loc::new())));
    assert_eq!(UnnamedField::with_loc(Loc::encapsulate(0)), UnnamedField(TestLoc(Loc::encapsulate(0))));

    assert_eq!(UnnamedFieldMulti::new(), UnnamedFieldMulti(TestLoc(Loc::new()), TestLoc(Loc::new())));
    assert_eq!(UnnamedFieldMulti::with_loc(Loc::encapsulate(0)), UnnamedFieldMulti(TestLoc(Loc::encapsulate(0)), TestLoc(Loc::encapsulate(0))));
}
