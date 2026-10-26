//  STEP 4: LAYING OUT.rs
//    by Lut99
//
//  Description:
//!   This module implements the fourth step of the layouting algorithm
//!   described in the [`layout`](super)-module.
//!
//!   Given a [`View`], it will turn that into a [`Layout`] that renders the
//!   abstract information in it to concretely rendered text with inline flags
//!   pointing to where stuff needs to be annotated.
//!   
//!   This also mean this is the step where we load the source text, and such,
//!   this step is fallible.
//

use std::borrow::Cow;

use unicode_segmentation::UnicodeSegmentation as _;

use super::super::Severity;
use super::Renderer;
use super::s3_build_view::View;


/***** DATA STRUCTURES *****/
/// A single element in a [`Line`].
pub enum Part<'a> {
    /// It's a piece of legible text.
    Text { text: String },
    /// It's a signal that the next part is annotated by one or more annotations.
    ///
    /// Note that the length of the annotation is in _cells_, i.e., graphemes.
    Annot { sev: Severity, msg: Option<Cow<'a, str>>, len: usize },
}

// Ops
impl<'a> Part<'a> {
    /// Returns the length of the part, in bytes.
    ///
    /// Note that it returns the length of the would-be rendered text (i.e., [`Part::Annot`]s have
    /// length 0).
    #[inline]
    pub fn len(&self) -> usize {
        match self {
            Self::Text { text } => text.len(),
            Self::Annot { .. } => 0,
        }
    }

    /// Returns the length of the part, in grapheme count.
    ///
    /// Note that it returns the length of the would-be rendered text (i.e., [`Part::Annot`]s have
    /// width 0).
    #[inline]
    pub fn width(&self) -> usize {
        match self {
            Self::Text { text } => text.graphemes(true).count(),
            Self::Annot { .. } => 0,
        }
    }
}



/// A buffer for storing a single line of concretely rendered [`View`]s.
pub struct Line<'a> {
    /// The parts in this line.
    pub parts: Vec<Part<'a>>,
}

// Constructors
impl<'a> Default for Line<'a> {
    #[inline]
    fn default() -> Self { Self::new() }
}
impl<'a> Line<'a> {
    /// Constructor for an empty line.
    ///
    /// # Returns
    /// A new Line that can store rendered text.
    #[inline]
    pub const fn new() -> Self { Self { parts: Vec::new() } }
}



/// A buffer for storing concretely rendered [`View`]s, [`Line`]-by-line.
pub struct Layout<'a> {
    /// A buffer storing all lines.
    pub lines: Vec<Line<'a>>,
}

// Constructors
impl<'a> Default for Layout<'a> {
    #[inline]
    fn default() -> Self { Self::new() }
}
impl<'a> Layout<'a> {
    /// Constructor for an empty layout.
    ///
    /// # Returns
    /// A new Layout that can store rendered text.
    #[inline]
    pub const fn new() -> Self { Self { lines: Vec::new() } }
}





/***** LIBRARY *****/
/// Given a set of annotations, it will organise them into a [`View`].
///
/// This implements the third step of the layouting algorithm detailled in [`layout`](super).
///
/// # Arguments
/// - `renderer: A [`Renderer`] that will do the interesting part of the layouting.
/// - `id`: A source ID pointing to the source the `renderer` should use as context.
/// - `view`: The [`View`] containing information on what to replace and annotate.
///
/// # Returns
/// The [`View`] that starts defining an abstract overview of the resulting snippet.
#[inline]
pub fn layout<'a, R: ?Sized + Renderer<E>, E: Clone>(renderer: &mut R, id: u64, view: View<'a, E>) -> Result<Layout<'a>, R::Error> {
    // I don't think this really has to do anything? Just let the renderer do the work
    renderer.layout(id, view)
}
