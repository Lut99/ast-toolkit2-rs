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
//!      RESULT: All [`Span`](ast_toolkit_span::Span)s in annotations are
//!      replaced with their `Range`s only.
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
//!   3. _Applying replaces:_
//!
//!      In the third step, we will apply the replaces by turning the main
//!      group span into a _virtual span_. This one consists of slices of the
//!      main span alternated with replaced values, allowing one to iterate
//!      over it as if it was one span. At the same time, it will update all
//!      annotation ranges with the ones in the virtual span, yielding _virtual
//!      ranges_.
//!
//!      RESULT: We stop caring about replaces.
//!
//!      RESULT: The main [`Span`](ast_toolkit_span::Span) becomes a
//!      `VirtualSpan`, and all `Range`s in annotations become `VirtualRange`s.
//!      No annotation will have a `replacement` anymore.
//!   4. _Layouters:_
//!
//!      In the fourth step, we apply a user-selected
//!      [`Layouter`] that, with a maximum line width in mind, will render the
//!      virtual span into a cellbuffer. It, albeit with help from the virtual
//!      span, is responsible for annotating the resulting cellbuffer with
//!      flags indicating when annotations start and stop.
//!
//!      RESULT: We unlock any visual information we need to place annotations.
//!
//!      RESULT: We stop caring about ranges, as the flags will signpost where
//!      to apply them instead.
//!
//!      RESULT: We obtain a populated [`CellBuffer`] that carries the visual
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

// Declare the modules
mod s1_grouping;
mod s2_split_replace;
mod s3_apply_replace;
mod s4_layouters;
mod s5_placing;
mod s6_rendering;

// Define public interfaces
pub use s3_apply_replace::VirtualSpan;
pub use s4_layouters::{AnnotFlag, Cell, CellBuffer, HexLayouter, Layouter, Line, Utf8Layouter};
