//  LAYOUTING ALGORITHM - v3.rs
//    by Lut99
//
//  Description:
//!   Third iteration of the infamous layouting algorithm for displaying
//!   [`Diag`]s.
//!
//!   # Algorithm
//!   The layouting algorithm is split into multiple steps. Each of those is
//!   described below:
//!   1. The first step splits all annotations into messages. In principle,
//!      every annotation ends up in the same message UNLESS a) their reference
//!      does not have a source ID; or b) they have a replacement suggestion.
//!   2. Using a [`Layouter`], we turn all 1D [`Range`](crate::loc::Range)s
//!      into 2D ones. How this is done precisely depends on how we layout the
//!      original source text. After this step, all ranges are local to a line
//!      and relative to the start of it.
//!   3. Run the placement algorithm for markers & messages associated to
//!      annotations. This is rendered to a buffer which then...
//!   4. Render each message using the [`Layouter`] again.
//

// Modules
pub mod layouter;
mod s1_messagify;
mod s2_layout;
mod s3_place;
mod s4_render;

// Imports/Exports
use std::fmt;

use layouter::Layouter;
use s1_messagify::s1_messagify;
use s2_layout::s2_layout;
use s3_place::s3_place;
use s4_render::s4_render;

use super::Diag;


/***** LIBRARY *****/
/// Helper struct that can be used to [`Display`](fmt::Display) a [`Diag`]nostic.
pub struct Display<'d, L, E> {
    layouter: L,
    diag:     &'d Diag<E>,
}

impl<'d, L: Layouter<E>, E> fmt::Display for Display<'d, L, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // STEP 1: Turn it into messages
        let messages = s1_messagify(self.diag);

        // STEP 2: Layouting
        let messages = s2_layout(&self.layouter, messages);

        // STEP 3: Placement
        let messages = s3_place(&self.diag.theme, &self.layouter, messages);

        // STEP 4: Rendering
        s4_render(f, &self.diag.theme, &self.layouter, messages)
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
