//  LAYOUTING ALGORITHM - v2.rs
//    by Lut99
//
//  Description:
//!   A second version of the layouting algorithm for `ast-toolkit2`'s
//!   [`Diag`](super::Diag)s.
//!
//!   This version observes that "inline" annotations are not projected over
//!   the source text; instead, they are placed compactly in special lines in
//!   the output render.
//!
//!   This will allow for a lot more virtual computation during layouting.
//!
//!   # Algorithm
//!   The steps of the algorithm are as follows:
//!   1. Split incoming [`Annotation`](super::annots::Annotation) by source ID
//!      stored in their [`Loc`](crate::loc::Loc)s. This will leave us with
//!      groups of annotations with [`Range`](crate::loc::Range)s pointing to
//!      the same source.
//!   2. Using a [`Layouter`](s2_layout::Layouter), we turn all 1D
//!      [`Range`](crate::loc::Range)s into 2D ones. How this is done precisely
//!      depends on how we layout the original source text. After this step,
//!      all ranges are local to a line and relative to the start of it. Only
//!      ranges without a source text are exempted, and instead have no range.
//!   3. We internally sort each source group into _snippets_: annotations that
//!      will be shown together in the same window. This will depend on
//!      physical distance to each other. Furthermore, rangeless annotations
//!      ánd replacements will get their own snippets, too.
//!   4. Inject suggestions into the snippets. This is done virtually, by
//!      building a continuous overview of when to pull source text from the OG
//!      source and when to use replacement text.
//!   5. Now, render all marks and annotations in special "annotation" lines.
//!      This is where placement happens; decide an optimal layout for all this
//!      information.
//!   6. Render the source text with the annotation lines using a
//!      [`Layouter`](s2_layout::Layouter) again. It gets to decide how to
//!      colour the text what, what colour to give it, etc.
//

// Modules
#[cfg(feature = "sources")]
pub mod layouters;
mod s1_split;
mod s2_layout;
mod s3_snippet;
mod s4_suggestion;
mod s5_placement;
mod s6_render;

// Imports & Exports
use std::fmt::{self, Formatter, Result as FResult};

use s1_split::s1_split;
use s2_layout::s2_layout;
pub use s2_layout::{Layouter, LineRange, Range2D};
use s3_snippet::s3_snippet;
use s4_suggestion::s4_suggestion;

use super::Diag;


/***** LIBRARY *****/
/// Helper struct that can be used to [`Display`](fmt::Display) a [`Diag`]nostic.
pub struct Display<'d, L, E> {
    layouter: L,
    diag:     &'d Diag<E>,
}

impl<'d, L: Layouter<E>, E> fmt::Display for Display<'d, L, E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        // STEP 1: Split
        let mut annots = s1_split(&self.diag.annots);

        // STEP 2: Layout
        let annots = s2_layout(&self.layouter, annots);

        // STEP 3: Snippets
        let snippets = s3_snippet(self.diag.sev, self.diag.msg.as_str(), self.diag.code.as_ref().map(String::as_str), annots);

        // STEP 4: Suggestion
        let snippets = s4_suggestion(snippets);

        /* Done */
        Ok(())
    }
}





/***** IMPLS *****/
impl<E> Diag<E> {
    /// Allows you to [`Display`](fmt::Display) the Diag.
    ///
    /// This will render the error.
    ///
    /// # Arguments
    /// - `layouter`: A [`Layouter`] that decides how to show the original source text.
    ///
    /// # Returns
    /// A [`Display`] implementing [`Display`](fmt::Display).
    #[inline]
    pub const fn display<L: Layouter<E>>(&self, layouter: L) -> Display<'_, L, E> { Display { layouter, diag: self } }
}
