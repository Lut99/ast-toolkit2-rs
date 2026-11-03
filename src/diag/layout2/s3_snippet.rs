//  STEP 3 - SNIPPETS.rs
//    by Lut99
//
//  Description:
//!   Defines step 3 of the [layouting algorith](super).
//

use std::collections::HashMap;

use super::s2_layout::LineAnnot;


/***** CONSTANTS *****/
/// Maximum number of lines that are between two lines before they are separated in different
/// snippets.
pub const MAX_UNTOUCHED_LINES: u64 = 1;





/***** HELPER FUNCTIONS *****/
fn split_vertically<'d, E>(lines: &mut Vec<Line<'d, E>>) -> Vec<Line<'d, E>> {
    // Find if there is a point at which to split the lines
    for i in 0..lines.len() {
        if lines[i].line - lines[i - 1].line > 1 + MAX_UNTOUCHED_LINES {
            // Pop everything from i onwards off the lines
            return lines.drain(i..).collect();
        }
    }
    // Else, nothing to split
    Vec::new()
}





/***** HELPERS *****/
/// Represents an entire snippet.
pub struct Snippet<'d, E> {
    /// The source belonging to this snippet.
    pub source_id: Option<u64>,
    /// The lines in this snippet.
    pub lines:     Vec<Line<'d, E>>,
}

pub struct Line<'d, E> {
    /// The line number.
    pub line:   u64,
    /// The list of annotations on this line.
    pub annots: Vec<LineAnnot<'d, E>>,
}





/***** LIBRARY *****/
/// Defines step 3 of the [layouting algorith](super).
///
/// # Arguments
/// - `annots`: The already sorted by-source, by-line annotations.
///
/// # Returns
/// A collection of [`Snippet`]s that sort all annotations by snippet, by line.
pub fn s3_snippet<'d, E>(annots: HashMap<Option<u64>, HashMap<u64, Vec<LineAnnot<'d, E>>>>) -> Vec<Snippet<'d, E>> {
    // First: split by source
    let mut snippets: Vec<Snippet<'d, E>> = annots
        .into_iter()
        .map(|(source_id, annots)| Snippet { source_id, lines: annots.into_iter().map(|(line, annots)| Line { line, annots }).collect() })
        .collect();

    // Then: split snippets that are too great a distance vertically
    let mut extra_snippets = Vec::new();
    for mut snippet in &mut snippets {
        snippet.lines.sort_by_key(|s| s.line);
        // Keep splitting, because the resulting extra snippet may also require vertical splits
        loop {
            let split_off = split_vertically(&mut snippet.lines);
            if !split_off.is_empty() {
                let source_id = snippet.source_id;
                extra_snippets.push(Snippet { source_id, lines: split_off });
                // SAFETY: We added it just above here
                snippet = unsafe { extra_snippets.last_mut().unwrap_unchecked() };
            } else {
                break;
            }
        }
    }
    snippets.extend(extra_snippets);

    // Then: potentially split snippets horizontally
    // TODO, one day, maybe

    // Done
    snippets
}
