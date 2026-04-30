//  STEP 3: APPLYING REPLACEMENTS.rs
//    by Lut99
//
//  Description:
//!   This module implements the third step of the layouting algorithm
//!   described in the [`layout`](super)-module.
//!
//!   Given an [`AnnotGroup`], it will wraps its [`Span`] in a [`VirtualSpan`]
//!   and interleave slices of the first with replacements embedded in the
//!   [`AnnotGroup`]. This allows the rest of the pipeline to abstract over
//!   suggestions.
//

use ast_toolkit_span::{Span, Spannable};

use super::s1_grouping::{AnnotGroup, Range};
use crate::annotations::Severity;
use crate::layout::s1_grouping::Annot;


/***** DATA STRUCTURES *****/
/// Abstracts over a [`Span`] with sliced replaced values.
pub struct VirtualSpan<'s, S: Spannable<'s>> {
    /// The source which the `slices` are slicing.
    ///
    /// This is a [`Span`] instead of a `source` to cheaply obtain a range over the whole span.
    span:   Span<S>,
    /// A list of the slices that make up this span.
    ///
    /// This structure MUST uphold that the [`Range`]s embedded in the
    /// [slices](VirtualSlice::Source) are within range. We're talking about consequences in the
    /// undefined territory if you fail.
    slices: Vec<VirtualSlice<S::Elem>>,
    /// The annotations that we haven't processed yet.
    annots: Vec<VirtualAnnot>,
}

// Virtual span
impl<'s, S: Spannable<'s>> VirtualSpan<'s, S> {
    /// Returns an iterator over elements in the VirtualSpan.
    ///
    /// # Returns
    /// An [`Iterator`] yielding [`S::Elem`]ents. In fact, it even implements
    /// [`DoubleEndedIterator`].
    #[inline]
    pub fn elems(&self) -> impl DoubleEndedIterator<Item = &S::Elem> { self.slices().flatten() }

    /// Returns an iterator over slices in the VirtualSpan.
    ///
    /// This will yield the closest thing a VirtualSpan has to a slice. It is the continious bits
    /// of memory available. They either represent a slice of the original source, or a replaced
    /// part.
    ///
    /// # Returns
    /// An [`Iterator`] yielding slices of [`S::Elem`]ents. In fact, it even implements
    /// [`ExactSizeIterator`] and [`DoubleEndedIterator`].
    #[inline]
    pub fn slices(&self) -> impl ExactSizeIterator<Item = &[S::Elem]> + DoubleEndedIterator<Item = &[S::Elem]> {
        self.slices.iter().map(|vs| match vs {
            VirtualSlice::Source(r) => r.slice(self.span.source().as_slice()),
            VirtualSlice::Replace(r) => r.as_slice(),
        })
    }
}



/// Abstracts over either an original `S`ource slice, or a replacement.
pub(super) enum VirtualSlice<E> {
    /// It's original source slice.
    Source(Range),
    /// It's a replacement array.
    Replace(Vec<E>),
}



/// Defines a processed version of an [`Annot`] that has no more replacement.
#[derive(Clone, Debug)]
pub(super) struct VirtualAnnot {
    /// Any message to give with this annotation.
    pub message:  Option<String>,
    /// The severity of this annotation.
    pub severity: Severity,
    /// The range describing which part of the main span this annotation concerns.
    pub range:    Range,
}

// Conversion
impl<E> From<Annot<E>> for VirtualAnnot {
    #[inline]
    fn from(value: Annot<E>) -> Self {
        let Annot { replacement: _, message, severity, range } = value;
        Self { message, severity, range }
    }
}





/***** LIBRARY *****/
/// Given an [`AnnotGroup`], it will wraps its [`Span`] in a [`VirtualSpan`] and interleave slices
/// of the first with replacements embedded in the [`AnnotGroup`].
///
/// This implements the third step of the layouting algorithm detailled in [`layout`](super).
///
/// # Arguments
/// - `group`: An [`AnnotGroup`] to turn into a [`VirtualSpan`].
///
/// # Returns
/// A [`VirtualSpan`]. Note that it does not embed [`Annot`]s, but rather [`VirtualAnnot`]s who do
/// not carry replacements with them anymore.
pub fn apply_replace_annots<'s, S: Spannable<'s>>(group: AnnotGroup<'s, S>) -> VirtualSpan<'s, S> {
    let AnnotGroup { span, mut annots } = group;

    // Find the span's resolved range
    let source: &S = span.source();
    let source_len: usize = source.len();
    let mut rem_range: Range = Range::new(span.range().start_resolved(source_len).unwrap_or(0), span.range().end_resolved(source_len).unwrap_or(0));

    // Order the annotations by starting range. Since we've guaranteed ourselves in step 2 that any
    // annotation with a replace is unique, this will allow us to scan and slice the span quickly.
    annots.sort_by_key(|a| a.range);

    // Then: loop through the replacing annotations and slice
    let mut slices: Vec<VirtualSlice<S::Elem>> = Vec::with_capacity(2 * annots.len());
    let mut vannots: Vec<VirtualAnnot> = Vec::with_capacity(annots.len());
    let mut running_offset: isize = 0;
    for annot in annots {
        let Annot { replacement, message, severity, range } = annot;

        // If it's a replacement, then we start slicing
        let mut new_running_offset: isize = running_offset;
        if let Some(replacement) = replacement {
            // Add any source up to the start of the replacement
            // NOTE: We don't slice because we're not working with Span-relative indices, but
            // source-relative
            let pre_range: Range = rem_range.cut_up_to_overlap(&range);
            if !pre_range.is_empty() {
                slices.push(VirtualSlice::Source(pre_range));
            }
            rem_range = rem_range.cut_up_to_and_including_overlap(&range);

            // Add the replacement
            // SAFETY: We can unwrap because `replace_annots` only contains replacements that are
            // `Some`
            let replacement_len: usize = replacement.len();
            slices.push(VirtualSlice::Replace(replacement));

            // Keep tally of how much offset to apply to any following ranges
            new_running_offset = running_offset + (replacement_len as isize - range.len() as isize);
        }

        // Add the annotation with updated offset
        // Note that, for _this_ one, the first still has to use the OLD offset (it isn't
        // influenced by its own, potential replace). The second value IS influenced.
        vannots.push(VirtualAnnot { message, severity, range: range.offset_start_and_end(running_offset, new_running_offset) });
        running_offset = new_running_offset;
    }

    // Add the final span, if any
    if !rem_range.is_empty() {
        slices.push(VirtualSlice::Source(rem_range));
    }

    // OK, done!
    VirtualSpan { span, slices, annots: vannots }
}





/***** TESTS *****/
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_replace_annots() {
        let span = Span::new("Hello, world!");
        let annot = Annot { replacement: Some("Goodbye".into()), message: None, severity: Severity::Suggestion, range: Range::from(0..5) };
        let vspan = apply_replace_annots(AnnotGroup { span, annots: vec![annot] });
        let bytes: Vec<u8> = vspan.slices().flat_map(|s| s.iter().copied()).collect();
        println!("{}", String::from_utf8_lossy(&bytes));
        assert_eq!(bytes, b"Goodbye, world!");
        assert_eq!(vspan.annots.len(), 1);
        assert_eq!(vspan.annots[0].range, Range::from(0..7));

        let annot1 = Annot { replacement: Some(":".into()), message: None, severity: Severity::Suggestion, range: Range::from(5..6) };
        let annot2 = Annot { replacement: Some("?".into()), message: None, severity: Severity::Suggestion, range: Range::from(12..13) };
        let vspan = apply_replace_annots(AnnotGroup { span, annots: vec![annot1, annot2] });
        let bytes: Vec<u8> = vspan.slices().flat_map(|s| s.iter().copied()).collect();
        assert_eq!(bytes, b"Hello: world?");
        assert_eq!(vspan.annots.len(), 2);
        assert_eq!(vspan.annots[0].range, Range::from(5..6));
        assert_eq!(vspan.annots[1].range, Range::from(12..13));

        let annot1 = Annot { replacement: Some("Goodbye".into()), message: None, severity: Severity::Suggestion, range: Range::from(0..5) };
        let annot2 = Annot { replacement: Some("!!".into()), message: None, severity: Severity::Suggestion, range: Range::from(12..13) };
        let vspan = apply_replace_annots(AnnotGroup { span, annots: vec![annot1, annot2] });
        let bytes: Vec<u8> = vspan.slices().flat_map(|s| s.iter().copied()).collect();
        assert_eq!(bytes, b"Goodbye, world!!");
        assert_eq!(vspan.annots.len(), 2);
        assert_eq!(vspan.annots[0].range, Range::from(0..7));
        assert_eq!(vspan.annots[1].range, Range::from(14..16));
    }
}
