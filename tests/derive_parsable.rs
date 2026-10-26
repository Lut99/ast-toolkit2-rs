//  DERIVE PARSABLE.rs
//    by Lut99
//
//  Description:
//!   Tests the derivation part of `nibble`.
//

use ast_toolkit2::loc::Loc;
use ast_toolkit2::loc::test::TestLoc;
use ast_toolkit2::nibble::convert::ParseIntError;
use ast_toolkit2::nibble::error::Needed;
use ast_toolkit2::nibble::{NibbleError, Parsable, Slice};


/***** AST *****/
#[derive(Debug, Eq, PartialEq, Parsable)]
#[parse(lit = "^[0-9]+")]
pub struct LitInt {
    value: u64,
    loc:   TestLoc,
}

#[derive(Debug, Eq, PartialEq, Parsable)]
#[parse(term = "fn", sep = "[A-Za-z0-9_]")]
pub struct Fn(TestLoc);

#[derive(Debug, Eq, PartialEq, Parsable)]
#[parse(term = ".")]
pub struct Dot(TestLoc);





/***** TESTS *****/
#[test]
fn test_parsable_lit() {
    let input1 = Slice::with_raw_id(0, b"12");
    let input2 = Slice::with_raw_id(0, b"0012");
    let input3 = Slice::with_raw_id(0, b"999999999");
    let input4 = Slice::with_raw_id(0, b"1a2");
    let input5 = Slice::with_raw_id(0, b"99999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999");
    let input6 = Slice::with_raw_id(0, b"a12");
    let input7 = Slice::with_raw_id(0, b"");
    assert_eq!(LitInt::parse(input1), Ok((LitInt { value: 12, loc: TestLoc::from(Loc::encapsulate_range(0, 0..2)) }, input1.slice(2..))));
    assert_eq!(LitInt::parse(input2), Ok((LitInt { value: 12, loc: TestLoc::from(Loc::encapsulate_range(0, 0..4)) }, input2.slice(4..))));
    assert_eq!(LitInt::parse(input3), Ok((LitInt { value: 999999999, loc: TestLoc::from(Loc::encapsulate_range(0, 0..9)) }, input3.slice(9..))));
    assert_eq!(LitInt::parse(input4), Ok((LitInt { value: 1, loc: TestLoc::from(Loc::encapsulate_range(0, 0..1)) }, input4.slice(1..))));
    assert_eq!(
        LitInt::parse(input5),
        Err(NibbleError::Error(ParseIntError::Overflow(
            64,
            b"99999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999".into()
        )))
    );
    assert_eq!(LitInt::parse(input6), Err(NibbleError::Unmatched("a literal matching the regex \"^[0-9]+\"", Loc::encapsulate_range(0, ..))));
    assert_eq!(LitInt::parse(input7), Err(NibbleError::Unmatched("a literal matching the regex \"^[0-9]+\"", Loc::encapsulate_range(0, ..))));
}

#[test]
fn test_parsable_kw_term() {
    let input1 = Slice::with_raw_id(0, b"fn");
    let input2 = Slice::with_raw_id(0, b"fn fn");
    let input3 = Slice::with_raw_id(0, b"fnfn");
    let input4 = Slice::with_raw_id(0, b"fa");
    let input5 = Slice::with_raw_id(0, b".");
    let input6 = Slice::with_raw_id(0, b"f");
    let input7 = Slice::with_raw_id(0, b"");
    assert_eq!(Fn::parse(input1), Ok((Fn(TestLoc::from(Loc::encapsulate_range(0, 0..2))), input1.slice(2..))));
    assert_eq!(Fn::parse(input2), Ok((Fn(TestLoc::from(Loc::encapsulate_range(0, 0..2))), input2.slice(2..))));
    assert_eq!(Fn::parse(input3), Err(NibbleError::Unmatched("fn", Loc::encapsulate_range(0, ..2))));
    assert_eq!(Fn::parse(input4), Err(NibbleError::Unmatched("fn", Loc::encapsulate_range(0, ..2))));
    assert_eq!(Fn::parse(input5), Err(NibbleError::Unmatched("fn", Loc::encapsulate_range(0, ..1))));
    assert_eq!(Fn::parse(input6), Err(NibbleError::NotEnough("fn", Needed::Bounded(1, 1), Loc::encapsulate_range(0, ..1))));
    assert_eq!(Fn::parse(input7), Err(NibbleError::NotEnough("fn", Needed::Bounded(2, 2), Loc::encapsulate_range(0, ()))));
}

#[test]
fn test_parsable_punct_term() {
    let input1 = Slice::with_raw_id(0, b".");
    let input2 = Slice::with_raw_id(0, b". .");
    let input3 = Slice::with_raw_id(0, b"..");
    let input4 = Slice::with_raw_id(0, b",");
    let input5 = Slice::with_raw_id(0, b"");
    assert_eq!(Dot::parse(input1), Ok((Dot(TestLoc::from(Loc::encapsulate_range(0, 0..1))), input1.slice(1..))));
    assert_eq!(Dot::parse(input2), Ok((Dot(TestLoc::from(Loc::encapsulate_range(0, 0..1))), input2.slice(1..))));
    assert_eq!(Dot::parse(input3), Ok((Dot(TestLoc::from(Loc::encapsulate_range(0, 0..1))), input3.slice(1..))));
    assert_eq!(Dot::parse(input4), Err(NibbleError::Unmatched(".", Loc::encapsulate_range(0, ..1))));
    assert_eq!(Dot::parse(input5), Err(NibbleError::NotEnough(".", Needed::Bounded(1, 1), Loc::encapsulate_range(0, ()))));
}
