//  STEP 2: SPLITTING REPLACEMENTS.rs
//    by Lut99
//
//  Description:
//!   This module implements the second step of the layouting algorithm
//!   described in the [`layout`](super)-module.
//!
//!   Given an [`AnnotGroup`], it will find all [`Annot`]s with a non-[`None`]
//!   `replacement` that overlap with another [`Annot`] in that group. For each
//!   of those, it will yank out the replacement; inject it as a message; and,
//!   if there already was a message, append a new [`Annot`] representing it.
//

use std::fmt::{Display, Write as _};

use ast_toolkit_span::Spannable;

use super::s1_grouping::{Annot, AnnotGroup, Range};
use crate::annotations::Severity;


/***** LIBRARY *****/
/// Given an [`AnnotGroup`], it will find all [`Annot`]s with a non-[`None`] `replacement` that
/// overlap with another [`Annot`] in that group. For each of those, it will yank out the
/// replacement; inject it as a message; and, if there already was a message, append a new
/// [`Annot`] representing it.
///
/// This implements the second step of the layouting algorithm detailled in [`layout`](super).
///
/// # Arguments
/// - `group`: An [`AnnotGroup`] to split any non-unique replacing [`Annot`]s in.
///
/// # Returns
/// Nothing, but does modify the given `group` to have it such that no [`Annot`]s exist anymore
/// that have a non-[`None`] `replacement` and overlap with another in the group.
#[inline]
pub fn split_replace_annots<'s, S: Spannable<'s>>(group: &mut AnnotGroup<'s, S>)
where
    S::Elem: Display,
{
    // First, determine for every annotation with a replacement if it is unique
    let mut nonunique_annots: Vec<usize> = Vec::with_capacity(4);
    'annots: for (i, annot) in group.annots.iter().enumerate() {
        if annot.replacement.is_some() {
            // Find any others in the same range
            for (j, annot2) in group.annots.iter().enumerate() {
                if i == j {
                    continue;
                }
                if annot.range.overlaps_with(&annot2.range) {
                    nonunique_annots.push(i);
                    continue 'annots;
                }
            }
        }
    }

    // For the non-unique ones, tear them in two messages saying there is a suggestion and one
    // doing the message (if any).
    for i in nonunique_annots {
        // SAFETY: We can unwrap because we only put indices we've seen in `nonunique_annots`
        let annot: &mut Annot<S::Elem> = unsafe { group.annots.get_unchecked_mut(i) };
        let message: Option<String> = annot.message.take();

        // Replace the message with one doing the suggestion
        annot.message = Some({
            // SAFETY: We can unwrap because we only add elements to `nonunique_annots` if
            // `annot.replacement` is there
            let replace: Vec<S::Elem> = unsafe { annot.replacement.take().unwrap_unchecked() };
            let mut message: String = String::with_capacity(14 + replace.len() + 1);
            message.push_str("Replace with '");
            for value in replace {
                write!(&mut message, "{}", value).unwrap();
            }
            message.push('\'');
            message
        });

        // Add a new annotation with the same range and everything doing the original message
        // (so it's below the suggestion)
        if message.is_some() {
            let severity: Severity = annot.severity;
            let range: Range = annot.range;
            group.annots.push(Annot { replacement: None, message, severity, range });
        }
    }
}
