//  ANNOTATED.rs
//    by Lut99
//
//  Description:
//!   Defines a frozen, annotations-rendered version of the [`LayoutBuffer`].
//

use std::fmt::{Display, Formatter, Result as FResult, Write as _};

use unicode_segmentation::UnicodeSegmentation as _;

use super::layout::{self, AnnotRange, Color, LayoutBuffer, Position, Range};
use crate::themes::Theme;


/***** HELPER FUNCTIONS *****/
/// Replaces all [`AnnotRange`]s with replacements with two ranges, one with a message manually
/// suggesting the replacement and one with the original message, if that range annotates something
/// already annoted by something else.
///
/// This procedure is needed because it's unclear how the range of the other thing needs to be
/// updated when we apply a replacement.
///
/// # Arguments
/// - `annots`: A list of [`AnnotRange`]s that we assume are all of them (so we can check for
///   uniqueness).
fn dereplace_multi_annotated_suggestions<E: Display>(annots: &mut Vec<AnnotRange<E>>) {
    // First, determine for every annotation with a replacement if it is unique
    let mut nonunique_annots: Vec<usize> = Vec::with_capacity(4);
    'annots: for (i, annot) in annots.iter().enumerate() {
        if annot.replacement.is_some() {
            // Find any others in the same range
            for (j, annot2) in annots.iter().enumerate() {
                if i == j {
                    continue;
                }
                if annot.range.overlaps_with(&annot2.range) {
                    nonunique_annots.push(i);
                    continue 'annots;
                }
            }
        }
    }

    // For the non-unique ones, tear them in two messages saying there is a suggestion and one
    // doing the message (if any).
    for i in nonunique_annots {
        // SAFETY: We can unwrap because we only put indices we've seen in `nonunique_annots`
        let annot: &mut AnnotRange<E> = unsafe { annots.get_unchecked_mut(i) };
        let message: Option<String> = annot.message.take();

        // Replace the message with one doing the suggestion
        annot.message = Some({
            // SAFETY: We can unwrap because we only add elements to `nonunique_annots` if
            // `annot.replacement` is there
            let replace: Vec<E> = unsafe { annot.replacement.take().unwrap_unchecked() };
            let mut message: String = String::with_capacity(14 + replace.len() + 1);
            message.push_str("Replace with '");
            for value in replace {
                write!(&mut message, "{}", value).unwrap();
            }
            message.push('\'');
            message
        });

        // Add a new annotation with the same range and everything doing the original message
        // (so it's below the suggestion)
        if message.is_some() {
            let color: Color = annot.color;
            let range: Range = annot.range;
            annots.push(AnnotRange { replacement: None, message, color, range });
        }
    }
}



/// Applies all replacements in the given list of [`AnnotRange`]s to the rendered text in [`A]

/***** FORMATTERS *****/
/// Renders a [`LayoutBuffer`] to some [`Formatter`].
pub struct AnnotatedBufferFormatter<'b, E, T> {
    /// The buffer to render.
    buffer: &'b AnnotatedBuffer<E>,
    /// The theme to render it with.
    theme:  T,
}
impl<'b, E: Display, T: Theme> Display for AnnotatedBufferFormatter<'b, E, T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        let Self { buffer, theme } = self;

        // Scan to find the line width
        let mut max_l_len: usize = 0;
        for line in &buffer.lines {
            max_l_len = std::cmp::max(max_l_len, line.l.as_ref().map(String::len).unwrap_or(0));
        }

        // Format it
        for line in &buffer.lines {
            if let Some(l) = &line.l {
                write!(f, "{l:>len$}", len = max_l_len)?;
            } else {
                write!(f, "{:>len$}", "", len = max_l_len)?;
            }
            write!(f, "| ")?;
            for cell in line.cells.iter() {
                if let Some(value) = &cell.value {
                    write!(f, "{value}")?;
                } else {
                    write!(f, " ")?;
                }
            }
        }
        Ok(())
    }
}





/***** LIBRARY *****/
/// Defines a frozen, annotations rendered version of a [`Cell`](layout::Cell) in [`layout`].
#[derive(Clone, Debug)]
pub struct Cell<E> {
    /// The value in the cell, if any.
    value: Option<E>,
    /// The color this cell has.
    color: Color,
}

// Conversion
impl<I, E> From<layout::Cell<I, E>> for Cell<E> {
    #[inline]
    fn from(value: layout::Cell<I, E>) -> Self {
        let layout::Cell { i: _, value } = value;
        Self { value, color: Color::Plain }
    }
}



/// Defines a Cell for annotated lines.
#[derive(Clone, Debug)]
pub enum CellAnnot {
    /// It has some text in it.
    Text(&'static str),
    /// It has a numeric marker in it to link it to a reference far, far away.
    LinkMarker(usize, Color),
    /// It has an emphasized marker in it (e.g., `^`).
    EmphMarker(Color),
    /// It has a normal marker in it (e.g., `-`).
    Marker(Color),
    /// It's an empty cell.
    Empty,
}



/// Defines a frozen, annotations rendered version of a [`Line`](layout::Line) in [`layout`].
#[derive(Clone, Debug)]
pub struct Line<E> {
    /// Any line number for this line.
    l: Option<String>,
    /// The rendered line as in the original [`LineBuffer`] (minus some source context).
    cells: Vec<Cell<E>>,
    /// Any "annotation lines" that come _before_ this cell. This is the place where we put markers
    /// and messages and whatnot.
    annot_before: Vec<Vec<CellAnnot>>,
    /// Any "annotation lines" that come _after_ this cell. This is the place where we put markers
    /// and messages and whatnot.
    annot_after: Vec<Vec<CellAnnot>>,
}

// Conversion
impl<I, E> From<layout::Line<I, E>> for Line<E> {
    #[inline]
    fn from(value: layout::Line<I, E>) -> Self {
        let layout::Line { l, cells } = value;
        Self { l, cells: cells.into_iter().map(Cell::from).collect(), annot_before: Vec::new(), annot_after: Vec::new() }
    }
}




/// Defines a frozen, annotations-rendered version of the [`LayoutBuffer`].
///
/// You typically obtain this by calling [`LayoutBuffer::apply_annotations()]`.
#[derive(Clone, Debug)]
pub struct AnnotatedBuffer<E> {
    /// The list of lines in this buffer.
    lines:    Vec<Line<E>>,
    /// Any dangling reference messages we still need to place.
    messages: Vec<(usize, String)>,
}

// Formatting
impl<E: Display> AnnotatedBuffer<E> {
    /// Shows the rendered version of the source text, with annotations and all.
    ///
    /// # Arguments
    /// - `theme`: Some [`Theme`] to color this rendering.
    ///
    /// # Returns
    /// An [`AnnotatedBufferFormatter`] that can render this buffer.
    #[inline]
    pub const fn display<T>(&self, theme: T) -> AnnotatedBufferFormatter<E, T> { AnnotatedBufferFormatter { buffer: self, theme } }
}

// Conversion
impl<I, E: Display> LayoutBuffer<I, E> {
    /// Applies all the annotations in this buffer.
    ///
    /// This will freeze the buffer, preventing you from adding any more. In return, the returned
    /// version is actually renderable.
    ///
    /// # Arguments
    /// - `max_line_width`: A maximum line width to apply when rendering the annotations.
    ///
    /// # Returns
    /// An [`AnnotatedBuffer`] that can be [displayed](AnnotatedBuffer::display()) to the user.
    #[inline]
    pub fn apply_annotations(self, max_line_width: usize) -> AnnotatedBuffer<E> {
        let LayoutBuffer { lines, mut annots } = self;
        let mut res = AnnotatedBuffer { lines: lines.into_iter().map(Line::from).collect(), messages: Vec::new() };

        // 1. Make sure that anything with a `replacement` is the only thing annotation that range
        dereplace_multi_annotated_suggestions(&mut annots);

        // 2. Apply replacements, updating any range coming after it appropriately.
        apply_replacements(&mut res, &mut annots);

        // Now we can run our beautiful placement algorithm
        'annots: for annot in annots {
            // Find the line this concerns itself with
            let mut todo: Option<(Option<&String>, Range)> = Some((annot.message.as_ref(), annot.range));
            while let Some((message, range)) = todo.take() {
                // Get the line in the range left to do
                let line: &mut Line<_> =
                    res.lines.get_mut(range.start.line).unwrap_or_else(|| panic!("Got range in annotation that isn't within range!"));
                if line.cells.is_empty() {
                    // Empty line, just skip it
                    if range.end.line > range.start.line {
                        // There's a possibility it's the next line still and this just covers an
                        // empty one. So retry for that line.
                        todo = Some((message, Range { start: Position::new(range.start.line + 1, 0), end: range.end }));
                        continue;
                    } else {
                        // This means the end lies on an empty line.
                        panic!("Got illegal range end which points to a non-existent cell on an empty line");
                    }
                }
                let line_end: usize = line.cells.len() - 1;

                // Try to find where to place the annotation itself. In order of preference:

                // Check if we can place below the span, to the right...
                let message_len: usize = message.map(|m| m.graphemes(true).count()).unwrap_or(0);
                let message_len_with_whitespace: usize = message_len + 1;
                if range.start.line == range.end.line {
                    // The end of this range is at the same line, which is promising. We just have
                    // to check whether it wouldn't be confusing.
                }

                // If this range goes beyond the line, we'll try to place the text _left_ first
                let full_range: Range = if range.end.line > range.start.line {
                    if message_len_with_whitespace <= range.start.col {
                        Range { start: Position::new(range.start.line, range.start.col - message_len_with_whitespace), end: range.end }
                    } else {
                        let full_len: usize = line.cells.len() - range.start.col;
                        // We'd love to place left, but unfortunately, there's no space. Attempt to annotate below
                        //
                    }
                } else {
                    // We attempt to place it to the _right_ first
                };
            }
        }
        res
    }
}

// NOTES about placement:
// 1. When replacing...
//     - ...remember that markers can be truncated/expanded, text has to be moved (potentially very
//       far away)
// 2. When truncating...
//     - ...remember that we may have to move the text into separate placements.
