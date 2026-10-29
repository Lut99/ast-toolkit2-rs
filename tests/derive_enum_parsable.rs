//  DERIVE PARSABLE.rs
//    by Lut99
//
//  Description:
//!   Tests the derivation part of `nibble`.
//

use ast_toolkit2::loc::Loc;
use ast_toolkit2::loc::test::TestLoc;
use ast_toolkit2::nibble::error::Needed;
use ast_toolkit2::nibble::{NibbleError, Parsable, Slice};


/***** AST *****/
#[derive(Debug, Eq, PartialEq, Parsable)]
pub enum Punct {
    Dot(Dot),
    Comma(Comma),
}

#[derive(Debug, Eq, PartialEq, Parsable)]
pub enum NestedSequence {
    ThreeDots(Dot, Dot, Dot),
    Comma(Comma),
}



#[derive(Debug, Eq, PartialEq, Parsable)]
#[parse(lit = "^[0-9]+")]
pub struct LitInt {
    value: u64,
    loc:   TestLoc,
}



#[derive(Debug, Eq, PartialEq, Parsable)]
#[parse(term = ",", whitespace)]
pub struct Comma(TestLoc);
impl From<Dot> for Comma {
    #[inline]
    fn from(value: Dot) -> Self { Self(value.0) }
}

#[derive(Debug, Eq, PartialEq, Parsable)]
#[parse(term = ".")]
pub struct Dot(TestLoc);
impl From<Comma> for Dot {
    #[inline]
    fn from(value: Comma) -> Self { Self(value.0) }
}





/***** TESTS *****/
#[test]
fn test_enum_parsable_punct() {
    let input1 = Slice::with_raw_id(0, b".");
    let input2 = Slice::with_raw_id(0, b",");
    let input3 = Slice::with_raw_id(0, b"a");
    let input4 = Slice::with_raw_id(0, b"");
    assert_eq!(Punct::parse(input1), Ok((Punct::Dot(Dot(TestLoc::from(Loc::encapsulate_range(0, ..1)))), input1.slice(1..))));
    assert_eq!(Punct::parse(input2), Ok((Punct::Comma(Comma(TestLoc::from(Loc::encapsulate_range(0, ..1)))), input2.slice(1..))));
    assert_eq!(Punct::parse(input3), Err(NibbleError::Unmatched(punct::Formatter(".", ","), Loc::encapsulate_range(0, ..))));
    assert_eq!(Punct::parse(input4), Err(NibbleError::NotEnough(punct::Formatter(".", ","), Needed::Bounded(1, 1), Loc::encapsulate_range(0, ()))));
}

#[test]
fn test_enum_parsable_nested_sequence() {
    let input1 = Slice::with_raw_id(0, b"...");
    let input2 = Slice::with_raw_id(0, b",");
    let input3 = Slice::with_raw_id(0, b"..");
    let input4 = Slice::with_raw_id(0, b"a");
    let input5 = Slice::with_raw_id(0, b"");
    assert_eq!(
        NestedSequence::parse(input1),
        Ok((
            NestedSequence::ThreeDots(
                Dot(TestLoc::from(Loc::encapsulate_range(0, ..1))),
                Dot(TestLoc::from(Loc::encapsulate_range(0, 1..2))),
                Dot(TestLoc::from(Loc::encapsulate_range(0, 2..3))),
            ),
            input1.slice(3..)
        ))
    );
    assert_eq!(NestedSequence::parse(input2), Ok((NestedSequence::Comma(Comma(TestLoc::from(Loc::encapsulate_range(0, ..1)))), input2.slice(1..))));
    assert_eq!(
        NestedSequence::parse(input3),
        Err(NibbleError::NotEnough(
            nested_sequence::Formatter(nested_sequence::three_dots::Formatter(".", ".", "."), ","),
            Needed::Bounded(1, 1),
            Loc::encapsulate_range(0, 2..2)
        ))
    );
    assert_eq!(
        NestedSequence::parse(input4),
        Err(NibbleError::Unmatched(
            nested_sequence::Formatter(nested_sequence::three_dots::Formatter(".", ".", "."), ","),
            Loc::encapsulate_range(0, ..)
        ))
    );
    assert_eq!(
        NestedSequence::parse(input5),
        Err(NibbleError::NotEnough(
            nested_sequence::Formatter(nested_sequence::three_dots::Formatter(".", ".", "."), ","),
            Needed::Bounded(1, 1),
            Loc::encapsulate_range(0, ())
        ))
    );
}
