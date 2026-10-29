//  DERIVE PARSABLE.rs
//    by Lut99
//
//  Description:
//!   Tests the derivation part of `nibble`.
//

use std::marker::PhantomData;

use ast_toolkit2::loc::Loc;
use ast_toolkit2::loc::test::TestLoc;
use ast_toolkit2::nibble::convert::ParseIntError;
use ast_toolkit2::nibble::error::Needed;
use ast_toolkit2::nibble::{NibbleError, Parsable, Slice};


/***** AST *****/
#[derive(Debug, Eq, PartialEq, Parsable)]
pub struct LitIntDot {
    lit: LitInt,
    dot: Dot,
}

#[derive(Debug, Eq, PartialEq, Parsable)]
pub struct Tuple(Comma, Dot);

#[derive(Debug, Eq, PartialEq, Parsable)]
#[parse(expected = "a silly example")]
pub struct FnDot {
    fn_token: Fn,
    dot:      Dot,
}

#[derive(Debug, Eq, PartialEq, Parsable)]
pub struct As {
    #[parse(as = Comma)]
    dot: Dot,
}

#[derive(Debug, Eq, PartialEq, Parsable)]
pub struct Skip {
    dot:   Dot,
    #[parse(skip)]
    extra: String,
}

#[derive(Debug, Eq, PartialEq, Parsable)]
pub struct SkipLong {
    dot1:  Dot,
    dot2:  Dot,
    #[parse(skip)]
    extra: String,
}

#[derive(Debug, Eq, PartialEq, Parsable)]
#[parse(as = Dot)]
pub struct Empty;
impl From<Dot> for Empty {
    #[inline]
    fn from(_value: Dot) -> Self { Self }
}



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
fn test_struct_parsable_seq() {
    let input1 = Slice::with_raw_id(0, b"12.");
    let input2 = Slice::with_raw_id(0, b"9999999.");
    let input3 = Slice::with_raw_id(0, b"12");
    let input4 = Slice::with_raw_id(0, b".");
    let input5 = Slice::with_raw_id(0, b"99999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999.");
    let input6 = Slice::with_raw_id(0, b"");
    assert_eq!(
        LitIntDot::parse(input1),
        Ok((
            LitIntDot {
                lit: LitInt { value: 12, loc: TestLoc::from(Loc::encapsulate_range(0, 0..2)) },
                dot: Dot(TestLoc::from(Loc::encapsulate_range(0, 2..3))),
            },
            input1.slice(3..)
        ))
    );
    assert_eq!(
        LitIntDot::parse(input2),
        Ok((
            LitIntDot {
                lit: LitInt { value: 9999999, loc: TestLoc::from(Loc::encapsulate_range(0, 0..7)) },
                dot: Dot(TestLoc::from(Loc::encapsulate_range(0, 7..8))),
            },
            input2.slice(8..)
        ))
    );
    assert_eq!(
        LitIntDot::parse(input3),
        Err(NibbleError::NotEnough(
            lit_int_dot::Formatter("a literal matching the regex \"^[0-9]+\"", "."),
            Needed::Bounded(1, 1),
            Loc::encapsulate_range(0, 2..2)
        ))
    );
    assert_eq!(
        LitIntDot::parse(input4),
        Err(NibbleError::Unmatched(lit_int_dot::Formatter("a literal matching the regex \"^[0-9]+\"", "."), Loc::encapsulate_range(0, ..)))
    );
    assert_eq!(
        LitIntDot::parse(input5),
        Err(NibbleError::Error(lit_int_dot::Error::Lit(ParseIntError::Overflow(
            64,
            b"99999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999".into()
        ))))
    );
    assert_eq!(
        LitIntDot::parse(input6),
        Err(NibbleError::Unmatched(lit_int_dot::Formatter("a literal matching the regex \"^[0-9]+\"", "."), Loc::encapsulate_range(0, ..)))
    );
}

#[test]
fn test_struct_parsable_seq_tuple() {
    let input1 = Slice::with_raw_id(0, b",.");
    let input2 = Slice::with_raw_id(0, b",");
    let input3 = Slice::with_raw_id(0, b".");
    let input4 = Slice::with_raw_id(0, b"");
    assert_eq!(
        Tuple::parse(input1),
        Ok((Tuple(Comma(TestLoc::from(Loc::encapsulate_range(0, 0..1))), Dot(TestLoc::from(Loc::encapsulate_range(0, 1..2)))), input1.slice(2..)))
    );
    assert_eq!(Tuple::parse(input2), Err(NibbleError::NotEnough(tuple::Formatter(",", "."), Needed::Bounded(1, 1), Loc::encapsulate_range(0, 1..1))));
    assert_eq!(Tuple::parse(input3), Err(NibbleError::Unmatched(tuple::Formatter(",", "."), Loc::encapsulate_range(0, ..1))));
    assert_eq!(Tuple::parse(input4), Err(NibbleError::NotEnough(tuple::Formatter(",", "."), Needed::Bounded(1, 1), Loc::encapsulate_range(0, ()))));
}

#[test]
fn test_struct_parsable_seq_expects() {
    let input1 = Slice::with_raw_id(0, b"fn.");
    let input2 = Slice::with_raw_id(0, b"fn");
    let input3 = Slice::with_raw_id(0, b".");
    let input4 = Slice::with_raw_id(0, b"");
    assert_eq!(
        FnDot::parse(input1),
        Ok((
            FnDot { fn_token: Fn(TestLoc::from(Loc::encapsulate_range(0, 0..2))), dot: Dot(TestLoc::from(Loc::encapsulate_range(0, 2..3))) },
            input1.slice(3..)
        ))
    );
    assert_eq!(FnDot::parse(input2), Err(NibbleError::NotEnough("a silly example", Needed::Bounded(1, 1), Loc::encapsulate_range(0, 2..2))));
    assert_eq!(FnDot::parse(input3), Err(NibbleError::Unmatched("a silly example", Loc::encapsulate_range(0, ..1))));
    assert_eq!(FnDot::parse(input4), Err(NibbleError::NotEnough("a silly example", Needed::Bounded(2, 2), Loc::encapsulate_range(0, 0..0))));
}

#[test]
fn test_struct_parsable_seq_as() {
    let input1 = Slice::with_raw_id(0, b",");
    let input2 = Slice::with_raw_id(0, b".");
    let input3 = Slice::with_raw_id(0, b"fn");
    let input4 = Slice::with_raw_id(0, b"");
    assert_eq!(As::parse(input1), Ok((As { dot: Dot(TestLoc::from(Loc::encapsulate_range(0, ..1))) }, input1.slice(1..))));
    assert_eq!(As::parse(input2), Err(NibbleError::Unmatched(",", Loc::encapsulate_range(0, ..1))));
    assert_eq!(As::parse(input3), Err(NibbleError::Unmatched(",", Loc::encapsulate_range(0, ..1))));
    assert_eq!(As::parse(input4), Err(NibbleError::NotEnough(",", Needed::Bounded(1, 1), Loc::encapsulate_range(0, 0..0))));
}

#[test]
fn test_struct_parsable_seq_skip() {
    let input1 = Slice::with_raw_id(0, b".");
    let input2 = Slice::with_raw_id(0, b"fn");
    let input3 = Slice::with_raw_id(0, b"");
    assert_eq!(
        Skip::parse(input1),
        Ok((Skip { dot: Dot(TestLoc::from(Loc::encapsulate_range(0, ..1))), extra: Default::default() }, input1.slice(1..)))
    );
    assert_eq!(Skip::parse(input2), Err(NibbleError::Unmatched(".", Loc::encapsulate_range(0, ..1))));
    assert_eq!(Skip::parse(input3), Err(NibbleError::NotEnough(".", Needed::Bounded(1, 1), Loc::encapsulate_range(0, 0..0))));

    let input1 = Slice::with_raw_id(0, b"..");
    let input2 = Slice::with_raw_id(0, b".");
    let input3 = Slice::with_raw_id(0, b"fn");
    let input4 = Slice::with_raw_id(0, b"");
    assert_eq!(
        SkipLong::parse(input1),
        Ok((
            SkipLong {
                dot1:  Dot(TestLoc::from(Loc::encapsulate_range(0, ..1))),
                dot2:  Dot(TestLoc::from(Loc::encapsulate_range(0, 1..2))),
                extra: Default::default(),
            },
            input1.slice(2..)
        ))
    );
    assert_eq!(
        SkipLong::parse(input2),
        Err(NibbleError::NotEnough(skip_long::Formatter(".", ".", PhantomData), Needed::Bounded(1, 1), Loc::encapsulate_range(0, 1..1)))
    );
    assert_eq!(SkipLong::parse(input3), Err(NibbleError::Unmatched(skip_long::Formatter(".", ".", PhantomData), Loc::encapsulate_range(0, ..1))));
    assert_eq!(
        SkipLong::parse(input4),
        Err(NibbleError::NotEnough(skip_long::Formatter(".", ".", PhantomData), Needed::Bounded(1, 1), Loc::encapsulate_range(0, 0..0)))
    );
}

#[test]
fn test_struct_parsable_empty() {
    let input1 = Slice::with_raw_id(0, b".");
    let input2 = Slice::with_raw_id(0, b"fn");
    let input3 = Slice::with_raw_id(0, b"");
    assert_eq!(Empty::parse(input1), Ok((Empty, input1.slice(1..))));
    assert_eq!(Empty::parse(input2), Err(NibbleError::Unmatched(".", Loc::encapsulate_range(0, ..1))));
    assert_eq!(Empty::parse(input3), Err(NibbleError::NotEnough(".", Needed::Bounded(1, 1), Loc::encapsulate_range(0, 0..0))));
}

#[test]
fn test_struct_parsable_lit() {
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
fn test_struct_parsable_term_whitespace() {
    let input1 = Slice::with_raw_id(0, b", ,");
    let input2 = Slice::with_raw_id(0, b"    , ,");
    let input3 = Slice::with_raw_id(0, b"    ,     ,");
    assert_eq!(Comma::parse(input1), Ok((Comma(TestLoc::from(Loc::encapsulate_range(0, 0..1))), input1.slice(1..))));
    assert_eq!(Comma::parse(input2), Ok((Comma(TestLoc::from(Loc::encapsulate_range(0, 4..5))), input2.slice(5..))));
    assert_eq!(Comma::parse(input3), Ok((Comma(TestLoc::from(Loc::encapsulate_range(0, 4..5))), input3.slice(5..))));
}

#[test]
fn test_struct_parsable_kw_term() {
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
fn test_struct_parsable_punct_term() {
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
