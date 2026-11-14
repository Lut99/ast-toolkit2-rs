//  STEP 3 - SNIPPETS.rs
//    by Lut99
//
//  Description:
//!   Defines step 3 of the [layouting algorith](super).
//

use std::collections::HashMap;

use super::s2_layout::LineAnnot;
use crate::diag::Severity;


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
pub enum Snippet<'d, E> {
    // A snippet that merely has annotations
    Annots {
        /// The source belonging to this snippet.
        source_id: u64,
        /// The main severity of the Diagnostic.
        sev: Severity,
        /// Any code to display with the main message.
        code: Option<&'d str>,
        /// The main message of this Diagnostic.
        msg: &'d str,
        /// The lines in this snippet.
        lines: Vec<ReplacelessLine<'d>>,
    },
    /// A snippet with a single replacement.
    Replace {
        /// The source belonging to this snippet.
        source_id: u64,
        /// The message of this snippet.
        msg: Option<&'d str>,
        /// The severity of this snippet.
        sev: Severity,
        /// The replacement text.
        lines: Vec<ReplacingLine<'d, E>>,
    },
    /// A snippet without source.
    Sourceless {
        /// The message.
        msg: &'d str,
        /// The severity.
        sev: Severity,
    },
}

pub struct ReplacelessLine<'d> {
    /// The line number.
    pub line:   u64,
    /// The list of annotations on this line.
    pub annots: Vec<ReplacelessAnnot<'d>>,
}

pub struct ReplacelessAnnot<'d> {
    pub msg:   Option<&'d str>,
    pub sev:   Severity,
    pub range: (u64, u64),
}

pub struct ReplacingLine<'d, E> {
    /// The line number.
    pub line:   u64,
    /// The list of annotations on this line.
    pub annots: Vec<ReplacingAnnot<'d, E>>,
}

pub struct ReplacingAnnot<'d, E> {
    pub repl:  &'d [E],
    pub range: (u64, u64),
}





/***** LIBRARY *****/
/// Defines step 3 of the [layouting algorith](super).
///
/// # Arguments
/// - `main_sev`: The severity of the main diagnostic.
/// - `main_msg`: The message of the main diagnostic.
/// - `main_code`: The code of the main diagnostic, if any.
/// - `main_annots`: The already sorted by-source, by-line annotations.
///
/// # Returns
/// A collection of [`Snippet`]s that sort all annotations by snippet, by line.
pub fn s3_snippet<'d, E>(
    main_sev: Severity,
    main_msg: &'d str,
    main_code: Option<&'d str>,
    main_annots: HashMap<Option<u64>, HashMap<u64, Vec<LineAnnot<'d, E>>>>,
) -> Vec<Snippet<'d, E>> {
    // First, divide the given annotation by type of snippet
    let mut snippets = Vec::<Snippet<'d, E>>::new();
    for (source_id, mut main_lines) in main_annots {
        let Some(source_id) = source_id else {
            // It's the special `None`. No ranges to include!
            // So generate a new snippet per-message.
            let line = main_lines.remove(&0).unwrap();
            #[cfg(debug_assertions)]
            assert!(main_lines.is_empty());
            for annot in line {
                let Some(msg) = annot.msg else { continue };
                snippets.push(Snippet::Sourceless { msg, sev: annot.sev });
            }
            continue;
        };

        // Go thru the lines
        let mut lines = Vec::<ReplacelessLine<'d>>::new();
        for (line, line_annots) in main_lines {
            let mut line = ReplacelessLine { line, annots: Vec::new() };
            for annot in line_annots {
                match annot.repl {
                    Some(repl) => {
                        // It's a replace - which becomes its own snippet
                        snippets.push(Snippet::Replace { source_id, msg: annot.msg, sev: annot.sev, lines: () })
                    },
                    None => line.annots.push(ReplacelessAnnot { msg: annot.msg, sev: annot.sev, range: annot.range.unwrap() }),
                }
            }
            if !line.annots.is_empty() {
                lines.push(line);
            }
        }

        // Add a snippet for this source's annotations
        snippets.push(Snippet::Annots { source_id, sev: main_sev, code: main_code, msg: (), lines })
    }

    // First: split by source
    let mut snippets: Vec<Snippet<'d, E>> = annots
        .into_iter()
        .flat_map(|(source_id, annots)| match source_id {
            Some(source_id) => Box::new(annots.into_iter().flat_map(|(line, annots)| {
                // Split the annotations into replacing or not
                let mut replacing_annots = Vec::new();
                let mut replaceless_annots = Vec::new();
                for annot in annots {
                    match annot.repl {
                        Some(repl) => replacing_annots.push(ReplacingAnnot { repl, msg: annot.msg, sev: annot.sev, range: annot.range.unwrap() }),
                        None => replaceless_annots.push(ReplacelessAnnot { msg: annot.msg, sev: annot.sev, range: annot.range.unwrap() }),
                    }
                }

                // Generate snippets out of that
            })) as Box<dyn Iterator<Item = Snippet<'d, E>>>,
            None => {
                Box::new(annots.into_iter().flat_map(|(_, a)| a.into_iter().filter_map(|a| a.msg.map(|msg| Snippet::Sourceless { msg, sev: a.sev }))))
            },
        })
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
