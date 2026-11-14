//  STEP 1 - SPLITTING.rs
//    by Lut99
//
//  Description:
//!   Defines step 1 of the [layouting algorith](super).
//!
//!   This step is pretty simple; we just sort by source ID.
//

use std::collections::HashMap;

use super::super::Severity;
use super::super::annots::Annotation;
use crate::loc::Range;


/***** HELPERS *****/
/// An [`Annotation`]-equivalent that is per-source.
pub(super) struct Annot<'d, E> {
    pub repl:  Option<&'d [E]>,
    pub msg:   Option<&'d str>,
    pub sev:   Severity,
    pub range: Range,
}





/***** LIBRARY *****/
/// Defines step 1 of the [layouting algorith](super).
///
/// This step is pretty simple; we just sort by source ID.
///
/// # Arguments
/// - `annots`: The list of [`Annotation`]s to process.
///
/// # Returns
/// A collection of all `annots` sorted by source IDs. The resulting [`Annot`] is equivalent to
/// each given [`Annotation`], except that it has [`Annot::range`] instead of [`Annotation::loc`].
pub fn s1_split<'d, E>(annots: &'d [Annotation<E>]) -> HashMap<Option<u64>, Vec<Annot<'d, E>>> {
    let mut res = HashMap::<Option<u64>, Vec<Annot<'d, E>>>::new();
    for annot in annots {
        res.entry(annot.loc.source).or_default().push(Annot {
            repl:  annot.repl.as_ref().map(Vec::as_slice),
            msg:   annot.msg.as_ref().map(String::as_str),
            sev:   annot.sev,
            range: annot.loc.range,
        });
    }
    res
}
