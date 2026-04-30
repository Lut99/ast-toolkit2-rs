//  STEP 4: LAYOUTERS.rs
//    by Lut99
//
//  Description:
//!   This module implements the fourth step of the layouting algorithm
//!   described in the [`layout`](super)-module.
//!
//!   Given a [`VirtualSpan`], the [`Layouter`] trait allows different ways for
//!   rendering different `S`ource texts to a [`CellBuffer`]. By default, we
//!   provide implementations for [`SpannableBytes`] `S`ources, who either
//!   render as UTF-8 text or as hex viewers.
//

use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::ops::{Index, IndexMut, RangeBounds};
use std::vec::Drain;

use ast_toolkit_span::{Span, Spannable, SpannableBytes};

use super::s3_apply_replace::VirtualSpan;
use crate::annotations::Severity;


/***** HELPER FUNCTIONS *****/
/// Finds the lines that are touched by the given Span.
///
/// # Arguments
/// - `span`: The [`Span`] to find the lines it touches of.
///
/// # Returns
/// A tuple with the line number of the first line (as a one-indexed number), and a list of
/// [`Span`]s, each of which spanning a line touched by the given `Span`. Note that the line number
/// is [`None`] if the span not spanning anything from the `S`ource.
fn find_touched_lines<'s, S: Clone + SpannableBytes<'s>>(span: Span<S>) -> (Option<usize>, Vec<Span<S>>) {
    let source: &S = span.source();
    let source_len: usize = source.len();
    let start: usize = span.range().start_resolved(source_len).unwrap_or(0); // Inclusive
    let end: usize = span.range().end_resolved(source_len).unwrap_or(0); // Exclusive

    // Go thru the _source_, not the span, to find the surrounding lines
    let mut l: usize = 1;
    let mut i: usize = 0;
    let mut line_start: usize = 0; // Inclusive
    let mut lines: Vec<Span<S>> = Vec::with_capacity(1);
    for b in source.as_bytes() {
        // We initially match on newlines
        if *b == b'\n' {
            // It's the last element of the current line

            // Store the line
            if i >= start {
                lines.push(Span::ranged(source.clone(), line_start..i + 1));
            } else {
                l += 1;
            }

            // Update the new line start
            line_start = i + 1;

            // Potentially quit
            if i + 1 >= end {
                break;
            }
        }

        // Now update our position according to the element and keep iterating
        i += 1;
    }
    if i + 1 < end {
        // There's still range left; get the remaining
        lines.push(Span::ranged(source.clone(), line_start..));
    }
    (if !lines.is_empty() { Some(l) } else { None }, lines)
}

/// Finds the chunks that are touched by the given Span.
///
/// # Arguments
/// - `chunk_len`: The size of every chunk.
/// - `span`: The [`Span`] to find the chunks it touches.
///
/// # Returns
/// A list of [`Span`]s, each of which spanning a chunk touched by the given `Span`.
fn find_touched_chunks<'s, S: Clone + SpannableBytes<'s>>(chunk_len: usize, span: Span<S>) -> Vec<Span<S>> {
    let source: &S = span.source();
    let source_len: usize = source.len();
    let mut start: usize = span.range().start_resolved(source_len).unwrap_or(0); // Inclusive
    let mut end: usize = span.range().end_resolved(source_len).unwrap_or(0); // Exclusive

    // Now extend start- and end to touch chunk boundaries.
    start -= start % chunk_len;
    end += if end % chunk_len > 0 { chunk_len - (end % chunk_len) } else { 0 };

    // Now slice
    let mut res = Vec::with_capacity((end - start) / chunk_len);
    let mut i: usize = start;
    while i < end {
        res.push(Span::ranged(source.clone(), i..i + chunk_len));
        i += chunk_len;
    }
    res
}





/***** DATA STRUCTURES *****/
/// Defines so-called _annotation flags_ that represents start- and stop flags for a ranged area.
#[derive(Clone, Debug)]
pub enum AnnotFlag {
    /// Start of an annotation, with anything we need to render it.
    Start {
        /// Some identifier to recognize the matching stop flag.
        id: usize,
        /// Any message to show.
        message: Option<String>,
        /// The severity of this annotation.
        severity: Option<Severity>,
    },
    /// Stop flag of an annotation with the given ID.
    Stop {
        /// Some identifier to recognize the matching start flag.
        id: usize,
    },
}

// Constructors
impl AnnotFlag {
    /// Constructor for an AnnotFlag that initializes it as a barebones
    /// [`Start`](AnnotFlag::Start)-flag.
    ///
    /// # Arguments
    /// - `id`: The identifier of the annotation started by this flag.
    ///
    /// # Returns
    /// A new [`AnnotFlag::Start`] with empty message and empty severity.
    #[inline]
    pub const fn new_start(id: usize) -> Self { Self::Start { id, message: None, severity: None } }

    /// Constructor for an AnnotFlag that initializes it as a barebones
    /// [`Stop`](AnnotFlag::Stop)-flag.
    ///
    /// # Arguments
    /// - `id`: The identifier of the annotation stopped by this flag.
    ///
    /// # Returns
    /// A new [`AnnotFlag::Stop`].
    #[inline]
    pub const fn new_stop(id: usize) -> Self { Self::Stop { id } }
}

// Flag
impl AnnotFlag {
    /// Returns the identifier in this flag.
    ///
    /// # Returns
    /// A [`usize`] representing a unique identifier.
    #[inline]
    pub fn id(&self) -> usize {
        match self {
            Self::Start { id, .. } => *id,
            Self::Stop { id } => *id,
        }
    }
}

// Ops
impl Eq for AnnotFlag {}
impl Hash for AnnotFlag {
    /// The [`AnnotFlag`] only [compares](AnnotFlag::eq()) and hashes using its variant and its
    /// [identifier](AnnotFlag::id()). This to make sure that one can overwrite flags for the same
    /// annotation when e.g. the message changes, but not accidentally change it into another flag.
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash the variant and the ID, and that's it
        match self {
            Self::Start { id, message: _, severity: _ } => {
                0.hash(state);
                id.hash(state);
            },
            Self::Stop { id } => {
                1.hash(state);
                id.hash(state);
            },
        }
    }
}
impl PartialEq for AnnotFlag {
    /// The [`AnnotFlag`] only compares and [hashes](AnnotFlag::hash()) using its variant and its
    /// [identifier](AnnotFlag::id()). This to make sure that one can overwrite flags for the same
    /// annotation when e.g. the message changes, but not accidentally change it into another flag.
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Start { id: lid, message: _, severity: _ }, Self::Start { id: rid, message: _, severity: _ }) => lid == rid,
            (Self::Stop { id: lid }, Self::Stop { id: rid }) => lid == rid,
            _ => false,
        }
    }
}



/// Individual cells in a [`Line`].
#[derive(Clone, Debug)]
pub struct Cell<E> {
    /// The value of the cell, if any.
    value: Option<E>,
    /// The color of the cell, as dictated by the severity.
    color: Option<Severity>,
    /// Any [`AnnotFlag`]s attached to this cell.
    flags: HashSet<AnnotFlag>,
}

// Constructors
impl<E> Default for Cell<E> {
    #[inline]
    fn default() -> Self { Self::new() }
}
impl<E> Cell<E> {
    /// Constructor for a Cell without value or color.
    ///
    /// # Returns
    /// A new Cell.
    #[inline]
    pub fn new() -> Self { Self { value: None, color: None, flags: HashSet::new() } }

    /// Constructor for a Cell with a value.
    ///
    /// # Arguments
    /// - `value`: The value to set.
    ///
    /// # Returns
    /// A new Cell.
    #[inline]
    pub fn from_value(value: E) -> Self { Self { value: Some(value), color: None, flags: HashSet::new() } }
}

// Cell
impl<E> Cell<E> {
    /// Sets the color in this cell.
    ///
    /// # Arguments
    /// - `severity`: Some [`Severity`] to color this cell with.
    ///
    /// # Returns
    /// Self for chaining.
    #[inline]
    pub const fn set_color(&mut self, severity: Severity) -> &mut Self {
        self.color = Some(severity);
        self
    }

    /// Removes the color from this cell.
    ///
    /// # Returns
    /// Self for chaining.
    #[inline]
    pub const fn clear_color(&mut self) -> &mut Self {
        self.color = None;
        self
    }

    /// Removes the value from this cell.
    ///
    /// If there's any color, that's also cleared.
    ///
    /// # Returns
    /// Self for chaining.
    #[inline]
    pub fn clear(&mut self) -> &mut Self {
        self.value = None;
        self.color = None;
        self
    }



    /// Adds an annotation flag to this cell.
    ///
    /// Note that an [`AnnotFlag`]'s equality is defined by its variant (i.e.,
    /// [`Start`](AnnotFlag::Start) or [`Stop`](AnnotFlag::Stop)) and its
    /// [identifier](AnnotFlag::id()) _only._ This means that giving the same flag with only a =
    /// different message will overwrite the first of the two. This is intended behaviour.
    ///
    /// # Arguments
    /// - `flag`: Some [`AnnotFlag`] to add to this cell.
    #[inline]
    pub fn add_flag(&mut self, flag: AnnotFlag) { self.flags.insert(flag); }

    /// Removes a flag from this cell.
    ///
    /// Since an [`AnnotFlag`]'s equality is defined by its variant (i.e.,
    /// [`Start`](AnnotFlag::Start) or [`Stop`](AnnotFlag::Stop)) and its
    /// [identifier](AnnotFlag::id()) _only,_ you can simply call [`AnnotFlag::new_start()`] or
    /// [`AnnotFlag::new_stop()`] with the appropriate ID to find the matching flag.
    ///
    /// # Arguments
    /// - `flag`: A [`AnnotFlag`] to remove from this cell.
    #[inline]
    pub fn remove_flag(&mut self, flag: &AnnotFlag) { self.flags.remove(&flag); }
}



/// Individual lines in a [`CellBuffer`].
#[derive(Debug, Clone)]
pub struct Line<E> {
    /// The line number, if any.
    l:     Option<String>,
    /// A list of cells in this line.
    cells: Vec<Cell<E>>,
}

// Constructors
impl<E> Line<E> {
    /// Constructor for a Line filled with [empty](Cell::new()) [`Cell`]s.
    ///
    /// # Arguments
    /// - `width`: The width of this line.
    ///
    /// # Returns
    /// A new [`Line`] that doesn't hold any data yet.
    #[inline]
    pub fn new(width: usize) -> Self {
        Self {
            l:     None,
            cells: {
                let mut cells = Vec::with_capacity(width);
                for _ in 0..width {
                    cells.push(Cell::new())
                }
                cells
            },
        }
    }
}

// Collection
impl<E> Line<E> {
    /// Gets the [`Cell`] at the given column position.
    ///
    /// This function takes columns position by zero-indexed value. See [`Line::get1()`] to get it
    /// by one-indexed value instead.
    ///
    /// # Arguments
    /// - `index`: A zero-indexed column position to get the cell at.
    ///
    /// # Returns
    /// A reference to the [`Cell`] or [`None`] if `index` was larger than [`Self::width()`].
    #[inline]
    pub fn get0(&self, index: usize) -> Option<&Cell<E>> { self.cells.get(index) }

    /// Gets the [`Cell`] at the given column position without checking if it's in bounds.
    ///
    /// It's YOUR responsibility to ensure that `index` is within bounds of [`Self::width()`].
    /// If it isn't, undefined behaviour looms.
    ///
    /// This function takes columns position by zero-indexed value. See [`Line::get1_unchecked()`]
    /// to get it by one-indexed value instead.
    ///
    /// # Arguments
    /// - `index`: A zero-indexed column position to get the cell at.
    ///
    /// # Returns
    /// A reference to the [`Cell`].
    #[inline]
    pub unsafe fn get0_unchecked(&self, index: usize) -> &Cell<E> { self.cells.get_unchecked(index) }

    /// Gets the [`Cell`] at the given column position, mutably.
    ///
    /// This function takes columns position by zero-indexed value. See [`Line::get1_mut()`] to get
    /// it by one-indexed value instead.
    ///
    /// # Arguments
    /// - `index`: A zero-indexed column position to get the cell at.
    ///
    /// # Returns
    /// A mutable reference to the [`Cell`] or [`None`] if `index` was larger than
    /// [`Self::width()`].
    #[inline]
    pub fn get0_mut(&mut self, index: usize) -> Option<&mut Cell<E>> { self.cells.get_mut(index) }

    /// Gets the [`Cell`] at the given column position without checking if it's in bounds, mutably.
    ///
    /// It's YOUR responsibility to ensure that `index` is within bounds of [`Self::width()`].
    /// If it isn't, undefined behaviour looms.
    ///
    /// This function takes columns position by zero-indexed value. See
    /// [`Line::get1_unchecked_mut()`] to get it by one-indexed value instead.
    ///
    /// # Arguments
    /// - `index`: A zero-indexed column position to get the cell at.
    ///
    /// # Returns
    /// A mutable reference to the [`Cell`].
    #[inline]
    pub unsafe fn get0_unchecked_mut(&mut self, index: usize) -> &mut Cell<E> { self.cells.get_unchecked_mut(index) }



    /// Gets the [`Cell`] at the given column position.
    ///
    /// This function takes columns position by one-indexed value. See [`Line::get0()`] to get it
    /// by zero-indexed value instead.
    ///
    /// # Arguments
    /// - `index`: A one-indexed column position to get the cell at.
    ///
    /// # Returns
    /// A reference to the [`Cell`] or [`None`] if `index` was larger than [`Self::width()`].
    ///
    /// # Panics
    /// This function panics if `index` was 0.
    #[inline]
    #[track_caller]
    pub fn get1(&self, index: impl TryInto<NonZeroUsize>) -> Option<&Cell<E>> {
        let index: NonZeroUsize = index.try_into().unwrap_or_else(|_| panic!("Got an index of 0 for a one-indexed function"));
        let index: usize = index.into();
        self.cells.get(index - 1)
    }

    /// Gets the [`Cell`] at the given column position without checking if it's in bounds.
    ///
    /// It's YOUR responsibility to ensure that `index` is larger than 0 AND within bounds of
    /// [`Self::width()`]. If either guarantee isn't met, undefined behaviour looms.
    ///
    /// This function takes columns position by one-indexed value. See [`Line::get0_unchecked()`]
    /// to get it by zero-indexed value instead.
    ///
    /// # Arguments
    /// - `index`: A one-indexed column position to get the cell at.
    ///
    /// # Returns
    /// A reference to the [`Cell`].
    #[inline]
    pub unsafe fn get1_unchecked(&self, index: impl Into<usize>) -> &Cell<E> {
        let index: usize = index.into();
        self.cells.get_unchecked(index - 1)
    }

    /// Gets the [`Cell`] at the given column position, mutably.
    ///
    /// This function takes columns position by one-indexed value. See [`Line::get0_mut()`] to get
    /// it by zero-indexed value instead.
    ///
    /// # Arguments
    /// - `index`: A one-indexed column position to get the cell at.
    ///
    /// # Returns
    /// A mutable reference to the [`Cell`] or [`None`] if `index` was larger than
    /// [`Self::width()`].
    ///
    /// # Panics
    /// This function panics if `index` was 0.
    #[inline]
    #[track_caller]
    pub fn get1_mut(&mut self, index: impl TryInto<NonZeroUsize>) -> Option<&mut Cell<E>> {
        let index: NonZeroUsize = index.try_into().unwrap_or_else(|_| panic!("Got an index of 0 for a one-indexed function"));
        let index: usize = index.into();
        self.cells.get_mut(index - 1)
    }

    /// Gets the [`Cell`] at the given column position without checking if it's in bounds, mutably.
    ///
    /// It's YOUR responsibility to ensure that `index` is larger than 0 AND within bounds of
    /// [`Self::width()`]. If either guarantee isn't met, undefined behaviour looms.
    ///
    /// This function takes columns position by one-indexed value. See
    /// [`Line::get0_unchecked_mut()`] to get it by zero-indexed value instead.
    ///
    /// # Arguments
    /// - `index`: A one-indexed column position to get the cell at.
    ///
    /// # Returns
    /// A mutable reference to the [`Cell`].
    #[inline]
    pub unsafe fn get1_unchecked_mut(&mut self, index: impl Into<usize>) -> &mut Cell<E> {
        let index: usize = index.into();
        self.cells.get_unchecked_mut(index - 1)
    }



    /// Gets the number of [`Cell`]s in this Line.
    ///
    /// # Returns
    /// A number that, when equal to or exceeded when giving an index, will cause problems.
    #[inline]
    pub const fn width(&self) -> usize { self.cells.len() }
}

// Line
impl<E> Line<E> {
    /// Set a line number for this line.
    ///
    /// The line number should be serializable through a [`ToString`]-implementation. Anything
    /// implementing [`Display`](std::fmt::Display) does so.
    ///
    /// # Arguments
    /// - `l`: The (rendered) line number to set.
    ///
    /// # Returns
    /// Self for chaining.
    #[inline]
    pub fn set_line_number(&mut self, l: impl ToString) -> &mut Self {
        self.l = Some(l.to_string());
        self
    }

    #[inline]
    /// Removes a line number for this line if any.
    ///
    /// # Returns
    /// Self for chaining.
    pub fn clear_line_number(&mut self) -> &mut Self {
        self.l = None;
        self
    }
}

// Ops
impl<E> Index<usize> for Line<E> {
    type Output = Cell<E>;

    #[inline]
    #[track_caller]
    fn index(&self, index: usize) -> &Self::Output {
        self.get0(index).unwrap_or_else(|| panic!("Index {index} is out-of-bounds for Line of width {}", self.cells.len()))
    }
}
impl<E> IndexMut<usize> for Line<E> {
    #[inline]
    #[track_caller]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let cells_len: usize = self.cells.len();
        self.get0_mut(index).unwrap_or_else(|| panic!("Index {index} is out-of-bounds for Line of width {cells_len}"))
    }
}



/// Defines the buffer for holding [`Line`]s and [`Cell`]s of output characters.
#[derive(Clone, Debug)]
pub struct CellBuffer<E> {
    /// The lines of output we'll write to.
    lines: Vec<Line<E>>,
    /// The width of lines in this buffer.
    width: usize,
}

// Constructors
impl<E> CellBuffer<E> {
    /// Constructs a new, empty CellBuffer.
    ///
    /// # Arguments
    /// - `width`: The width of the lines in the CellBuffer.
    ///
    /// # Returns
    /// A new, empty CellBuffer.
    #[inline]
    pub const fn new(width: usize) -> Self { Self { lines: Vec::new(), width } }

    /// Constructs a new, empty CellBuffer with capacity for at least the given number of lines.
    ///
    /// # Arguments
    /// - `width`: The width of the lines in the CellBuffer.
    /// - `capacity`: The minimum number of lines to reserve space for.
    ///
    /// # Returns
    /// A new, empty CellBuffer that will be able to store at least `capacity` [`Line`]s before
    /// reallocation is necessary.
    #[inline]
    pub fn with_capacity(width: usize, capacity: usize) -> Self { Self { lines: Vec::with_capacity(capacity), width } }
}

// Collection
impl<E> CellBuffer<E> {
    /// Adds a new line to the end of the CellBuffer.
    ///
    /// # Arguments
    /// - `line`: Some [`Line`](-like) to write to the buffer.
    ///
    /// # Returns
    /// Self for chaining.
    ///
    /// # Panics
    /// This function panics if `line` does not have a [`Line::width()`] of exactly
    /// [`CellBuffer::width()`].
    #[inline]
    #[track_caller]
    pub fn push(&mut self, line: impl Into<Line<E>>) -> &mut Self {
        let line: Line<E> = line.into();
        if line.width() != self.width() {
            panic!("Attempted to push line of width {} to CellBuffer of width {}", line.width(), self.width())
        }
        self.lines.push(line);
        self
    }

    /// Adds a new line to the end of the CellBuffer.
    ///
    /// This overload requires you to make sure that the [`Line::width()`] of the given `line` is
    /// exactly equal to the [`CellBuffer::width()`], or else undefined behaviour may loom.
    ///
    /// # Arguments
    /// - `line`: Some [`Line`](-like) to write to the buffer.
    ///
    /// # Returns
    /// Self for chaining.
    #[inline]
    pub unsafe fn push_unchecked(&mut self, line: impl Into<Line<E>>) -> &mut Self {
        self.lines.push(line.into());
        self
    }

    /// Adds a bunch of lines to the end of the CellBuffer.
    ///
    /// # Arguments
    /// - `lines`: Some [`Iterator`]-to-be yielding [`Line`]s to write to the buffer. Its
    ///   [`Iterator::size_hint()`] implementation is used to optimize allocations.
    ///
    /// # Returns
    /// Self for chaining.
    ///
    /// # Panics
    /// This function panics if any line in `lines` does not have a [`Line::width()`] of exactly
    /// [`CellBuffer::width()`].
    #[track_caller]
    pub fn extend(&mut self, lines: impl IntoIterator<Item = Line<E>>) -> &mut Self {
        let iter = lines.into_iter();
        let size_hint: (usize, Option<usize>) = iter.size_hint();
        self.lines.reserve(size_hint.1.unwrap_or(size_hint.0));

        // Now push all the lines
        for line in iter {
            if line.width() != self.width() {
                panic!("Attempted to push line of width {} to CellBuffer of width {}", line.width(), self.width())
            }
            self.lines.push(line);
        }

        // Done
        self
    }

    /// Adds a bunch of lines to the end of the CellBuffer.
    ///
    /// This overload requires you to make sure that the [`Line::width()`] of all given `line`s are
    /// exactly equal to the [`CellBuffer::width()`], or else undefined behaviour may loom.
    ///
    /// # Arguments
    /// - `lines`: Some [`Iterator`]-to-be yielding [`Line`]s to write to the buffer. Its
    ///   [`Iterator::size_hint()`] implementation is used to optimize allocations.
    ///
    /// # Returns
    /// Self for chaining.
    pub unsafe fn extend_unchecked(&mut self, lines: impl IntoIterator<Item = Line<E>>) -> &mut Self {
        let iter = lines.into_iter();
        let size_hint: (usize, Option<usize>) = iter.size_hint();
        self.lines.reserve(size_hint.1.unwrap_or(size_hint.0));

        // Now push all the lines
        for line in iter {
            self.lines.push(line);
        }

        // Done
        self
    }

    /// Pops the last [`Line`] off this CellBuffer.
    ///
    /// # Returns
    /// The last [`Line`] in the buffer, or [`None`] if we were empty.
    #[inline]
    pub fn pop(&mut self) -> Option<Line<E>> { self.lines.pop() }

    /// Removes a [`Line`] from this CellBuffer.
    ///
    /// This overload preserves line order. It also uses a zero-indexed `index`. If you're using
    /// one-indexing instead, use [`CellBuffer::remove1()`].
    ///
    /// # Arguments
    /// - `index`: Some zero-indexed index pointing to the line to remove from this buffer.
    ///
    /// # Returns
    /// The remove [`Line`].
    ///
    /// # Panics
    /// This function panics if `index` was out-of-bounds for this buffer.
    #[inline]
    #[track_caller]
    pub fn remove0(&mut self, index: usize) -> Line<E> { self.lines.remove(index) }

    /// Removes a [`Line`] from this CellBuffer.
    ///
    /// This overload preserves line order. It also uses a one-indexed `index`. If you're using
    /// zero-indexing instead, use [`CellBuffer::remove0()`].
    ///
    /// # Arguments
    /// - `index`: Some zero-indexed index pointing to the line to remove from this buffer.
    ///
    /// # Returns
    /// The remove [`Line`].
    ///
    /// # Panics
    /// This function panics if `index` was 0 or out-of-bounds for this buffer.
    #[inline]
    #[track_caller]
    pub fn remove1(&mut self, index: impl TryInto<NonZeroUsize>) -> Line<E> {
        let index: NonZeroUsize = index.try_into().unwrap_or_else(|_| panic!("Got an index of 0 for a one-indexed function"));
        let index: usize = index.into();
        self.lines.remove(index - 1)
    }

    /// Removes a whole slice of elements from this CellBuffer.
    ///
    /// # Arguments
    /// - `range`: Some (zero-indexed) [`RangeBounds`] describing the range to remove.
    ///
    /// # Returns
    /// A [`Drain`] iterator yielding the removed elements.
    ///
    /// # Panics
    /// If the start point of `range` is larger than the end point, or if the end point is larger
    /// than [`CellBuffer::len()`].
    #[inline]
    #[track_caller]
    pub fn drain(&mut self, range: impl RangeBounds<usize>) -> Drain<Line<E>> { self.lines.drain(range) }



    /// Gets the [`Line`] at the given line position.
    ///
    /// This function takes lines position by zero-indexed value. See [`CellBuffer::get1()`] to get
    /// it by one-indexed value instead.
    ///
    /// # Arguments
    /// - `index`: A zero-indexed line position to get the cell at.
    ///
    /// # Returns
    /// A reference to the [`Line`] or [`None`] if `index` was larger than [`Self::width()`].
    #[inline]
    pub fn get0(&self, index: usize) -> Option<&Line<E>> { self.lines.get(index) }

    /// Gets the [`Line`] at the given line position without checking if it's in bounds.
    ///
    /// It's YOUR responsibility to ensure that `index` is within bounds of [`Self::width()`].
    /// If it isn't, undefined behaviour looms.
    ///
    /// This function takes lines position by zero-indexed value. See
    /// [`CellBuffer::get1_unchecked()`] to get it by one-indexed value instead.
    ///
    /// # Arguments
    /// - `index`: A zero-indexed line position to get the cell at.
    ///
    /// # Returns
    /// A reference to the [`Line`].
    #[inline]
    pub unsafe fn get0_unchecked(&self, index: usize) -> &Line<E> { self.lines.get_unchecked(index) }

    /// Gets the [`Line`] at the given line position, mutably.
    ///
    /// This function takes lines position by zero-indexed value. See [`CellBuffer::get1_mut()`] to
    /// get it by one-indexed value instead.
    ///
    /// # Arguments
    /// - `index`: A zero-indexed line position to get the cell at.
    ///
    /// # Returns
    /// A mutable reference to the [`Line`] or [`None`] if `index` was larger than
    /// [`Self::width()`].
    #[inline]
    pub fn get0_mut(&mut self, index: usize) -> Option<&mut Line<E>> { self.lines.get_mut(index) }

    /// Gets the [`Line`] at the given line position without checking if it's in bounds, mutably.
    ///
    /// It's YOUR responsibility to ensure that `index` is within bounds of [`Self::width()`].
    /// If it isn't, undefined behaviour looms.
    ///
    /// This function takes lines position by zero-indexed value. See
    /// [`CellBuffer::get1_unchecked_mut()`] to get it by one-indexed value instead.
    ///
    /// # Arguments
    /// - `index`: A zero-indexed line position to get the cell at.
    ///
    /// # Returns
    /// A mutable reference to the [`Line`].
    #[inline]
    pub unsafe fn get0_unchecked_mut(&mut self, index: usize) -> &mut Line<E> { self.lines.get_unchecked_mut(index) }



    /// Gets the [`Line`] at the given line position.
    ///
    /// This function takes lines position by one-indexed value. See [`CellBuffer::get0()`] to get
    /// it by zero-indexed value instead.
    ///
    /// # Arguments
    /// - `index`: A one-indexed line position to get the cell at.
    ///
    /// # Returns
    /// A reference to the [`Line`] or [`None`] if `index` was larger than [`Self::width()`].
    ///
    /// # Panics
    /// This function panics if `index` was 0.
    #[inline]
    #[track_caller]
    pub fn get1(&self, index: impl TryInto<NonZeroUsize>) -> Option<&Line<E>> {
        let index: NonZeroUsize = index.try_into().unwrap_or_else(|_| panic!("Got an index of 0 for a one-indexed function"));
        let index: usize = index.into();
        self.lines.get(index - 1)
    }

    /// Gets the [`Line`] at the given line position without checking if it's in bounds.
    ///
    /// It's YOUR responsibility to ensure that `index` is larger than 0 AND within bounds of
    /// [`Self::width()`]. If either guarantee isn't met, undefined behaviour looms.
    ///
    /// This function takes lines position by one-indexed value. See
    /// [`CellBuffer::get0_unchecked()`] to get it by zero-indexed value instead.
    ///
    /// # Arguments
    /// - `index`: A one-indexed line position to get the cell at.
    ///
    /// # Returns
    /// A reference to the [`Line`].
    #[inline]
    pub unsafe fn get1_unchecked(&self, index: impl Into<usize>) -> &Line<E> {
        let index: usize = index.into();
        self.lines.get_unchecked(index - 1)
    }

    /// Gets the [`Line`] at the given line position, mutably.
    ///
    /// This function takes lines position by one-indexed value. See [`CellBuffer::get0_mut()`] to
    /// get it by zero-indexed value instead.
    ///
    /// # Arguments
    /// - `index`: A one-indexed line position to get the cell at.
    ///
    /// # Returns
    /// A mutable reference to the [`Line`] or [`None`] if `index` was larger than
    /// [`Self::width()`].
    ///
    /// # Panics
    /// This function panics if `index` was 0.
    #[inline]
    #[track_caller]
    pub fn get1_mut(&mut self, index: impl TryInto<NonZeroUsize>) -> Option<&mut Line<E>> {
        let index: NonZeroUsize = index.try_into().unwrap_or_else(|_| panic!("Got an index of 0 for a one-indexed function"));
        let index: usize = index.into();
        self.lines.get_mut(index - 1)
    }

    /// Gets the [`Line`] at the given line position without checking if it's in bounds, mutably.
    ///
    /// It's YOUR responsibility to ensure that `index` is larger than 0 AND within bounds of
    /// [`Self::width()`]. If either guarantee isn't met, undefined behaviour looms.
    ///
    /// This function takes lines position by one-indexed value. See
    /// [`CellBuffer::get0_unchecked_mut()`] to get it by zero-indexed value instead.
    ///
    /// # Arguments
    /// - `index`: A one-indexed line position to get the cell at.
    ///
    /// # Returns
    /// A mutable reference to the [`Line`].
    #[inline]
    pub unsafe fn get1_unchecked_mut(&mut self, index: impl Into<usize>) -> &mut Line<E> {
        let index: usize = index.into();
        self.lines.get_unchecked_mut(index - 1)
    }



    /// Reserves space for additional [`Line`]s.
    ///
    /// This has the same guarantees as [`Vec::reserve()`]: in particular, that if the internal
    /// storage already has enough capacity, it doesn't do anything.
    ///
    /// # Arguments
    /// - `additional`: The minimum number of [`Line`]s for which the CellBuffer should
    ///   have space after this function returns.
    #[inline]
    pub fn reserve(&mut self, additional: usize) -> &mut Self {
        self.lines.reserve(additional);
        self
    }



    /// Returns the capacity of this CellBuffer.
    ///
    /// # Returns
    /// A [`usize`] encoding how many [`Line`]s this buffer can store before it needs a re-
    /// allocation.
    #[inline]
    pub const fn capacity(&self) -> usize { self.lines.capacity() }

    /// Returns the width of EVERY line in this CellBuffer.
    ///
    /// # Returns
    /// A [`usize`] encoding the number of [`Cell`]s in every line in this CellBuffer.
    #[inline]
    pub const fn width(&self) -> usize { self.width }

    /// Returns the number of lines currently in the CellBuffer.
    ///
    /// # Returns
    /// A [`usize`] encoding the number of elements within.
    #[inline]
    pub const fn len(&self) -> usize { self.lines.len() }

    /// Returns true if there are no lines in the CellBuffer.
    ///
    /// # Returns
    /// True if [`CellBuffer::len() == 0`](CellBuffer::len()).
    #[inline]
    pub const fn is_empty(&self) -> bool { self.lines.is_empty() }
}

// Iterators
impl<E> CellBuffer<E> {
    /// Returns an iterator over the lines in this CellBuffer.
    ///
    /// # Returns
    /// An [`Iter`](std::slice::Iter) over the internal elements, yielding them by reference.
    #[inline]
    pub fn iter(&self) -> std::slice::Iter<Line<E>> { self.lines.iter() }

    /// Returns a mutable iterator over the lines in this CellBuffer.
    ///
    /// # Returns
    /// An [`Iter`](std::slice::Iter) over the internal elements, yielding them by mutable
    /// reference.
    #[inline]
    pub fn iter_mut(&mut self) -> std::slice::IterMut<Line<E>> { self.lines.iter_mut() }
}
impl<'b, E> IntoIterator for &'b CellBuffer<E> {
    type Item = &'b Line<E>;
    type IntoIter = std::slice::Iter<'b, Line<E>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.iter() }
}
impl<'b, E> IntoIterator for &'b mut CellBuffer<E> {
    type Item = &'b mut Line<E>;
    type IntoIter = std::slice::IterMut<'b, Line<E>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.iter_mut() }
}
impl<E> IntoIterator for CellBuffer<E> {
    type Item = Line<E>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.lines.into_iter() }
}

// Conversion
impl<E> FromIterator<Line<E>> for CellBuffer<E> {
    #[inline]
    #[track_caller]
    fn from_iter<T: IntoIterator<Item = Line<E>>>(iter: T) -> Self {
        let lines: Vec<Line<E>> = Vec::from_iter(iter);
        if let Some(first) = lines.first() {
            for line in &lines[1..] {
                if line.width() != first.width() {
                    panic!("Attempted to push line of width {} to CellBuffer of width {}", line.width(), first.width())
                }
            }
            let width: usize = first.width();
            Self { lines, width }
        } else {
            panic!("Cannot build a CellBuffer from an empty iterator, as the width is unknown.")
        }
    }
}





/***** LIBRARY *****/
/// Defines a procedure for rendering a [`VirtualSpan`] of some `S`ource to a [`CellBuffer`].
///
/// This can be done in various ways. For example, for [`S`ources over bytes](SpannableBytes), one
/// could either render it as [UTF-8 text](Utf8Layouter), _or_ as a
/// [raw hexadecimal viewer](HexLayouter).
pub trait Layouter<'s, S: Spannable<'s>> {
    /// Lays a [`VirtualSpan`] out by writing it to the given [`CellBuffer`].
    ///
    /// # Arguments
    /// - `span`: Some [`VirtualSpan`] to write.
    /// - `buffer`: Some [`CellBuffer`] to write it. You can be assured it is empty, but it may
    ///   still have memory allocated for efficiency purposes. Also note that it has a certain line
    ///   [`CellBuffer::width()`] that must be respected.
    fn layout(&self, span: &VirtualSpan<'s, S>, buffer: &mut CellBuffer<S::Elem>);
}



/// [`Layouter`] for rendering a [span over bytes](SpannableBytes) as a UTF-8 text source snippet.
pub struct Utf8Layouter;
impl<'s, S: SpannableBytes<'s>> Layouter<'s, S> for Utf8Layouter {
    fn layout(&self, span: &VirtualSpan<'s, S>, buffer: &mut CellBuffer<u8>) { todo!() }
}



/// [`Layouter`] for rendering a [span over bytes](SpannableBytes) as a grid of Base16-encoded bytes.
pub struct HexLayouter;
impl<'s, S: SpannableBytes<'s>> Layouter<'s, S> for HexLayouter {
    fn layout(&self, span: &VirtualSpan<'s, S>, buffer: &mut CellBuffer<u8>) { todo!() }
}
