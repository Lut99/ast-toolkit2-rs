#![allow(unused)]
//  DERIVE LOCATED.rs
//    by Lut99
//
//  Description:
//!   Showcases the use of the [`Located`](derive@Located)-macro.
//

use ast_toolkit2::loc::test::TestLoc;
use ast_toolkit2::loc::{Loc, Located as _};
use ast_toolkit2::macros::Located;


/***** TESTS *****/
#[test]
fn test_derive_located_structs() {
    /// Struct-style, where it defaults to a field named `loc`.
    #[derive(Located)]
    struct StructAuto {
        foo: String,
        loc: TestLoc,
    }

    /// Struct-style but we manually point it to what we want to .
    #[derive(Located)]
    struct StructManual {
        foo: String,
        #[loc]
        bar: TestLoc,
    }

    /// Struct-style but we manually point to something else than the default.
    #[derive(Located)]
    struct StructManualHard {
        loc: TestLoc,
        #[loc]
        bar: TestLoc,
    }

    /// Struct-style with multiple locs
    #[derive(Located)]
    struct StructMultiple {
        #[loc]
        loc1: TestLoc,
        #[loc]
        loc2: TestLoc,
    }

    /// Struct-style with multiple locs and then one non-used one has a confusing name.
    #[derive(Located)]
    struct StructMultipleHard {
        loc:  TestLoc,
        #[loc]
        loc1: TestLoc,
        #[loc]
        loc2: TestLoc,
    }

    /// Struct-style nested.
    #[derive(Located)]
    struct StructNested {
        loc: StructManual,
    }

    /// All-style for structs.
    #[derive(Located)]
    #[loc(all)]
    struct StructAll {
        foo: TestLoc,
        bar: TestLoc,
        baz: TestLoc,
    }



    /// Tuple-style, only field
    #[derive(Located)]
    struct TupleAuto(TestLoc);

    /// Tuple-style but we manually point to the loc field
    #[derive(Located)]
    struct TupleManual(String, #[loc] TestLoc);

    /// Multi-style where we select all appropriate fields.
    #[derive(Located)]
    struct TupleMultiple(#[loc] TestLoc, #[loc] TestLoc, TestLoc);

    /// Nested style for tuples.
    #[derive(Located)]
    struct TupleNested(TupleAuto);

    /// All-style for tuples.
    #[derive(Located)]
    #[loc(all)]
    struct TupleAll(TestLoc, TestLoc, TestLoc);



    assert_eq!(TestLoc(StructAuto { foo: "Hello, world!".into(), loc: TestLoc(Loc::encapsulate(0)) }.loc()), TestLoc(Loc::encapsulate(0)));
    assert_eq!(TestLoc(StructManual { foo: "Hello, world!".into(), bar: TestLoc(Loc::encapsulate(1)) }.loc()), TestLoc(Loc::encapsulate(1)));
    assert_eq!(
        TestLoc(StructManualHard { loc: TestLoc(Loc::encapsulate(2)), bar: TestLoc(Loc::encapsulate(3)) }.loc()),
        TestLoc(Loc::encapsulate(3))
    );
    assert_eq!(
        TestLoc(StructMultiple { loc1: TestLoc(Loc::encapsulate_range(4, 0..2)), loc2: TestLoc(Loc::encapsulate_range(4, 3..5)) }.loc()),
        TestLoc(Loc::encapsulate_range(4, 0..5))
    );
    assert_eq!(
        TestLoc(
            StructMultipleHard {
                loc:  TestLoc(Loc::encapsulate_range(5, 6..7)),
                loc1: TestLoc(Loc::encapsulate_range(5, 0..2)),
                loc2: TestLoc(Loc::encapsulate_range(5, 3..5)),
            }
            .loc()
        ),
        TestLoc(Loc::encapsulate_range(5, 0..5))
    );
    assert_eq!(
        TestLoc(StructNested { loc: StructManual { foo: "Hello, world!".into(), bar: TestLoc(Loc::encapsulate(6)) } }.loc()),
        TestLoc(Loc::encapsulate(6))
    );
    assert_eq!(
        TestLoc(
            StructAll {
                foo: TestLoc(Loc::encapsulate_range(7, ..2)),
                bar: TestLoc(Loc::encapsulate_range(7, 2..4)),
                baz: TestLoc(Loc::encapsulate_range(7, 4..6)),
            }
            .loc()
        ),
        TestLoc(Loc::encapsulate_range(7, ..6))
    );
    assert_eq!(TestLoc(TupleAuto(TestLoc(Loc::encapsulate(8))).loc()), TestLoc(Loc::encapsulate(8)));
    assert_eq!(TestLoc(TupleManual("Hello, world!".into(), TestLoc(Loc::encapsulate(9))).loc()), TestLoc(Loc::encapsulate(9)));
    assert_eq!(
        TestLoc(
            TupleMultiple(
                TestLoc(Loc::encapsulate_range(10, 0..2)),
                TestLoc(Loc::encapsulate_range(10, 2..4)),
                TestLoc(Loc::encapsulate_range(10, 4..6))
            )
            .loc()
        ),
        TestLoc(Loc::encapsulate_range(10, 0..4))
    );
    assert_eq!(TestLoc(TupleNested(TupleAuto(TestLoc(Loc::encapsulate(11)))).loc()), TestLoc(Loc::encapsulate(11)));
    assert_eq!(
        TestLoc(
            TupleAll(TestLoc(Loc::encapsulate_range(12, ..2)), TestLoc(Loc::encapsulate_range(12, 2..4)), TestLoc(Loc::encapsulate_range(12, 4..6)),)
                .loc()
        ),
        TestLoc(Loc::encapsulate_range(12, ..6))
    );
}

#[test]
fn test_derive_located_enums() {
    #[derive(Located)]
    enum EnumEmpty {}

    #[derive(Located)]
    enum EnumAuto {
        // Auto struct-style
        Foo { loc: TestLoc },
        // Auto tuple-style
        Bar(TestLoc),
    }

    #[derive(Located)]
    enum EnumManual {
        // Manual struct-style
        Foo {
            foo: String,
            #[loc]
            bar: TestLoc,
        },
        // Manual struct-style - hard
        Bar {
            loc: TestLoc,
            #[loc]
            bar: TestLoc,
        },
        // Manual tuple style
        Baz(#[loc] TestLoc, String),
    }

    #[derive(Located)]
    enum EnumMultiple {
        // Multiple struct-style
        Foo {
            #[loc]
            foo: TestLoc,
            #[loc]
            bar: TestLoc,
        },
        // Multiple struct-style, hard
        Bar {
            loc: TestLoc,
            #[loc]
            foo: TestLoc,
            #[loc]
            bar: TestLoc,
        },
        // Multiple tuple style
        Baz(#[loc] TestLoc, String, #[loc] TestLoc),
    }

    #[derive(Located)]
    enum EnumNested {
        // Struct-style
        Foo { loc: EnumAuto },
        // Tuple-style
        Bar(EnumAuto),
    }

    #[derive(Located)]
    #[loc(all)]
    enum EnumAll {
        Foo { foo: TestLoc, bar: TestLoc, baz: TestLoc },
        Bar(TestLoc, TestLoc, TestLoc),
    }

    #[derive(Located)]
    enum EnumVariantAll {
        #[loc(all)]
        Foo {
            foo: TestLoc,
            bar: TestLoc,
            baz: TestLoc,
        },
        #[loc(all)]
        Bar(TestLoc, TestLoc, TestLoc),
        Baz(TestLoc, #[loc] TestLoc),
    }



    assert_eq!(TestLoc(EnumAuto::Foo { loc: TestLoc(Loc::encapsulate(0)) }.loc()), TestLoc(Loc::encapsulate(0)));
    assert_eq!(TestLoc(EnumAuto::Bar(TestLoc(Loc::encapsulate(1))).loc()), TestLoc(Loc::encapsulate(1)));
    assert_eq!(TestLoc(EnumManual::Foo { foo: "Hello, world!".into(), bar: TestLoc(Loc::encapsulate(2)) }.loc()), TestLoc(Loc::encapsulate(2)));
    assert_eq!(TestLoc(EnumManual::Bar { loc: TestLoc(Loc::encapsulate(3)), bar: TestLoc(Loc::encapsulate(4)) }.loc()), TestLoc(Loc::encapsulate(4)));
    assert_eq!(TestLoc(EnumManual::Baz(TestLoc(Loc::encapsulate(5)), "Hello, world!".into()).loc()), TestLoc(Loc::encapsulate(5)));
    assert_eq!(
        TestLoc(EnumMultiple::Foo { foo: TestLoc(Loc::encapsulate_range(6, ..2)), bar: TestLoc(Loc::encapsulate_range(6, 4..6)) }.loc()),
        TestLoc(Loc::encapsulate_range(6, ..6))
    );
    assert_eq!(
        TestLoc(
            EnumMultiple::Bar {
                loc: TestLoc(Loc::encapsulate_range(7, ..2)),
                foo: TestLoc(Loc::encapsulate_range(7, 2..4)),
                bar: TestLoc(Loc::encapsulate_range(7, 4..6)),
            }
            .loc()
        ),
        TestLoc(Loc::encapsulate_range(7, 2..6))
    );
    assert_eq!(
        TestLoc(EnumMultiple::Baz(TestLoc(Loc::encapsulate_range(8, 0..2)), "Hello, world!".into(), TestLoc(Loc::encapsulate_range(8, 2..4)),).loc()),
        TestLoc(Loc::encapsulate_range(8, 0..4))
    );
    assert_eq!(TestLoc(EnumNested::Foo { loc: EnumAuto::Bar(TestLoc(Loc::encapsulate(9))) }.loc()), TestLoc(Loc::encapsulate(9)));
    assert_eq!(TestLoc(EnumNested::Bar(EnumAuto::Bar(TestLoc(Loc::encapsulate(10)))).loc()), TestLoc(Loc::encapsulate(10)));
    assert_eq!(
        TestLoc(
            EnumAll::Foo {
                foo: TestLoc(Loc::encapsulate_range(11, ..2)),
                bar: TestLoc(Loc::encapsulate_range(11, 2..4)),
                baz: TestLoc(Loc::encapsulate_range(11, 4..6)),
            }
            .loc()
        ),
        TestLoc(Loc::encapsulate_range(11, ..6))
    );
    assert_eq!(
        TestLoc(
            EnumAll::Bar(
                TestLoc(Loc::encapsulate_range(12, ..2)),
                TestLoc(Loc::encapsulate_range(12, 2..4)),
                TestLoc(Loc::encapsulate_range(12, 4..6)),
            )
            .loc()
        ),
        TestLoc(Loc::encapsulate_range(12, ..6))
    );
    assert_eq!(
        TestLoc(
            EnumVariantAll::Foo {
                foo: TestLoc(Loc::encapsulate_range(13, ..2)),
                bar: TestLoc(Loc::encapsulate_range(13, 2..4)),
                baz: TestLoc(Loc::encapsulate_range(13, 4..6)),
            }
            .loc()
        ),
        TestLoc(Loc::encapsulate_range(13, ..6))
    );
    assert_eq!(
        TestLoc(
            EnumVariantAll::Bar(
                TestLoc(Loc::encapsulate_range(14, ..2)),
                TestLoc(Loc::encapsulate_range(14, 2..4)),
                TestLoc(Loc::encapsulate_range(14, 4..6)),
            )
            .loc()
        ),
        TestLoc(Loc::encapsulate_range(14, ..6))
    );
    assert_eq!(
        TestLoc(EnumVariantAll::Baz(TestLoc(Loc::encapsulate_range(15, ..2)), TestLoc(Loc::encapsulate_range(15, 2..4)),).loc()),
        TestLoc(Loc::encapsulate_range(15, 2..4))
    );
}
