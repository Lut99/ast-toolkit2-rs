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
//!      depends on how we layout the original source text.
//!   3. We internally sort each source group into _snippets_: annotations that
//!      will be shown together in the same window. This will depend on
//!      physical distance to each other.
//!   4. Turn coordinates in each snippet from source coordinates to line
//!      coordinate space. I.e., all of them become relative to the start of
//!      the line they are on. This is also where we inject "suggestion" text
//!      by moving all annotations that are physically coming after it.
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
pub use s2_layout::{Layouter, LineRange, Range2D};

use super::Diag;
use crate::diag::layout2::s2_layout::s2_layout;
use crate::diag::layout2::s3_snippet::s3_snippet;


/***** LIBRARY *****/
/// Helper struct that can be used to [`Display`](fmt::Display) a [`Diag`]nostic.
pub struct Display<'l, 'd, L, E> {
    layouter: &'l L,
    diag:     &'d Diag<E>,
}

impl<'l, 'd, L: Layouter<E>, E> fmt::Display for Display<'l, 'd, L, E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        // STEP 1: Split
        let mut annots = s1_split(&self.diag.annots);

        // STEP 1.5: Handle `None` source
        if let Some(annots) = annots.remove(&None) {
            todo!()
        }
        let annots = annots.into_iter().map(|(k, v)| (k.unwrap(), v)).collect();

        // STEP 2: Layout
        let annots = s2_layout(self.layouter, annots);

        // STEP 3: Snippets
        let snippets = s3_snippet(annots);

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
    pub const fn display<'l, L: Layouter<E>>(&self, layouter: &'l L) -> Display<'l, '_, L, E> { Display { layouter, diag: self } }
}
