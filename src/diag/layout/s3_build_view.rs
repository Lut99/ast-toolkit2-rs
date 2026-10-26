//  STEP 3: BUILDING A VIEW.rs
//    by Lut99
//
//  Description:
//!   This module implements the third step of the layouting algorithm
//!   described in the [`layout`](super)-module.
//!
//!   Given a group of [`Annot`]s, organises them in a [`View`] that starts
//!   abstractly defining the resulting source snippet. This function
//!   essentially just re-organises the data, but usefully so.
//

use std::borrow::Cow;

use super::super::annots::Severity;
use super::s1_grouping;
use crate::loc::Range;


/***** DATA STRUCTURES *****/
/// Defines a further simplified version of an annotation that doesn't know about replacements anymore.
pub struct Annot<'a> {
    /// Any message to write.
    pub msg:   Option<Cow<'a, str>>,
    /// The severity to annotate with.
    pub sev:   Severity,
    /// The range where we physically annotate.
    pub range: Range,
}
impl<'a, E: Clone> From<s1_grouping::Annot<'a, E>> for Annot<'a> {
    #[inline]
    fn from(value: s1_grouping::Annot<'a, E>) -> Self {
        let s1_grouping::Annot { repl, msg, sev, range } = value;
        assert!(repl.is_none());
        Self { msg, sev, range }
    }
}



/// Defines a single area in a [`View`].
pub enum Area<'a, E: Clone> {
    /// It's a piece of source text that we'll need load later.
    Source { annots: Vec<Annot<'a>>, range: Range },
    /// It's a piece of replaced text we already know. Note that this is always annotated, so hence
    /// we embed the annotation here.
    Replaced { repl: Cow<'a, [E]>, sev: Severity, msg: Option<Cow<'a, str>> },
}



/// First abstraction of a resulting snippet of annotated source text.
///
/// Simply organises annotations chronologically and determines what will be original source text
/// and what will be replaced text.
pub struct View<'a, E: Clone> {
    /// The full range of the view.
    pub range: Range,
    /// The sequential list of areas that make up this View.
    ///
    /// It is promised that these are tightly coupled; i.e., rendering these one after another does
    /// not cause confusion.
    pub areas: Vec<Area<'a, E>>,
}





/***** LIBRARY *****/
/// Given a set of annotations, it will organise them into a [`View`].
///
/// This implements the third step of the layouting algorithm detailled in [`layout`](super).
///
/// # Arguments
/// - `group`: A single group of [`Annot`]s that will be reorganized into a [`View`].
///
/// # Returns
/// The [`View`] that starts defining an abstract overview of the resulting snippet.
pub fn build_view<'a, E: Clone>(mut group: Vec<s1_grouping::Annot<'a, E>>) -> View<'a, E> {
    // First, we sort the group by position to easily find neighbours and overlapping and such
    group.sort_by_key(|a| a.range.pos);

    // Then go through them to build a coherent view
    // The trick is: in the previous step, we ensured that any remaining replace is now uniquely
    // spanning an area. These are boundaries: all annotations before it end before it, and any
    // annotation after it starts after it.
    // As such, we simply collect everything up to a replacement in the same area, and then use
    // replacement areas as boundaries between these.
    let mut areas: Vec<Area<'a, E>> = Vec::with_capacity(group.len());
    let (mut annots, mut range): (Vec<Annot<'a>>, Option<Range>) = (Vec::with_capacity(4), None);
    for annot in group {
        // Check what's what
        if let Some(repl) = annot.repl {
            // First, add everything up to this point
            if !annots.is_empty() {
                // SAFETY: We can unwrap here because `annots` is only ever non-empty when there
                // is a `range`
                areas.push(Area::Source { annots: annots.drain(..).collect(), range: range.take().unwrap() });
            }

            // Then add the replacement area and move to the next
            areas.push(Area::Replaced { repl, sev: annot.sev, msg: annot.msg });
            continue;
        }

        // Else, we just collect a single source text area
        if let Some(range) = &mut range {
            range.extend(annot.range);
        } else {
            range = Some(annot.range);
        };
        annots.push(annot.into());
    }

    // Don't forget to add any remaining in the buffer
    if !annots.is_empty() {
        // SAFETY: We can unwrap here because `annots` is only ever non-empty when there
        // is a `range`
        areas.push(Area::Source { annots: annots.drain(..).collect(), range: range.take().unwrap() });
    }

    // Done
    View {
        // Build a range spanning the entire view
        range: areas
            .iter()
            .filter_map(|a| match a {
                Area::Source { range, .. } => Some(*range),
                _ => None,
            })
            .collect(),
        areas,
    }
}
