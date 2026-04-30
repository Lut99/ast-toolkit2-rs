//  STEP 1: GROUPING.rs
//    by Lut99
//
//  Description:
//!   This module implements the first step of the layouting algorithm
//!   described in the [`layout`](super)-module.
//!
//!   Given a list of [`Annotation`]s, it will split them into several
//!   [`AnnotGroup`]s who are unified by the same source. Accordingly, each is
//!   represented by a new data structure that doesn't consider [`Span`]s but
//!   [`Range`]s instead.
//

use std::cmp::Ordering;
use std::collections::HashMap;

use ast_toolkit_span::{Span, Spannable};

use crate::annotations::{Annotation, Severity};


/***** DATA STRUCTURES *****/
/// Represents a bundle of [`Annot`]ations with their `S`ources externalized here.
#[derive(Clone, Debug)]
pub(super) struct AnnotGroup<'s, S: Spannable<'s>> {
    /// The main span bundling the source itself.
    pub span:   Span<S>,
    /// The bundle of annotations that are grouped in this source.
    pub annots: Vec<Annot<S::Elem>>,
}



/// Represents a simplified [`Annotation`] where its [`Span`]'s the original
/// source is externalized.
#[derive(Clone, Debug)]
pub(super) struct Annot<E> {
    /// Any replacement to do for this annotation.
    pub replacement: Option<Vec<E>>,
    /// Any message to give with this annotation.
    pub message: Option<String>,
    /// The severity of this annotation.
    pub severity: Severity,
    /// The range describing which part of the main span this annotation concerns.
    pub range: Range,
}

// Conversion
impl<'s, S: Spannable<'s>> From<Annotation<'s, S>> for Annot<S::Elem> {
    #[inline]
    fn from(value: Annotation<'s, S>) -> Self {
        let Annotation { replacement, message, severity, span } = value;

        // Resolve the span range
        let source: &S = span.source();
        let source_len: usize = source.len();
        let start: usize = span.range().start_resolved(source_len).unwrap_or(0);
        let end: usize = span.range().end_resolved(source_len).unwrap_or(0);

        // Build self
        Self { replacement, message, severity, range: Range { start, end } }
    }
}



/// Defines a concrete range over an array.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Range {
    /// The start index (inclusive).
    ///
    /// These are not publically accessible to force us to use operations to use the internal
    /// range, which allows us to ensure that certain bounds are upheld (e.g., what happens if a
    /// range is empty, that sortta thing).
    start: usize,
    /// The stop index (exclusive).
    ///
    /// These are not publically accessible to force us to use operations to use the internal
    /// range, which allows us to ensure that certain bounds are upheld (e.g., what happens if a
    /// range is empty, that sortta thing).
    end:   usize,
}

// Constructors
impl Range {
    /// Builds a new range.
    ///
    /// # Arguments
    /// - `start`: The start index (inclusive, zero-indexed) to build this range from.
    /// - `end`: The end index (exclusive, zero-indexed) to build this range from.
    ///
    /// # Returns
    /// A new Range ready to span something.
    #[inline]
    pub const fn new(start: usize, end: usize) -> Self { Self { start, end } }
}

// Range
impl Range {
    /// Slice any slice-like.
    ///
    /// We do this through this function in order to ensure that it can never panic.
    ///
    /// # Arguments
    /// - `slice`: Some [`[T]`](std::slice) to slice.
    ///
    /// # Returns
    /// Another slice which is the same or a continuous subset of `slice`.
    ///
    /// Note the slice is empty if this Range is, after bounding it to the length of the slice,
    /// is empty.
    #[inline]
    pub fn slice<'s, T>(&'_ self, slice: &'s [T]) -> &'s [T] {
        // Bound the range
        let slice_len: usize = slice.len();
        let start: usize = if self.start <= slice_len { self.start } else { slice_len };
        let end: usize = if self.end < slice_len { self.end } else { slice_len };

        // Slice
        // SAFETY: The checks above ensure that the indices are never out-of-bounds as far as
        // index-by-range is concerned. That is, `start <= end` and `end` will never be strictly
        // larger than `slice_len`.
        unsafe { slice.get_unchecked(start..end) }
    }

    /// Slice any slice-like mutably.
    ///
    /// We do this through this function in order to ensure that it can never panic.
    ///
    /// # Arguments
    /// - `slice`: Some [`[T]`](std::slice) to slice.
    ///
    /// # Returns
    /// Another slice which is the same or a continuous subset of `slice`.
    ///
    /// Note the slice is empty if this Range is, after bounding it to the length of the slice,
    /// is empty.
    #[inline]
    pub fn slice_mut<'s, T>(&'_ self, slice: &'s mut [T]) -> &'s mut [T] {
        // Bound the range
        let slice_len: usize = slice.len();
        let start: usize = if self.start <= slice_len { self.start } else { slice_len };
        let end: usize = if self.end < slice_len { self.end } else { slice_len };

        // Slice
        // SAFETY: The checks above ensure that the indices are never out-of-bounds as far as
        // index-by-range is concerned. That is, `start <= end` and `end` will never be strictly
        // larger than `slice_len`.
        unsafe { slice.get_unchecked_mut(start..end) }
    }



    /// Cuts everything from this range that overlaps with the second one, and everything following
    /// that.
    ///
    /// In essence, this creates a new Range with this range's start as start, and the other
    /// range's start as end.
    ///
    /// # Arguments
    /// - `other`: Another range to cut until its start.
    ///
    /// # Returns
    /// A new [`Range`] that has the same start, but `other`'s start as end.
    #[inline]
    pub const fn cut_up_to_overlap(&self, other: &Range) -> Self { Self { start: self.start, end: other.start } }

    /// Cuts everything from this range that overlaps with the second one, and everything before
    /// that.
    ///
    /// In essence, this creates a new Range with other's end as start, and self's end as end.
    ///
    /// # Arguments
    /// - `other`: Another range to cut until its start.
    ///
    /// # Returns
    /// A new [`Range`] that has the same start, but `other`'s start as end.
    #[inline]
    pub const fn cut_up_to_and_including_overlap(&self, other: &Range) -> Self { Self { start: other.end, end: self.end } }

    /// Applies two offsets to this Range.
    ///
    /// # Arguments
    /// - `dstart`: An offset for the start index.
    /// - `dend`: An offset for the end index.
    ///
    /// # Returns
    /// A new [`Range`] that has `self.start + dstart` as start index, and `self.end + dend` as end
    /// index.
    #[inline]
    pub const fn offset_start_and_end(&self, dstart: isize, dend: isize) -> Self {
        Self { start: (self.start as isize + dstart) as usize, end: (self.end as isize + dend) as usize }
    }

    /// Check if this range overlaps with another one.
    ///
    /// # Arguments
    /// - `other`: Some other [`Range`] to check for overlap with.
    ///
    /// # Returns
    /// True if they overlap, or false otherwise.
    #[inline]
    pub const fn overlaps_with(&self, other: &Self) -> bool {
        if self.is_empty() || other.is_empty() {
            return false;
        }
        // Based on https://stackoverflow.com/a/3269471
        self.start < other.end && other.start < self.end
    }



    /// Returns the length of this Range.
    ///
    /// # Returns
    /// A [`usize`] counting how many elements are spanned.
    #[inline]
    pub const fn len(&self) -> usize { if self.start <= self.end { self.end - self.start } else { 0 } }

    /// Checks if this Range spans anything.
    ///
    /// # Returns
    /// True if [`Range::len() == 0`](Range::len()).
    #[inline]
    pub const fn is_empty(&self) -> bool { self.len() == 0 }
}

// Ops
impl Ord for Range {
    /// Implements Ord for the Range.
    ///
    /// Note that the Range is only ordered by first index.
    ///
    /// See [`PartialOrd::partial_cmp()`] for more information.
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        // SAFETY: We can unwrap unsafely here because we know that `PartialOrd::partial_cmp()`
        // never returns [`None`].
        unsafe { self.partial_cmp(other).unwrap_unchecked() }
    }
}
impl PartialOrd for Range {
    /// Implements PartialOrd for the Range.
    ///
    /// Note that the range is purely ordered by _start_ index.
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { self.start.partial_cmp(&other.start) }
}

// Conversion
impl From<std::ops::Range<usize>> for Range {
    #[inline]
    fn from(value: std::ops::Range<usize>) -> Self { Self { start: value.start, end: value.end } }
}
impl From<Range> for ast_toolkit_span::range::Range {
    #[inline]
    fn from(value: Range) -> Self { Self::bounded(value.start, value.end) }
}





/***** LIBRARY *****/
/// Given a list of [`Annotation`]s, groups them by source and returns them as [`Annot`]ations that
/// are agnostic to the source they apply in.
///
/// This implements the first step of the layouting algorithm detailled in [`layout`](super).
///
/// # Arguments
/// - `annots`: Something yielding [`Annotation`]s that we will group.
///
/// # Returns
/// A set of [`AnnotGroup`]s that group [`Annot`]s by `S`ource.
#[inline]
pub(super) fn group_annots<'s, S>(annots: impl IntoIterator<Item = Annotation<'s, S>>) -> HashMap<S::SourceId, AnnotGroup<'s, S>>
where
    S: Spannable<'s>,
{
    let iter = annots.into_iter();
    let size_hint: (usize, Option<usize>) = iter.size_hint();
    let mut res: HashMap<S::SourceId, AnnotGroup<'s, S>> = HashMap::with_capacity(size_hint.1.unwrap_or(size_hint.0));
    for annot in iter {
        // Check if we've seen annotations of this source before
        let source_id: S::SourceId = annot.span.source_id();
        if let Some(group) = res.get_mut(&source_id) {
            group.span.extend(&annot.span);
            group.annots.push(annot.into());
        } else {
            let Annotation { replacement, message, severity, span } = annot;

            // Resolve the span range
            let source: &S = span.source();
            let source_len: usize = source.len();
            let start: usize = span.range().start_resolved(source_len).unwrap_or(0);
            let end: usize = span.range().end_resolved(source_len).unwrap_or(0);

            // Insert the annotation
            res.insert(source_id, AnnotGroup { span, annots: vec![Annot { replacement, message, severity, range: Range { start, end } }] });
        }
    }
    res
}
