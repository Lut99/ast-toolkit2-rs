//  LAYOUT.rs
//    by Lut99
//
//  Description:
//!   Defines an output layout buffer.
//

use std::fmt::Display;

use ast_toolkit_span::{Span, Spannable};

use super::annotated::AnnotatedBuffer;
use crate::annotations::{Annotation, AnnotationInner, AnnotationInnerHighlight, AnnotationInnerSuggestion, Severity};


/***** AUXILLARY *****/
/// Defines a range of line/column pairs with start/stop.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Range {
    /// The start position (inclusive).
    pub start: Position,
    /// The end position (inclusive).
    pub end:   Position,
}
impl Range {
    /// Checks if this range overlaps with another.
    ///
    /// # Arguments
    /// - `other`: The other range to check for overlap with.
    ///
    /// # Returns
    /// True if they overlap, or false otherwise.
    pub fn overlaps_with(&self, other: &Self) -> bool {
        // Based on https://stackoverflow.com/a/3269471
        (self.start.line < other.end.line || self.start.col <= other.end.col) && (other.start.line < self.end.line || other.start.col <= self.end.col)
    }
}

/// Defines a line/column pair.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Position {
    /// The line number, zero indexed.
    pub line: usize,
    /// The column number, zero indexed.
    pub col:  usize,
}
impl Position {
    /// Constructor for the Position.
    ///
    /// # Arguments
    /// - `line`: The zero-indexed line number this position points to.
    /// - `col`: The zero-indexed column number this position points to.
    ///
    /// # Returns
    /// A Position pointing to `(line, col)`.
    #[inline]
    pub const fn new(line: usize, col: usize) -> Self { Self { line, col } }
}



/// Defines the colouring options for a [`Chunk`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Color {
    Severity(Severity),
    Suggestion,
    Plain,
}





/***** LIBRARY *****/
/// Individual cells in a line.
#[derive(Debug, Clone)]
pub struct Cell<I, E> {
    /// The original source location, if any, as a pair of source ID & position.
    pub(super) i:     Option<(I, usize)>,
    /// The value of the cell, if any.
    pub(super) value: Option<E>,
}

// Constructors
impl<I, E> Cell<I, E> {
    /// Constructor for a Cell with a value but not from source.
    ///
    /// # Arguments
    /// - `value`: The value to set.
    ///
    /// # Returns
    /// A new Cell.
    #[inline]
    pub const fn from_value(value: E) -> Self { Self { i: None, value: Some(value) } }

    /// Constructor for a Cell from source with a value.
    ///
    /// # Arguments
    /// - `id`: The source identifier of the source producing this value.
    /// - `i`: The original source position.
    /// - `value`: The value to set.
    ///
    /// # Returns
    /// A new Cell.
    #[inline]
    pub const fn from_source(id: I, i: usize, value: E) -> Self { Self { i: Some((id, i)), value: Some(value) } }
}



/// Defines an abstract annotation range to apply to a [`Line`].
#[derive(Debug, Clone)]
pub struct AnnotRange<E> {
    /// Any replaceing to do.
    pub(super) replacement: Option<Vec<E>>,
    /// Any message to display.
    pub(super) message: Option<String>,
    /// The color of the annotation.
    pub(super) color: Color,
    /// The range it concerns, as a pair of (start, end) pairs of (line, column). Note the end is
    /// INCLUSIVE!
    pub(super) range: Range,
}



/// Individual lines in the output.
#[derive(Debug, Clone)]
pub struct Line<I, E> {
    /// The line number
    pub(super) l:     Option<String>,
    /// A list of cells in this line.
    pub(super) cells: Vec<Cell<I, E>>,
}

// Constructors
impl<I, E> Line<I, E> {
    /// Constructs a new Line with capacity for at least the given number of cells.
    ///
    /// # Arguments
    /// - `capacity`: The minimum number of cells to reserve space for.
    ///
    /// # Returns
    /// A new Line that will be able to store at least `capacity` cells before reallocation
    /// is necessary.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self { Self { l: None, cells: Vec::with_capacity(capacity) } }
}

// Collection
impl<I, E> Line<I, E> {
    /// Set a line number for this line, already rendered.
    ///
    /// Note that any preceding spaces are added automatically later.
    ///
    /// # Arguments
    /// - `l`: The rendered line number to set.
    ///
    /// # Returns
    /// Self for chaining.
    #[inline]
    pub fn set_line_number(&mut self, l: impl Display) -> &mut Self {
        self.l = Some(l.to_string());
        self
    }

    /// Write a new cell to the buffer.
    ///
    /// # Arguments
    /// - `cell`: Some [`Cell`](-like) to write to the buffer.
    ///
    /// # Returns
    /// Self for chaining.
    #[inline]
    pub fn push(&mut self, cell: impl Into<Cell<I, E>>) -> &mut Self {
        self.cells.push(cell.into());
        self
    }
}

// Conversion
impl<'s, S: Spannable<'s>> From<Span<S>> for Line<S::SourceId, S::Elem>
where
    S::Elem: Clone,
{
    #[inline]
    fn from(value: Span<S>) -> Self {
        // Resolve the start index of the Span
        let source: &S = value.source();
        let source_len: usize = source.len();
        let start: usize = value.range().start_resolved(source_len).unwrap_or(0);

        // Add them
        let mut res: Self = Self::with_capacity(source_len);
        for (i, elem) in value.as_slice().iter().enumerate() {
            res.cells.push(Cell { i: Some((value.source_id(), start + i)), value: Some(elem.clone()) })
        }

        // Done
        res
    }
}



/// Defines the buffer in which we'll write the output source fragment.
#[derive(Debug, Clone)]
pub struct LayoutBuffer<I, E> {
    /// The lines of output we'll write to.
    pub(super) lines:  Vec<Line<I, E>>,
    /// A list of annotations to apply to this line together with the range of cells they concern.
    pub(super) annots: Vec<AnnotRange<E>>,
}

// Constructors
impl<I, E> LayoutBuffer<I, E> {
    /// Constructs a new LayoutBuffer with capacity for at least the given number of lines.
    ///
    /// # Arguments
    /// - `capacity`: The minimum number of lines to reserve space for.
    ///
    /// # Returns
    /// A new LayoutBuffer that will be able to store at least `capacity` lines before reallocation
    /// is necessary.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self { Self { lines: Vec::with_capacity(capacity), annots: Vec::new() } }
}

// Collection
impl<I, E> LayoutBuffer<I, E> {
    /// Write a new line to the buffer.
    ///
    /// # Arguments
    /// - `line`: Some [`Line`](-like) to write to the buffer.
    ///
    /// # Returns
    /// Self for chaining.
    #[inline]
    pub fn push(&mut self, line: impl Into<Line<I, E>>) -> &mut Self {
        self.lines.push(line.into());
        self
    }

    /// Pushes an [`AnnotRange`] to this buffer.
    ///
    /// This is to collect annotations before we render them.
    ///
    /// # Arguments
    /// - `replacement`: If [`Some`], should attempt to replace text in the source with the one
    ///   given one when visually rendering the line. This is only possible if there is no other
    ///   annotation covering this range.
    /// - `message`: Any message to add to the annotation.
    /// - `color`: The [`Color`] to apply when rendering it.
    /// - `range`: The range this annotation concerns. At this point, **this range must be in range
    ///   for this span!**.
    ///
    /// # Returns
    /// Self for chaining.
    #[inline]
    pub fn push_annot(&mut self, replacement: Option<Vec<E>>, message: Option<String>, color: Color, range: Range) -> &mut Self {
        self.annots.push(AnnotRange { replacement, message, color, range });
        self
    }
}

// Annotations
impl<I, E> LayoutBuffer<I, E> {
    /// Applies the given [`Annotation`] to the buffer's contents.
    ///
    /// Note that this function does nothing if the annotation is from a different source than the
    /// internal spans, or if it's out-of-range.
    ///
    /// # Arguments
    /// - An [`Annotation`] to apply to this buffer's source text.
    ///
    /// # Returns
    /// Self for chaining.
    pub fn annotate<'s, S>(&mut self, annot: Annotation<'s, S>) -> &mut Self
    where
        S: Spannable<'s, Elem = E>,
        S::SourceId: PartialEq<I>,
    {
        // Get the span bounds
        let source: &S = annot.span.source();
        let source_id: S::SourceId = annot.span.source_id();
        let source_len: usize = source.len();
        let start: usize = annot.span.range().start_resolved(source_len).unwrap_or(0);
        let end: usize = annot.span.range().end_resolved(source_len).unwrap_or(0);

        // Scan to find where the span applies
        let mut write_range: Option<Range> = None;
        for l in 0..self.lines.len() {
            // SAFETY: Possible because the loop ensures `l` is within range
            let line: &Line<_, _> = unsafe { self.lines.get_unchecked(l) };
            for c in 0..line.cells.len() {
                // SAFETY: Possible because the loop ensures `c` is within range
                let cell: &Cell<_, _> = unsafe { line.cells.get_unchecked(c) };
                if let Some((id, i)) = &cell.i {
                    // We ensure that this cell is from the annotation's range and within range
                    if source_id.eq(id) && start >= *i && *i < end {
                        // OK, update the writes
                        match &mut write_range {
                            Some(Range { end, .. }) => *end = Position::new(l, c),
                            None => write_range = Some(Range { start: Position::new(l, c), end: Position::new(l, c) }),
                        }
                    }
                }
            }
        }
        let write_range: Range = match write_range {
            Some(range) => range,
            // Not within range / other source!
            None => return self,
        };

        // Get some annotation particulars
        let (message, replacement, color): (Option<String>, Option<Vec<E>>, Color) = match annot.inner {
            AnnotationInner::Highlight(AnnotationInnerHighlight { message, severity }) => (message, None, Color::Severity(severity)),
            AnnotationInner::Suggestion(AnnotationInnerSuggestion { message, replacement }) => (message, Some(replacement), Color::Suggestion),
        };

        // Now mark this annotation as ready-to-write
        self.push_annot(replacement, message, color, write_range)
    }
}
// There's also an impl at `annotated.rs`, placed there to have this file be a bit coherent

// Iterators
impl<I, E> FromIterator<Line<I, E>> for LayoutBuffer<I, E> {
    #[inline]
    fn from_iter<T: IntoIterator<Item = Line<I, E>>>(iter: T) -> Self { Self { lines: Vec::from_iter(iter), annots: Vec::new() } }
}
