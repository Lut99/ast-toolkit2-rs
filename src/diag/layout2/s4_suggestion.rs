//  STEP 4 - SUGGESTIONS.rs
//    by Lut99
//
//  Description:
//!   Defines step 4 of the [layouting algorithm](super).
//

use super::s2_layout::LineAnnot;
use super::s3_snippet::Snippet;


/***** HELPERS *****/
pub struct RenderableSnippet<'d, E> {
    source_id: u64,
    lines:     Vec<RenderableLine<'d, E>>,
}



pub struct RenderableLine<'d, E> {
    /// The line number.
    pub line:   u64,
    /// The list of when to pull source text where.
    pub source: Vec<Text<'d, E>>,
    /// The list of annotations on this line.
    pub annots: Vec<LineAnnot<'d, E>>,
}



pub enum Text<'d, E> {
    Source(u64, u64),
    Replace(&'d [E]),
}





/***** LIBRARY *****/
/// Defines step 4 of the [layouting algorith](super).
///
/// # Arguments
/// - `snippets`: A list of the [`Snippet`]s resulting from the previous step.
///
/// # Returns
/// A collection of [`RenderableSnippet`]s that are updated to include a map of where to render OG
/// source and where to render suggestion source and updated ranges to be correct for this map.
pub fn s4_suggestion<'d, E>(snippets: Vec<Snippet<'d, E>>) -> Vec<RenderableSnippet<'d, E>> {
    let mut res = Vec::new();
    for snippet in snippets {
        let mut lines = Vec::new();
        for mut line in snippet.lines {
            // First, build source overzealously by simply mapping annotations one-to-one.
            line.annots.sort_by_key(|a| a.range.0);
            let mut source: Vec<(usize, Text<'d, E>)> = line
                .annots
                .iter_mut()
                .enumerate()
                .map(|(i, a)| if let Some(repl) = a.repl.take() { (i, Text::Replace(repl)) } else { (i, Text::Source(a.range.0, a.range.1)) })
                .collect();

            // Then trim the source by 

            // Go through the annotations to decide which are replaces
            let mut source = Vec::new();

            // Done, build the line
            lines.push(RenderableLine { line: line.line, source, annots: line.annots })
        }
        res.push(RenderableSnippet { source_id: snippet.source_id, lines });
    }
    res
}
