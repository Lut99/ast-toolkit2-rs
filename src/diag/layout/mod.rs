//  MOD.rs
//    by Lut99
//
//  Description:
//!   Defines everything relating to rendering source snippets with
//!   annotations.
//!
//!   This process is quite the headbreaker to me to do it elegantly and in a
//!   way that doesn't require me to do everything all at once. Hence, this is
//!   implemented using the following pipeline that attempts to manage the
//!   complexity:
//!   1. _Grouping:_
//!
//!      In the first step, we take the heap of annotation spans and group them
//!      together by those having the same source (first). After this, we will
//!      render one snippet per group.
//!
//!      RESULT: We stop caring about unique sources.
//!
//!      RESULT: All [`Loc`](ast_toolkit2::Loc)s in annotations are replaced
//!      with their `Range`s only.
//!   2. _Splitting overlapping replaces:_
//!
//!      In the second step, we examine any annotation with a `replace` whether
//!      it uniquely spans an area. If not, we take away its replace, make its
//!      message something that describes the replace instead and, if it had a
//!      message originally, create a new annotation for that message.
//!
//!      RESULT: We stop caring about what to do with an annotation when it
//!      spans a replaced bit of source text.
//!
//!      RESULT: No more annotations appear in the list that have a replace and
//!      overlap with another one in the list.
//!   3. _Organizing annotations:_
//!      
//!      In the third step, we collect all annotations and put them in a
//!      sequential datastructure, the [`View`](s3_build_view::View). This will
//!      be an abstract overview of the source text that marks areas to be
//!      highlighted source text, unhighlighted source text or replaced source
//!      text.
//!
//!      RESULT: We stop thinking about annotations as loose items and instead
//!      start building a coherent snippet.
//!      
//!      RESULT: Replacements are removed from annotations and instead treated
//!      as text that needs to be loaded differently.
//!   4. _Layouters:_
//!
//!      In the fourth step, we apply [`Renderer::layout()`] to turn the
//!      abstract view into something concrete and visual, depending on the
//!      layouter's preference. This concrete and visual is stored in a
//!      [`Layout`](s4_layout::Layout) that no longer does ranges pointing in
//!      the source text, but rather embeds information about when to start-
//!      and stop annotations in the buffer itself.
//!
//!      RESULT: We unlock any visual information we need to place annotations.
//!
//!      RESULT: We stop caring about ranges, as the flags will signpost where
//!      to apply them instead.
//!
//!      RESULT: We obtain a populated buffer that carries the visual
//!      representation of the source text as well as annotations in flag form.
//!   5. _Placing:_
//!
//!      In the fifth and arguably main step, we apply the placing algorithm to
//!      turn the flags into newly populated cells in the cellbuffer.
//!      Optionally, the algorithm has the freedom to introduce new annotation
//!      lines for it to place the markings in instead of in-source.
//!
//!      RESULT: We stop caring about annotations.
//!
//!      RESULT: The [`CellBuffer`] is now free of annotation flags, and
//!      instead purely consists of marked cells, each of which corresponds to
//!      a character in the resulting output string.
//!   6. _Rendering:_
//!
//!      In the sixth and final step, we render the resulting cellbuffer to
//!      some output and color it by applying some
//!      [`Theme`](crate::themes::Theme). This results in an annotated, styled
//!      source snippet.
//!
//!      RESULT: We are done rendering one of the annotation group to a source
//!      snippet.
//!
//!   Each step is represented by a module below. In general, modules are
//!   monotinic: they only depent on previous steps.
//

// Modules
mod s1_grouping;
mod s2_split_replace;
mod s3_build_view;
mod s4_layout;
// mod s5_placing;
// mod s6_rendering;
#[cfg(feature = "sources")]
pub mod renderers;

// Imports
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FResult};

pub use s3_build_view::View;
pub use s4_layout::Layout;

use super::diag::Diag;


/***** AUXILLARY *****/
/// The [`RenderedDiag`]-struct that carries the result of rendering a [`Diag`].
///
/// You rarely have to use this struct yourself; instead, call [`Renderer::render()`] on a renderer
/// to do it for you.
pub struct RenderedDiag;

// Ops
impl Display for RenderedDiag {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult { todo!() }
}





/***** LIBRARY *****/
/// Defines something that can render [`Diag`]s.
pub trait Renderer<E: Clone> {
    /// Types of errors emitted by this renderer.
    type Error: Error;


    /* User-implemented stuff */
    /// Defines how to map [`Loc`] source IDs to some human-friendly name the user understands.
    ///
    /// For example, this could be backed by a map of IDs to filenames.
    ///
    /// # Arguments
    /// - `id`: The ID to map.
    ///
    /// # Returns
    /// A name for the source with the given `id`.
    ///
    /// # Panics
    /// This function should panic if the `id` was not recognized by it.
    fn display_id(&self, id: u64) -> impl Display;

    /// Defines how to render a sequence of `E`lements.
    ///
    /// # Arguments
    /// - `seq`: The sequence of elements to render.
    ///
    /// # Returns
    /// Something implementing [`Display`] that will render the given sequence.
    fn display_seq(&self, seq: &[E]) -> impl Display;

    /// Layouting algorithm that does two things:
    /// 1. Load the appropriate parts of the source text; and
    /// 2. Renders a [`View`] into a [`Layout`] by making abstract areas concrete and renderable.
    ///
    /// # Arguments
    /// - `id`: The ID of the source to render.
    /// - `view`: The [`View`] who's annotation markings and range to render.
    ///
    /// # Returns
    /// A new [`Layout`] that has most abstractions removed and has the laid-out text with inline
    /// flags for where to annotate.
    ///
    /// # Errors
    /// Since this function potentially loads source text, we can expect it to fail with its own
    /// error.
    ///
    /// # Panics
    /// This function should panic if the `id` was not recognized by it.
    fn layout<'a>(&mut self, id: u64, view: View<'a, E>) -> Result<Layout<'a>, Self::Error>;


    /* Auto-implemented stuff */
    /// Renders a given [`Diag`] into a [`RenderedDiag`].
    ///
    /// That one implements [`Display`] to show the result.
    ///
    /// # Returns
    /// A [`RenderedDiag`] that implements [`Display`](std::fmt::Display).
    fn render(&mut self, diag: &Diag<E>) -> Result<RenderedDiag, Self::Error> {
        // (S1) Begin by grouping annotations
        let groups: HashMap<Option<u64>, Vec<s1_grouping::Annot<E>>> = s1_grouping::group_annots(&diag.annots);
        for (id, mut group) in groups.into_iter() {
            // (S2) Ensure that any replace that overlaps with another annotation is reframed as a message
            s2_split_replace::split_replace_annots(self, &mut group);
            // (S3) Turn that ~~frown upside down~~ I mean group into a view
            let view: View<E> = s3_build_view::build_view(group);
            // (S4) Render the abstract view to a layout
            let layout: Option<Layout> = id.map(|id| s4_layout::layout(self, id, view)).transpose()?;
        }

        todo!()
    }
}
