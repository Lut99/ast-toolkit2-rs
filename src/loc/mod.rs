//  MOD.rs
//    by Lut99
//
//  Description:
//!   Defines the [`Loc`], a loving (but actually working) tribute to the
//!   now-deprecated `Span` from
//!   [`ast-toolkit-span`](https://github.com/Lut99/ast-toolkit-rs/tree/main/lib/ast-toolkit-span).
//!
//!   # Rationale
//!   The main reason why `Span`s weren't working is the fatal mistake of
//!   making them continuously reference the original source text. This seemed
//!   nice, initially, because of two reasons:
//!   1. They can provide source information (e.g., identifier names or string
//!      literal contents) in a copyless fashion; and
//!   2. Additional information for rendering errors (e.g., neighbouring lines)
//!      is available behind a safe link to the source (i.e., it's impossible
//!      to render a `Span` with an incompatible source).
//!
//!   However, this is a TERRIBLE idea for two reasons:
//!   1. No new Spans can be generated, because the miss the link to the
//!      original source text; and
//!   2. Because the source text is actually generic and through a reference,
//!      the ENTIRE compiler is polluted with generics and lifetimes. This
//!      makes it extremely unwieldy to work with.
//!
//!   In short, there was a lot of focus on wrong things: copying small strings
//!   is pretty cheap nowadays, and the static link only occurs at render time
//!   but is otherwise highly annoying everywhere else in the compiler!
//!
//!   This renew attempt will solve this issue by going for a more classic
//!   approach to source references: we just do it for debugging, so a [`Loc`]
//!   is just a location to a source text that can be printed but only in
//!   combination _with_ the source text it came from.
//!
//!   This time, a lot of effort is being put into making them easy to work
//!   with during the bulk of the compiler. That is, any worry about there not
//!   being any location to point to or it being a merge of different sources
//!   is left internalized to the [`Loc`].
//

// Modules
mod range;
mod spec;
pub mod test;

// Imports
use std::cmp::Ordering;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub use range::{Length, Range};
pub use spec::Located;

/// Shorthand for including all the traits of this crate.
pub mod prelude {
    pub use super::spec::*;
}





/***** LIBRARY *****/
/// Representation of a contigious slice of source text.
///
/// It is the spiritual successor of a `Span`, which carries the same information but also
/// internalizes references to the source so that the area is actually accessible. This is not the
/// case for the Loc, which instead only becomes useful when combined with the original source
/// text.
///
/// Until then, the Loc provides the following while carrying it around:
/// - You can easily create new, (meaningless) Locs for e.g. code generation;
/// - You can easily compose them, whether they are from the same source or not; and
/// - They are cheap and easy to carry around. No [`Clone`]ing, no generics, no lifetimes!
///
/// # Comparing Locs
/// Note that [`Eq`], [`Hash`] and [`PartialEq`] are all implemented for Locs but do nothing (i.e.,
/// all Locs are reported to be the same). This to make e.g. deriving them on parent structs much easier.
#[derive(Clone, Copy, Debug)]
pub struct Loc {
    /// Some unique ID (e.g., a hash) of the source text this was from.
    ///
    /// NOTE: Change at your own risk! Later parts of the code using locs might expect this to be
    /// a valid identifier and/or at least one that relates things from the same source text. You
    /// WILL run into panics if you don't replace this with a valid identifier.
    pub source: Option<u64>,
    /// The range that does the slicing.
    pub range:  Range,
}

// Constructors
impl Default for Loc {
    #[inline]
    fn default() -> Self { Self::new() }
}
impl Loc {
    /// Creates a new Loc that points to nothing.
    ///
    /// # Returns
    /// A Loc that doesn't point to source text whatsoever.
    #[inline]
    pub const fn new() -> Self { Self { source: None, range: Range::empty() } }

    /// Creates a new Loc that points to a source with the given identifier.
    ///
    /// It will initially point to the whole source. See [`Loc::encapsulate_range()`] if you
    /// want to start with a specific subset of it instead.
    ///
    /// If your ranges are instead dependent on hashes (of identifiers), see [`Loc::encapsulate()`]
    /// instead.
    ///
    /// # Arguments
    /// - `id`: Some identifier (as a [`u64`]) that is **unique for this source.** This is used to
    ///   determine which sources are mergable (i.e., Locs with the same source ID are assumed to
    ///   have the same "range" or "namespace" they span).
    ///
    /// # Returns
    /// A Loc that points to a source with the given `id`, and spans it in its entirety.
    #[inline]
    pub const fn encapsulate(id: u64) -> Self { Self { source: Some(id), range: Range::full() } }

    /// Creates a new Loc that points to a source with the given identifier, and a specific subset
    /// of it.
    ///
    /// See [`Loc::encapsulate()`] if you are fine with this Loc spanning the entire source
    /// instead.
    ///
    /// # Arguments
    /// - `id`: Some identifier (as a [`u64`]) that is **unique for this source.** This is used to
    ///   determine which sources are mergable (i.e., Locs with the same source ID are assumed to
    ///   have the same "range" or "namespace" they span).
    /// - `range`: Some [`Range`]-like that determines which subset of the source to do. Note that
    ///   this isn't yet checked; it is just remembered until later.
    ///
    /// # Returns
    /// A Loc that points to a source with the given `id` and spans a subset in the given `range`
    /// of it.
    #[inline]
    pub fn encapsulate_range(id: u64, range: impl Into<Range>) -> Self { Self { source: Some(id), range: range.into() } }
}

// Ops
impl Eq for Loc {}
impl Hash for Loc {
    /// WARNING: Note that this function does nothing, as it considers all Locs to be equivalent
    /// from an AST perspective.
    ///
    /// It exists to make deriving this trait on a parent struct easier and harmless.
    #[inline]
    fn hash<H: Hasher>(&self, _state: &mut H) {}
}
impl PartialEq for Loc {
    /// WARNING: Note that this function **always** returns true, as it considers all Locs to be
    /// equivalent from an AST perspective.
    ///
    /// It exists to make deriving this trait on a parent struct easier and harmless.
    #[inline]
    fn eq(&self, _other: &Self) -> bool { true }
    /// WARNING: Note that this function **always** returns false, as it considers all Locs to be
    /// equivalent from an AST perspective.
    ///
    /// It exists to make deriving this trait on a parent struct easier and harmless.
    #[inline]
    fn ne(&self, _other: &Self) -> bool { false }
}
impl PartialOrd for Loc {
    /// WARNING: Note that this function **always** returns [`Ordering::Equal`], as it considers
    /// all Locs to be equivalent from an AST perspective.
    ///
    /// It exists to make deriving this trait on a parent struct easier and harmless.
    #[inline]
    fn partial_cmp(&self, _other: &Self) -> Option<Ordering> { Some(Ordering::Equal) }
}
impl Ord for Loc {
    /// WARNING: Note that this function **always** returns [`Ordering::Equal`], as it considers
    /// all Locs to be equivalent from an AST perspective.
    ///
    /// It exists to make deriving this trait on a parent struct easier and harmless.
    #[inline]
    fn cmp(&self, _other: &Self) -> Ordering { Ordering::Equal }
}

// Range
impl Loc {
    /// Returns a new Loc which is a subset of this one.
    ///
    /// The subset is expressed as a [`Range`]. See [`Loc::slice_range()`] for something
    /// [`Range`]-like instead.
    ///
    /// If you want to do this operation in-place, see [`Loc::shrink()`].
    ///
    /// # Arguments
    /// - `range`: Some [`Range`] that defines a subset of self. Take note: a Loc starting at
    ///   0 starts at `self.range.pos`, _not_ 0 in the sequence. Similarly, if `range` has
    ///   [`Length::Indefinite`], then the returned loc will have the end of _this_ Loc, not
    ///   necessarily that of the sequence.
    ///
    /// # Returns
    /// A new Loc that is a subset of this one and spans the same [`source`](Loc::source).
    #[inline]
    pub const fn slice(mut self, other: Range) -> Self {
        // Defer to the in-place variant
        self.shrink(other);
        self
    }

    /// Convenience alias for [`Loc::slice()`] that accepts anything converting into a [`Range`],
    /// not just the Loc itself.
    ///
    /// See [it](Loc::slice()) for more information.
    ///
    /// # Arguments
    /// - `range`: Some [`Range`](-like) that defines a subset of self. Take note: a Loc starting
    ///   at 0 starts at `self.range.pos`, _not_ 0 in the sequence. Similarly, if `range` has
    ///   [`Length::Indefinite`], then the returned loc will have the end of _this_ Loc, not
    ///   necessarily that of the sequence.
    ///
    /// # Returns
    /// A new Loc that is a subset of this one and spans the same [`source`](Loc::source).
    #[inline]
    pub fn slice_range(self, other: impl Into<Range>) -> Self { self.slice(other.into()) }

    /// Shrinks this Loc to a specific subset of itself.
    ///
    /// The subset is expressed as a [`Range`]. See [`Loc::shrink_range()`] for something
    /// [`Range`]-like instead.
    ///
    /// If you don't need to mutate self, see [`Loc::slice()`] instead.
    ///
    /// # Arguments
    /// - `range`: Some [`Range`] that defines a subset of self. Take note: a Loc starting at
    ///   0 starts at `self.range.pos`, _not_ 0 in the sequence. Similarly, if `range` has
    ///   [`Length::Indefinite`], then the returned loc will have the end of _this_ Loc, not
    ///   necessarily that of the sequence.
    ///
    /// # Returns
    /// Self for chaining.
    #[inline]
    pub const fn shrink(&mut self, other: Range) -> &mut Self {
        self.range.shrink(other);
        self
    }

    /// Convenience alias for [`Loc::shrink()`] that accepts anything converting into a Range,
    /// not just the Loc itself.
    ///
    /// See [it](Loc::shrink()) for more information.
    ///
    /// # Arguments
    /// - `range`: Some [`Range`](-like) that defines a subset of self. Take note: a Loc starting
    ///   at 0 starts at `self.range.pos`, _not_ 0 in the sequence. Similarly, if `range` has
    ///   [`Length::Indefinite`], then the returned loc will have the end of _this_ Loc, not
    ///   necessarily that of the sequence.
    ///
    /// # Returns
    /// Self for chaining.
    #[inline]
    pub fn shrink_range(&mut self, other: impl Into<Range>) -> &mut Self { self.shrink(other.into()) }



    /// Returns a new Loc that is the union of this and the given Loc.
    ///
    /// Visually, given two ranges:
    /// ```plain
    ///      A  <============>
    ///      B                     <=======>
    /// result  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    /// ```
    ///
    /// If you want to do this in-place, see [`Loc::extend()`] instead.
    ///
    /// # Arguments
    /// - `other`: Some other Loc to join with this one.
    ///
    /// # Returns
    /// A new range representing the union of the two.
    ///
    /// Note that, if `self` and `other` have differing [`source`](Loc::source)-fields, this is a
    /// no-op! `self` is returned in that case.
    #[inline]
    pub const fn join(mut self, other: Self) -> Self {
        // Defer to the in-place variant
        self.extend(other);
        self
    }

    /// Extends this Loc to encapsulate both `self` and another given Loc.
    ///
    /// Visually, given two ranges:
    /// ```plain
    ///      A  <============>
    ///      B                     <=======>
    /// result  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    /// ```
    ///
    /// If you don't need to mutate `self`, consider [`Loc::join()`] instead.
    ///
    /// # Arguments
    /// - `other`: Some other Loc to encapsulate `self` around.
    ///
    /// # Returns
    /// Self for chaining.
    ///
    /// Note that, if `self` and `other` have differing [`source`](Loc::source)-fields, this is a
    /// no-op! `self` is returned in that case.
    #[inline]
    pub const fn extend(&mut self, other: Self) -> &mut Self {
        // NOTE: Let's use match mechanics so that this `const`ant!
        match (self.source, other.source) {
            (Some(lhs), Some(rhs)) if lhs == rhs => {
                self.range.extend(other.range);
                self
            },
            (None, None) => {
                self.range.extend(other.range);
                self
            },
            _ => self,
        }
    }



    /// Returns the starting position of this Loc.
    ///
    /// Simply equal to [`Range::pos`] in [`Loc::range`].
    ///
    /// # Returns
    /// The zero-indexed index of the leftmost element part of the range (i.e., inclusive start
    /// index).
    #[inline]
    pub const fn start(&self) -> u64 { self.range.start() }

    /// Returns the ending position of this Loc.
    ///
    /// Computes [`Range::pos`] + [`Range::len`] in [`Loc::range`], except when the second is
    /// [`Length::Indefinite`]. To get a concrete number in that case, see [`Loc::end_in()`].
    ///
    /// # Returns
    /// The zero-indexed index of the first element after the range _not_ part of it (i.e.,
    /// exclusive end index), or [`None`] if [`Range::len`] is [`Length::Indefinite`].
    #[inline]
    pub const fn end(&self) -> Option<u64> { self.range.end() }

    /// Returns the ending position of this Loc taking into account the total length of the
    /// sequence it ranges in.
    ///
    /// Computes [`Range::pos`] + [`Range::len`] in [`Loc::range`] bounded to the given length.
    /// [`Length::Indefinite`] is resolved to the given length, always. See [`Range::end()`] if you
    /// don't (want to) know `max_len`.
    ///
    /// # Arguments
    /// - `max_len`: The length of the sequence in which we contextualize this Loc.
    ///
    /// # Returns
    /// The zero-indexed index of the first element after the range _not_ part of it (i.e.,
    /// exclusive end index).
    #[inline]
    pub const fn end_in(&self, max_len: u64) -> u64 { self.range.end_in(max_len) }
}

// Uniformity
impl Located for Loc {
    #[inline(always)]
    fn loc(&self) -> Loc { *self }
}

// Conversion
impl From<Range> for Loc {
    #[inline]
    fn from(value: Range) -> Self { Self { source: None, range: value } }
}
impl From<Loc> for Range {
    #[inline]
    fn from(value: Loc) -> Self { value.range }
}
