//  SPAN.rs
//    by Lut99
//
//  Description:
//!   Defines things that relate to the source text and how.
//

use std::fmt::{Debug, Formatter, Result as FResult};


/***** INTERFACES *****/
/// Defines something _even more_ general than a [`Node`](super::node::Node): anything that
/// relates to the source text through [`Span`]s.
pub trait Spanning {
    /// Returns a [`Span`] that relates this node to the source text.
    ///
    /// You should assume that the returned span will link to the _whole_ source text backing this
    /// node. Depending on where you are in your tree, this may grow very large and complex,
    /// potentially even including snippets from _different_ texts.
    ///
    /// As such, when e.g. throwing errors, it tends to be worthwhile to be as specific as you can
    /// when providing spans.
    ///
    /// # Returns
    /// A [`Span`] that points to this node in the source text(s).
    fn span(&self) -> Span;
}





/***** HELPERS *****/
/// Defines a unique identifier for all source texts.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
struct SourceId(usize);

// Ops
impl Debug for SourceId {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult { self.0.fmt(f) }
}



/// Defines a contiguous range in a source text.
///
/// Note: disjoint from [`std::ops::Range`] because that doesn't implement [`Copy`] :/
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum Range {}

// Ops
impl Debug for Range {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult { todo!() }
}



/// Defines the (non-public!) inner workings of a [`Span`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct SpanInner {
    /// Defines the ID to which this source belongs.
    source_id: SourceId,
    /// Defines the range that does the actual spanning.
    range:     Range,
}





/***** LIBRARY *****/
/// Defines an area of (a) source text(s).
///
/// Generally, through [`Spanning`], this is used to communicate back to the user where certain
/// errors occur. Or, in language servers, it is used to transform understood source text.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Span<const NSOURCES: usize = 16>(SpanInner);
