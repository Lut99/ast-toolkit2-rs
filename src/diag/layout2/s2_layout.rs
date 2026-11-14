//  STEP 2 - LAYOUT.rs
//    by Lut99
//
//  Description:
//!   Defines step 2 of the [layouting algorith](super).
//

use std::collections::HashMap;

use super::s1_split::Annot;
use crate::diag::Severity;
use crate::loc::Range;


/***** HELPERS *****/
pub(super) struct LineAnnot<'d, E> {
    pub repl:  Option<&'d [E]>,
    pub msg:   Option<&'d str>,
    pub sev:   Severity,
    pub range: Option<(u64, u64)>,
}





/***** AUXILLARY *****/
/// A 2 dimensional version of a [`Range`].
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Range2D {
    /// The lines that are touched by this range, together with where the line is touched.
    pub lines: Vec<LineRange>,
}

/// A range only spanning a single line.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct LineRange {
    /// The line this range lives on.
    pub line: u64,
    /// The start pos of the range, relative to the start of the line.
    pub pos:  u64,
    /// The length of the range, bounded by the end of the line.
    pub len:  u64,
}





/***** INTERFACES *****/
/// Represents an algorithm (with state) for rendering source text somehow.
pub trait Layouter<E> {
    /// Turns a set of 1-dimensional [`Range`]s into 2-dimensional [`LineRange`]s.
    ///
    /// The layouter should compute where each given [`Range`] lies in 2D-space on the source text;
    /// then, it should split each range into one [`LineRange`] for every line it touches.
    ///
    /// # Arguments
    /// - `source_id`: The ID of the source for which we're looking. It is expected that the
    ///   [`Layouter`] can resolve this to a physical source by itself.
    /// - `ranges`: The list of [`Range`]s to process.
    ///
    /// # Returns
    /// A collection of [`Range2D`], one for every [`Range`], describing how it is split over
    /// lines.
    fn layout(&self, source_id: u64, ranges: Vec<Range>) -> Vec<Range2D>;

    /// Function for rendering the output of the layouting algorithm.
    fn render(&self);
}





/***** LIBRARY *****/
/// Defines step 2 of the [layouting algorith](super).
///
/// # Arguments
/// - `layouter`: A [`Layouter`] that computes 2D variants of [`Range`]s.
/// - `annots`: The list of [`Annotation`]s to process.
///
/// # Returns
/// A collection of all `annots` sorted by source IDs and line number. Each resulting [`LineAnnot`]
/// is equivalent to the input [`Annot`]s, excepting that its range is now scoped to a single line.
/// There may also be more if any original annot spanned multiple lines.
pub fn s2_layout<'d, E>(
    layouter: &impl Layouter<E>,
    annots: HashMap<Option<u64>, Vec<Annot<'d, E>>>,
) -> HashMap<Option<u64>, HashMap<u64, Vec<LineAnnot<'d, E>>>> {
    // Let the layouter do its job
    let mut res = HashMap::<Option<u64>, HashMap<u64, Vec<LineAnnot<'d, E>>>>::with_capacity(annots.len());
    for (source_id, annots) in annots {
        let Some(source_id) = source_id else {
            // These cannot be mapped to a source text - instead, leave their annotations empty
            res.entry(None)
                .or_default()
                .insert(0, annots.into_iter().map(|a| LineAnnot { repl: a.repl, msg: a.msg, sev: a.sev, range: None }).collect());
            continue;
        };

        // Compute the range maps
        let ranges = layouter.layout(source_id, annots.iter().map(|a| a.range).collect());
        #[cfg(debug_assertions)]
        if annots.len() != ranges.len() {
            panic!("You must make sure that `Layouter::layout()` returns one `Range2D` per input `Range`!")
        }

        // Turn each of those into multiple annotations
        let mut line = HashMap::<u64, Vec<LineAnnot<'d, E>>>::new();
        for (annot, range) in annots.into_iter().zip(ranges) {
            // Always add the first range given
            let mut iter = range.lines.into_iter();
            let Some(first) = iter.next() else {
                panic!("You must make sure that `Layouter::layout()` always returns at least one `LineRange` per `Range2D`!")
            };
            line.entry(first.line).or_default().push(LineAnnot {
                repl:  annot.repl,
                msg:   annot.msg,
                sev:   annot.sev,
                range: Some((first.pos, first.len)),
            });

            // Any others will not do anything complex
            while let Some(range) = iter.next() {
                line.entry(range.line).or_default().push(LineAnnot {
                    repl:  None,
                    msg:   None,
                    sev:   annot.sev,
                    range: Some((range.pos, range.len)),
                });
            }
        }

        // Done, save this line...
        res.insert(Some(source_id), line);
    }
    // ...and save the result
    res
}
