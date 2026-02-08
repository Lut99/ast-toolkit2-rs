//  RANGE.rs
//    by Lut99
//
//  Description:
//!   Implements [`Range`], an abstraction of a slice of an array.
//

use std::fmt::{Debug, Display, Formatter, Result as FResult};
use std::hash::{Hash, Hasher};
use std::ops;


/***** HELPER FUNCTIONS *****/
/// Manual re-implementation of [`min()`](std::cmp::min()) but then `const`.
///
/// # Arguments
/// - `lhs`: The first value to compare.
/// - `rhs`: The second value to compare.
///
/// # Returns
/// `lhs` if it's smaller or equal to `rhs`, else `rhs`.
#[inline]
pub const fn min(lhs: u64, rhs: u64) -> u64 { if lhs <= rhs { lhs } else { rhs } }

/// Manual re-implementation of [`max()`](std::cmp::max()) but then `const`.
///
/// # Arguments
/// - `lhs`: The first value to compare.
/// - `rhs`: The second value to compare.
///
/// # Returns
/// `lhs` if it's larger or equal to `rhs`, else `rhs`.
#[inline]
pub const fn max(lhs: u64, rhs: u64) -> u64 { if lhs >= rhs { lhs } else { rhs } }





/***** INTERFACES *****/
/// Things that are 100% guaranteed to be convertible to [`u64`].
///
/// # A notice
/// For your convenience, and, specifically, to support e.g.
/// ```rust
/// # use ast_toolkit2::loc::Range;
/// Range::from(0..10);
/// ```
/// this is also implemented for _signed_ types. Obviously, these are _not_ guaranteed to be
/// convertible to [`u64`] per sé; so we panic if they are too small.
pub trait Index {
    /// Returns it as a [`u64`].
    ///
    /// # Returns
    /// A [`u64`] representing us as index.
    fn as_u64(&self) -> u64;
}

// Core impls
impl Index for u8 {
    #[inline]
    fn as_u64(&self) -> u64 { *self as u64 }
}
impl Index for u16 {
    #[inline]
    fn as_u64(&self) -> u64 { *self as u64 }
}
impl Index for u32 {
    #[inline]
    fn as_u64(&self) -> u64 { *self as u64 }
}
impl Index for u64 {
    #[inline]
    fn as_u64(&self) -> u64 { *self as u64 }
}
#[cfg(any(target_pointer_width = "16", target_pointer_width = "32", target_pointer_width = "64"))]
impl Index for usize {
    #[inline]
    fn as_u64(&self) -> u64 { *self as u64 }
}
impl Index for i8 {
    #[inline]
    #[track_caller]
    fn as_u64(&self) -> u64 {
        if *self < 0 {
            panic!("Cannot use negative `i8` as index! (Only zero or positive integers are indices)")
        }
        *self as u64
    }
}
impl Index for i16 {
    #[inline]
    #[track_caller]
    fn as_u64(&self) -> u64 {
        if *self < 0 {
            panic!("Cannot use negative `i16` as index! (Only zero or positive integers are indices)")
        }
        *self as u64
    }
}
impl Index for i32 {
    #[inline]
    #[track_caller]
    fn as_u64(&self) -> u64 {
        if *self < 0 {
            panic!("Cannot use negative `i32` as index! (Only zero or positive integers are indices)")
        }
        *self as u64
    }
}
impl Index for i64 {
    #[inline]
    #[track_caller]
    fn as_u64(&self) -> u64 {
        if *self < 0 {
            panic!("Cannot use negative `i64` as index! (Only zero or positive integers are indices)")
        }
        *self as u64
    }
}
#[cfg(any(target_pointer_width = "16", target_pointer_width = "32", target_pointer_width = "64"))]
impl Index for isize {
    #[inline]
    #[track_caller]
    fn as_u64(&self) -> u64 {
        if *self < 0 {
            panic!("Cannot use negative `isize` as index! (Only zero or positive integers are indices)")
        }
        *self as u64
    }
}





/***** AUXILLARY *****/
/// Represents the length of a [`Range`].
///
/// This either a concrete (but excessive) number of elements, or a
/// [special value](Length::Indefinite) indicating that the range continues until the end of the
/// sequence, always.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum Length {
    /// It's a concrete number of elements.
    Fixed(u64),
    /// It's a sequence of as-of-yet indetermined length that always continues until the end of the
    /// range.
    Indefinite,
}

// Conversion
impl<T: Index> From<T> for Length {
    #[inline]
    #[track_caller]
    fn from(value: T) -> Self { Self::Fixed(value.as_u64()) }
}





/***** LIBRARY *****/
/// Implements an abstraction of a slice of a contiguous sequence of elements.
///
/// This differs from [`std::ops::Range`] in the following ways:
/// - It implements [`Copy`];
/// - It can encode special states of the range, e.g. [`Empty`](Range::Empty) or open-ended ranges,
///   in one type.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Range {
    /// The position upon which the Range starts.
    ///
    /// We assume that _all_ sequences always start at `0`.
    pub pos: u64,
    /// The length for which the range continues.
    ///
    /// This may be [`Length::Indefinite`], in which case the end of the range is placed as rightmost in
    /// the sequence as possible.
    ///
    /// Note that the Range is bounded; if `pos + len` esceeds [`u64::MAX`], then it's the same as
    /// a `len` that will perfectly add to [`u64::MAX`].
    pub len: Length,
}

// Constructors
impl Default for Range {
    /// By default, constructs the largest range possible.
    ///
    /// Alias for [`Range::full()`].
    #[inline]
    fn default() -> Self { Self::full() }
}
impl Range {
    /// Convenience constructor for the Range that will initialize it with the given position and
    /// length.
    ///
    /// # Arguments
    /// - `start`: The inclusive index of the element where to start the range.
    ///
    ///   I.e., this is the zero-indexed position of the first element in the range.
    /// - `len`: The length of the Range. Use [`None`], `()` or `..` to indicate it is unbounded.
    ///
    /// # Returns
    /// A range that starts at 0 and has length [`Length::Indefinite`].
    #[inline]
    #[track_caller]
    pub fn new(start: impl Index, len: impl Into<Length>) -> Self { Self { pos: start.as_u64(), len: len.into() } }

    /// Constructor for the Range that will initialize it to always span everything.
    ///
    /// # Returns
    /// A Range that starts at 0 and has length [`Length::Indefinite`].
    #[inline]
    pub const fn full() -> Self { Self { pos: 0, len: Length::Indefinite } }

    /// Constructor for the Range that places it on a specific index, but spans until the end of
    /// the sequence.
    ///
    /// # Arguments
    /// - `start`: The inclusive index of the element where to start the range.
    ///
    ///   I.e., this is the zero-indexed position of the first element in the range.
    ///
    /// # Returns
    /// A Range that starts at the given `index` and has length [`Length::Indefinite`].
    #[inline]
    pub const fn onwards(start: u64) -> Self { Self { pos: start, len: Length::Indefinite } }

    /// Constructor for the Range that places it at the start and stretches _up until_ (but not
    /// including) the given index.
    ///
    /// # Arguments
    /// - `end`: The exclusive index of the element where to end the range.
    ///
    ///   I.e., this is the zero-indexed position of the first element **NOT** in the range.
    ///
    /// # Returns
    /// A Range that starts at 0 and has length [`Length::Fixed`] of the given `end`.
    #[inline]
    pub const fn until(end: u64) -> Self { Self { pos: 0, len: Length::Fixed(end) } }

    /// Constructor for the Range that places it in the given bounded area.
    ///
    /// # Arguments
    /// - `start`: The inclusive index of the element where to start the range.
    ///
    ///   I.e., this is the zero-indexed position of the first element in the range.
    /// - `end`: The exclusive index of the element where to end the range.
    ///
    ///   I.e., this is the zero-indexed position of the first element **NOT** in the range.
    ///
    /// # Returns
    /// A Range that starts at `start` and has length [`Length::Fixed`] such that it ends on the
    /// element before `end`.
    #[inline]
    pub const fn bounded(start: u64, end: u64) -> Self { Self { pos: 0, len: Length::Fixed(end.saturating_sub(start)) } }

    /// Constructor for a Range that is always empty and starts on 0.
    ///
    /// # Returns
    /// A Range that start at 0 and has length [`Length::Fixed`] of 0.
    #[inline]
    pub const fn empty() -> Self { Self { pos: 0, len: Length::Fixed(0) } }

    /// Constructor for a Range that is always empty and starts on the given position.
    ///
    /// This is useful to place a Range already at a certain position, but only give it a length
    /// later.
    ///
    /// # Arguments
    /// - `start`: The inclusive index of the element where to start the range.
    ///
    ///   I.e., this is the zero-indexed position of the first element in the range (if it wasn't
    ///   empty, that is).
    ///
    /// # Returns
    /// A Range that start at `start` and has length [`Length::Fixed`] of 0.
    #[inline]
    pub const fn empty_at(start: u64) -> Self { Self { pos: start, len: Length::Fixed(0) } }
}

// Ops
impl Debug for Range {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        // We will always write the start index if relevant
        if self.pos > 0 {
            write!(f, "{}", self.pos)?;
        }

        // Special case: if the length is empty, assume empty
        if let Length::Fixed(0) = self.len {
            return write!(f, "!");
        }

        // Otherwise, write the range with blanks where it's unbounded
        write!(f, "..")?;
        if let Length::Fixed(len) = self.len {
            write!(f, "{}", self.pos + len)?;
        }
        Ok(())
    }
}
impl Display for Range {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult { <Self as Debug>::fmt(self, f) }
}
impl Eq for Range {}
impl Hash for Range {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        let Self { pos, len } = self;
        pos.hash(state);
        match len {
            // If the length is fixed, then we compute the "end" and hash that; else, we don't hash
            // anything (only some discriminant for the enum variant).
            Length::Fixed(len) => {
                0u8.hash(state);
                pos.saturating_add(*len).hash(state)
            },
            Length::Indefinite => 1u8.hash(state),
        }
    }
}
impl PartialEq for Range {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        match (self.len, other.len) {
            // NOTE: We basically compare `start` and `start` and `end` and `end`. This way, we
            // equate all Ranges that exceed [`u64::MAX`].
            (Length::Fixed(lhs), Length::Fixed(rhs)) => self.pos == other.pos && self.pos.saturating_add(lhs) == other.pos.saturating_add(rhs),
            (Length::Indefinite, Length::Indefinite) => self.pos == other.pos,
            _ => false,
        }
    }
}
impl<T: Index> PartialEq<std::ops::Range<T>> for Range {
    #[inline]
    #[track_caller]
    fn eq(&self, other: &std::ops::Range<T>) -> bool {
        match self.len {
            Length::Fixed(len) => self.pos == other.start.as_u64() && self.pos.saturating_add(len) == other.end.as_u64(),
            Length::Indefinite => false,
        }
    }
}
impl<T: Index> PartialEq<std::ops::RangeFrom<T>> for Range {
    #[inline]
    #[track_caller]
    fn eq(&self, other: &std::ops::RangeFrom<T>) -> bool {
        match self.len {
            Length::Indefinite => self.pos == other.start.as_u64(),
            Length::Fixed(_) => false,
        }
    }
}
impl<T: Index> PartialEq<std::ops::RangeTo<T>> for Range {
    #[inline]
    #[track_caller]
    fn eq(&self, other: &std::ops::RangeTo<T>) -> bool {
        match self.len {
            Length::Fixed(len) => self.pos == 0 && len == other.end.as_u64(),
            _ => false,
        }
    }
}
impl PartialEq<std::ops::RangeFull> for Range {
    #[inline]
    fn eq(&self, _other: &std::ops::RangeFull) -> bool {
        match self.len {
            Length::Indefinite => self.pos == 0,
            _ => false,
        }
    }
}
impl<T: Index> PartialEq<T> for Range {
    #[inline]
    #[track_caller]
    fn eq(&self, other: &T) -> bool {
        match self.len {
            Length::Fixed(1) => self.pos == other.as_u64(),
            _ => false,
        }
    }
}
impl PartialEq<()> for Range {
    #[inline]
    fn eq(&self, _other: &()) -> bool {
        match self.len {
            Length::Fixed(0) => true,
            _ => false,
        }
    }
}

// Range
impl Range {
    /// Returns a new Range which is a subset of this one.
    ///
    /// The subset is expressed as another Range.
    ///
    /// If you want to do this operation in-place, see [`Range::shrink()`] instead.
    ///
    /// # Arguments
    /// - `range`: Some other Range that defines a subset of self. Take note: a Range starting at
    ///   0 starts at `self.pos`, _not_ 0 in the sequence. Similarly, if it has
    ///   [`Length::Indefinite`], then the returned range will have the end of _this_ Range, not
    ///   necessarily that of the sequence.
    ///
    /// # Returns
    /// A new Range that is a subset of this one.
    #[inline]
    pub const fn slice(mut self, other: Self) -> Self {
        // Defer to the in-place variant
        self.shrink(other);
        self
    }

    /// Convenience alias for [`Range::slice()`] that accepts anything converting into a Range,
    /// not just the Range itself.
    ///
    /// See [it](Range::slice()) for more information.
    ///
    /// # Arguments
    /// - `range`: Some other Range(-like) that defines a subset of self. Take note: a Range
    ///   starting at 0 starts at `self.pos`, _not_ 0 in the sequence. Similarly, if it has
    ///   [`Length::Indefinite`], then the returned range will have the end of _this_ Range, not
    ///   necessarily that of the sequence.
    ///
    /// # Returns
    /// A new Range that is a subset of this one.
    #[inline]
    pub fn slice_range(self, other: impl Into<Self>) -> Self { self.slice(other.into()) }

    /// Shrinks this Range to a specific subset of itself.
    ///
    /// The subset is expressed as another Range.
    ///
    /// If you don't need to mutate self, see [`Range::slice()`] instead.
    ///
    /// # Arguments
    /// - `range`: Some other Range that defines a subset of self. Take note: a Range starting at
    ///   0 starts at `self.pos`, _not_ 0 in the sequence. Similarly, if it has
    ///   [`Length::Indefinite`], then the returned range will have the end of _this_ Range, not
    ///   necessarily that of the sequence.
    ///
    /// # Returns
    /// Self for chaining.
    #[inline]
    pub const fn shrink(&mut self, other: Self) -> &mut Self {
        match (&mut self.len, other.len) {
            (Length::Fixed(lhs), Length::Fixed(rhs)) => {
                // First, we start at the added position bounded by our own length
                let delta: u64 = min(other.pos, *lhs);
                self.pos = self.pos.saturating_add(delta);
                // Then we update our length to be `rhs` unless it would go out-of-bounds
                // SAFETY: The subtraction can never underflow, because `delta` is never larger
                // than `lhs`
                *lhs = min(rhs, *lhs - delta);
                self
            },
            (Length::Fixed(lhs), Length::Indefinite) => {
                let delta: u64 = min(other.pos, *lhs);
                self.pos = self.pos.saturating_add(delta);
                // SAFETY: This will never underflow, because `delta` is never larger than `lhs`.
                *lhs -= delta;
                self
            },
            (Length::Indefinite, rhs) => {
                // Straightforward; there's no rightmost bound to take into account
                self.pos = self.pos.saturating_add(other.pos);
                self.len = rhs;
                self
            },
        }
    }

    /// Convenience alias for [`Range::shrink()`] that accepts anything converting into a Range,
    /// not just the Range itself.
    ///
    /// See [it](Range::shrink()) for more information.
    ///
    /// # Arguments
    /// - `range`: Some other Range(-like) that defines a subset of self. Take note: a Range
    ///   starting at 0 starts at `self.pos`, _not_ 0 in the sequence. Similarly, if it has
    ///   [`Length::Indefinite`], then the returned range will have the end of _this_ Range, not
    ///   necessarily that of the sequence.
    ///
    /// # Returns
    /// Self for chaining.
    #[inline]
    pub fn shrink_range(&mut self, other: impl Into<Self>) -> &mut Self { self.shrink(other.into()) }



    /// Returns a new Range that is the union of this and the given Range.
    ///
    /// Visually, given two ranges:
    /// ```plain
    ///      A  <============>
    ///      B                     <=======>
    /// result  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    /// ```
    ///
    /// If you want to do this in-place, see [`Range::extend()`] instead.
    ///
    /// # Arguments
    /// - `other`: Some other Range to join with this one.
    ///
    /// # Returns
    /// A new range representing the union of the two.
    #[inline]
    pub const fn join(mut self, other: Self) -> Self {
        // Defer to the in-place variant
        self.extend(other);
        self
    }

    /// Extends this Range to encapsulate both `self` and another given Range.
    ///
    /// Visually, given two ranges:
    /// ```plain
    ///      A  <============>
    ///      B                     <=======>
    /// result  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    /// ```
    ///
    /// If you don't need to mutate `self`, consider [`Range::join()`] instead.
    ///
    /// # Arguments
    /// - `other`: Some other Range to encapsulate `self` around.
    ///
    /// # Returns
    /// Self for chaining.
    #[inline]
    pub const fn extend(&mut self, other: Self) -> &mut Self {
        match (&mut self.len, other.len) {
            (Length::Fixed(lhs), Length::Fixed(rhs)) => {
                let new_end: u64 = max(self.pos.saturating_add(*lhs), other.pos.saturating_add(rhs));
                self.pos = min(self.pos, other.pos);
                // SAFETY: Can never underflow because, at the very minimum, `new_end` is equal to
                // `self.pos` (all lengths are zero and all positions equal).
                *lhs = new_end - self.pos;
                self
            },
            (Length::Indefinite, _) | (_, Length::Indefinite) => {
                self.pos = min(self.pos, other.pos);
                self.len = Length::Indefinite;
                self
            },
        }
    }



    /// Returns the starting position of this Range.
    ///
    /// Simply equal to [`Range::pos`].
    ///
    /// # Returns
    /// The zero-indexed index of the leftmost element part of the range (i.e., inclusive start
    /// index).
    #[inline]
    pub const fn start(&self) -> u64 { self.pos }

    /// Returns the ending position of this Range.
    ///
    /// Computes [`Range::pos`] + [`Range::len`], except when the second is [`Length::Indefinite`].
    /// To get a concrete number in that case, see [`Range::end_in()`].
    ///
    /// # Returns
    /// The zero-indexed index of the first element after the range _not_ part of it (i.e.,
    /// exclusive end index), or [`None`] if [`Range::len`] is [`Length::Indefinite`].
    #[inline]
    pub const fn end(&self) -> Option<u64> {
        match self.len {
            Length::Fixed(len) => Some(self.pos.saturating_add(len)),
            Length::Indefinite => None,
        }
    }

    /// Returns the ending position of this Range taking into account the total length of the
    /// sequence it ranges in.
    ///
    /// Computes [`Range::pos`] + [`Range::len`] bounded to the given length.
    /// [`Length::Indefinite`] is resolved to the given length, always. See [`Range::end()`] if you
    /// don't (want to) know `max_len`.
    ///
    /// # Arguments
    /// - `max_len`: The length of the sequence in which we contextualize this Range.
    ///
    /// # Returns
    /// The zero-indexed index of the first element after the range _not_ part of it (i.e.,
    /// exclusive end index).
    #[inline]
    pub const fn end_in(&self, max_len: u64) -> u64 {
        match self.len {
            Length::Fixed(len) => min(self.pos.saturating_add(len), max_len),
            Length::Indefinite => max_len,
        }
    }
}

// Conversion
impl<T: Index> From<ops::Range<T>> for Range {
    #[inline]
    #[track_caller]
    fn from(value: ops::Range<T>) -> Self {
        let start: u64 = value.start.as_u64();
        let end: u64 = value.end.as_u64();
        Self { pos: start, len: Length::Fixed(end.saturating_sub(start)) }
    }
}
impl<T: Index + PartialOrd> From<ops::RangeInclusive<T>> for Range {
    #[inline]
    #[track_caller]
    fn from(value: ops::RangeInclusive<T>) -> Self {
        let start: u64 = value.start().as_u64();
        if !value.is_empty() {
            let end: u64 = value.end().as_u64();
            Self { pos: start, len: Length::Fixed(1 + end.saturating_sub(start)) }
        } else {
            Self { pos: start, len: Length::Fixed(0) }
        }
    }
}
impl<T: Index> From<ops::RangeTo<T>> for Range {
    #[inline]
    #[track_caller]
    fn from(value: ops::RangeTo<T>) -> Self { Self { pos: 0, len: Length::Fixed(value.end.as_u64()) } }
}
impl<T: Index + PartialOrd> From<ops::RangeToInclusive<T>> for Range {
    /// NOTE: We are assuming that [`ops::RangeToInclusive]` cannot represent empty ranges.
    #[inline]
    #[track_caller]
    fn from(value: ops::RangeToInclusive<T>) -> Self { Self { pos: 0, len: Length::Fixed(1 + value.end.as_u64()) } }
}
impl<T: Index> From<ops::RangeFrom<T>> for Range {
    #[inline]
    #[track_caller]
    fn from(value: ops::RangeFrom<T>) -> Self { Self { pos: value.start.as_u64(), len: Length::Indefinite } }
}
impl From<ops::RangeFull> for Range {
    #[inline]
    fn from(_value: ops::RangeFull) -> Self { Self { pos: 0, len: Length::Indefinite } }
}
impl From<()> for Range {
    #[inline]
    fn from(_value: ()) -> Self { Self { pos: 0, len: Length::Fixed(0) } }
}





/***** TESTS *****/
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slice() {
        // Some testcases; extend when more are known!
        assert_eq!(Range::from(0..10).slice_range(0..5), 0..5);
        assert_eq!(Range::from(..10).slice_range(0..5), 0..5);
        assert_eq!(Range::from(..10).slice_range(..5), ..5);
        assert_eq!(Range::from(5..10).slice_range(..5), 5..10);
        assert_eq!(Range::from(10..5), ());
        assert_eq!(Range::from(5..).slice_range(..5), 5..10);
        assert_eq!(Range::from(5..).slice_range(..10), 5..15);
        assert_eq!(Range::from(..).slice_range(5..10), 5..10);
        assert_eq!(Range::from(()).slice_range(1..10), ());

        // Remember, we're slicing _in_ the left slice
        assert_eq!(Range::from(1..).slice_range(1..), 2..);
        assert_eq!(Range::from(1..).slice_range(1..3), 2..4);
        assert_eq!(Range::from(1..).slice_range(1..1), ());
        assert_eq!(Range::from(1..3).slice_range(1..), 2..3);
        assert_eq!(Range::from(1..4).slice_range(1..2), 2..3);
        assert_eq!(Range::from(1..4).slice_range(1..7), 2..4);
        assert_eq!(Range::from(1..8).slice_range(1..2), 2..3);
        assert_eq!(Range::from(1..4).slice_range(..2), 1..3);

        assert_eq!(Range::from(2..).slice_range(..1), 2..3);
    }
}
