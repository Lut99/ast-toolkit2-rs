//  STEP 1: GROUPING.rs
//    by Lut99
//
//  Description:
//!   This module implements the first step of the layouting algorithm
//!   described in the [`layout`](super)-module.
//!
//!   Given a list of [`Annotation`]s, it will split them into several
//!   groups of [`Annot`]s who are unified by the same source. Accordingly,
//!   each of those only considers a [`Range`] instead of a [`Loc`].
//

use std::borrow::Cow;
use std::collections::HashMap;

use super::super::annots::{Annotation, Severity};
use crate::loc::Range;


/***** DATA STRUCTURES *****/
/// Represents a simplified [`Annotation`] where its [`Loc`]'s the original
/// source is externalized.
pub(super) struct Annot<'a, E: Clone> {
    /// Any replacement to do for this annotation.
    pub repl:  Option<Cow<'a, [E]>>,
    /// Any message to give with this annotation.
    pub msg:   Option<Cow<'a, str>>,
    /// The severity of this annotation.
    pub sev:   Severity,
    /// The range describing which part of the main span this annotation concerns.
    pub range: Range,
}

// Conversion
impl<'a, E: Clone> From<&'a Annotation<E>> for Annot<'a, E> {
    #[inline]
    fn from(value: &'a Annotation<E>) -> Self {
        let Annotation { repl, msg, sev, loc } = value;

        // Build self
        Self {
            repl:  repl.as_ref().map(Vec::as_slice).map(Cow::Borrowed),
            msg:   msg.as_ref().map(String::as_str).map(Cow::Borrowed),
            sev:   *sev,
            range: loc.range,
        }
    }
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
/// A set of [`Annot`]s that are grouped by source ID.
#[inline]
pub fn group_annots<'a, E: Clone>(annots: impl IntoIterator<Item = &'a Annotation<E>>) -> HashMap<Option<u64>, Vec<Annot<'a, E>>> {
    let iter = annots.into_iter();
    let size_hint: (usize, Option<usize>) = iter.size_hint();
    let mut res: HashMap<Option<u64>, Vec<Annot<E>>> = HashMap::with_capacity(size_hint.1.unwrap_or(size_hint.0));
    for annot in iter {
        // Check if we've seen annotations of this source before
        let source_id: Option<u64> = annot.loc.source;
        if let Some(group) = res.get_mut(&source_id) {
            group.push(annot.into());
        } else {
            res.insert(source_id, vec![annot.into()]);
        }
    }
    res
}
