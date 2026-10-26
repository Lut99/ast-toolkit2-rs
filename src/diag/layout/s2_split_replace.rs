//  STEP 2: SPLITTING REPLACEMENTS.rs
//    by Lut99
//
//  Description:
//!   This module implements the second step of the layouting algorithm
//!   described in the [`layout`](super)-module.
//!
//!   Given a group of [`Annot`]s, it will find all with a non-[`None`]
//!   `repl`acement field that overlap with other [`Annot`]s in that group. For
//!   each of those, it will yank out the `repl`acement; inject it as a `msg`
//!   or, if there already was a message, append a new [`Annot`] representing
//!   it as a message.
//

use std::borrow::Cow;

use super::Renderer;
use super::s1_grouping::Annot;


/***** LIBRARY *****/
/// Given a set of annotations, it will find all [`Annot`]s with a non-[`None`] `repl`acement that
/// overlap with another [`Annot`] in that group. For each of those, it will yank out the
/// replacement and either set it as message or, if it had one already, splits it off as a new
/// annotation.
///
/// This implements the second step of the layouting algorithm detailled in [`layout`](super).
///
/// # Arguments
/// - `group`: A collection of [`Annot`]s to split any non-unique replacing [`Annot`]s in.
///
/// # Returns
/// Nothing, but does modify the given `group` to have it such that no [`Annot`]s exist anymore
/// that have a non-[`None`] `repl`acement and overlap with another in the group.
pub fn split_replace_annots<'a, R: ?Sized + Renderer<E>, E: Clone>(renderer: &R, group: &mut Vec<Annot<'a, E>>) {
    // First, determine for every annotation with a replacement if it is unique
    let mut nonunique_annots: Vec<usize> = Vec::with_capacity(4);
    'annots: for (i, annot) in group.iter().enumerate() {
        if annot.repl.is_some() {
            // Find any others in the same range
            for (j, annot2) in group.iter().enumerate() {
                if i == j {
                    continue;
                }
                if annot.range.overlaps_with(annot2.range) {
                    nonunique_annots.push(i);
                    continue 'annots;
                }
            }
        }
    }

    // For the non-unique ones, tear them in two messages saying there is a suggestion and one
    // doing the message (if any).
    for i in nonunique_annots {
        let new_annot: Annot<'a, E> = {
            // SAFETY: We can unwrap because we only put indices we've seen in `nonunique_annots`
            let annot: &mut Annot<'a, E> = unsafe { group.get_unchecked_mut(i) };
            // SAFETY: This is OK because only annots with a repl can land in `nonunique_annots`
            let repl: Cow<'a, [E]> = unsafe { annot.repl.take().unwrap_unchecked() };

            if annot.msg.is_none() {
                // Easy case: just set the replacement as a message!
                annot.msg = Some(Cow::Owned(format!("Replace this with {}", renderer.display_seq(&repl))));
                continue;
            }

            // Else, create a new message and escape the scope to be able to add to `group`
            Annot {
                repl:  None,
                msg:   Some(Cow::Owned(format!("Replace this with {}", renderer.display_seq(&repl)))),
                sev:   annot.sev,
                range: annot.range,
            }
        };
        group.push(new_annot);
    }
}
