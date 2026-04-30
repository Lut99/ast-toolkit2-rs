//  LAYOUTERS.rs
//    by Lut99
//
//  Description:
//!   Defines so-called layouters, which can take snippets of source text and
//!   arrange them differently suiting with the user's usecase.
//

use std::borrow::Cow;

use ast_toolkit_span::{Span, Spannable, SpannableBytes};
use unicode_segmentation::UnicodeSegmentation as _;

use super::buffers::layout::{LayoutBuffer, Line};
use crate::layout::buffers::layout::Cell;


/***** CONSTANTS *****/
/// Defines the character we use for when we encounter non-UTF-8 byte sequences.
pub const UNKNOWN_CHAR: &'static str = "\u{FFFD}";





/***** HELPER FUNCTIONS *****/
/// Converts the _lower_ half of a byte into a single hexadecimal character, uppercase.
///
/// # Arguments
/// - `b`: A bytes with the top four bits set to `0`.
///
/// # Returns
/// A single byte representing the UTF-8 hexadecimal.
///
/// # Panics
/// If the given byte is too high.
#[inline]
#[track_caller]
fn render_byte(b: u8) -> char {
    match b {
        0x00 => '0',
        0x01 => '1',
        0x02 => '2',
        0x03 => '3',
        0x04 => '4',
        0x05 => '5',
        0x06 => '6',
        0x07 => '7',
        0x08 => '8',
        0x09 => '9',
        0x0A => 'A',
        0x0B => 'B',
        0x0C => 'C',
        0x0D => 'D',
        0x0E => 'E',
        0x0F => 'F',
        _ => panic!("Given byte 0x{b:X} is too high (> 0x0F)"),
    }
}



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
    source.match_while(|elem| {
        // We initially match on newlines
        if *elem == b'\n' {
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
                return false;
            }
        }

        // Now update our position according to the element and keep iterating
        i += 1;
        true
    });
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





/***** LIBRARY *****/
/// Defines a general Layouter that will take care of physically rendering everything.
pub trait Layouter<'s, S: Spannable<'s>> {
    type CellValue;

    /// Renders some [`Span`] to a [`LayoutBuffer`].
    ///
    /// # Arguments
    /// - `span`: Some [`Span`] to layout.
    ///
    /// # Returns
    /// A [`LayoutBuffer`] that will do the layouting.
    fn layout(&self, span: Span<S>) -> LayoutBuffer<S::SourceId, Self::CellValue>;
}



/// Defines a Layouter which lays something out as UTF-8 text.
pub struct TextLayouter;
impl TextLayouter {
    /// Constructs a new TextLayouter.
    ///
    /// This function is only here for convenience. This struct does not have any arguments.
    ///
    /// # Returns
    /// A TextLayouter.
    #[inline]
    pub const fn new() -> Self { Self }
}
impl<'s, S: Clone + SpannableBytes<'s>> Layouter<'s, S> for TextLayouter {
    type CellValue = Cow<'s, str>;

    fn layout(&self, span: Span<S>) -> LayoutBuffer<S::SourceId, Self::CellValue> {
        let (l, lines): (Option<usize>, Vec<Span<S>>) = find_touched_lines(span);
        lines
            .into_iter()
            .enumerate()
            .map(|(i, span)| {
                // Get the absolute span offset
                let source_len: usize = span.source().len();
                let mut start: usize = span.range().start_resolved(source_len).unwrap_or(0);

                // Prepare a `Line` buffer with the correct line number
                let mut bytes: &[u8] = span.as_bytes();
                let mut line = Line::with_capacity(bytes.len());
                // SAFETY: We can unwrap here because `l` is only every `None` if there are no
                // lines (in which case this closure won't be called)
                line.set_line_number(unsafe { l.unwrap_unchecked() } + i);

                // Interpret the span as UTF-8, manually injecting owned `Cow`s in order to re-use
                // as much from the source as possible.
                // NOTE: This section is very much taken from
                // <https://doc.rust-lang.org/std/str/struct.Utf8Error.html#examples>
                loop {
                    match std::str::from_utf8(bytes) {
                        Ok(valid) => {
                            // The remaining `bytes` are valid; store them and be done with it
                            for (i, s) in valid.grapheme_indices(true) {
                                line.push(Cell::from_source(span.source_id(), start + i, Cow::Borrowed(s)));
                            }
                            break;
                        },
                        Err(err) => {
                            // It was valid up to a certain point; so those we can add
                            let (valid, after_valid) = bytes.split_at(err.valid_up_to());
                            // SAFETY: We can do this because we know they are valid up to `valid`
                            for (i, s) in unsafe { std::str::from_utf8_unchecked(valid) }.grapheme_indices(true) {
                                line.push(Cell::from_source(span.source_id(), start + i, Cow::Borrowed(s)));
                            }
                            // Now push the "unknown" character
                            line.push(Cell::from_source(span.source_id(), start + valid.len(), Cow::Owned(UNKNOWN_CHAR.into())));

                            // If it's just an unexpected byte, we continue. Else, we quit and don't
                            // add the last part
                            if let Some(invalid_len) = err.error_len() {
                                start += valid.len() + invalid_len;
                                bytes = &after_valid[invalid_len..];
                            } else {
                                break;
                            }
                        },
                    }
                }
                line
            })
            .collect()
    }
}

/// Defines a Layouter which lays something out as a hex reader.
pub struct BytesLayouter {
    /// The size of every line.
    pub line_len: usize,
}
impl Default for BytesLayouter {
    #[inline]
    fn default() -> Self { Self { line_len: 16 } }
}
impl BytesLayouter {
    /// Constructs a new BytesLayouter that will create lines of the given size.
    ///
    /// # Arguments
    /// - `line_len`: The number of bytes represented by every line.
    ///
    /// # Returns
    /// A new BytesLayouter.
    #[inline]
    pub const fn new(line_len: usize) -> Self { Self { line_len } }
}
impl<'s, S: Clone + SpannableBytes<'s>> Layouter<'s, S> for BytesLayouter {
    type CellValue = char;

    fn layout(&self, span: Span<S>) -> LayoutBuffer<S::SourceId, Self::CellValue> {
        // Get the chunks of 16 which will be our lines
        let source_len: usize = span.source().len();
        let start: usize = span.range().start_resolved(source_len).unwrap_or(0);
        let lines: Vec<Span<S>> = find_touched_chunks(self.line_len, span.clone());

        // Prepare the initial line
        let mut line = Line::with_capacity(2 * self.line_len + self.line_len + 1);
        for j in 0..16u8 {
            if j > 0 {
                line.push(Cell::from_value(' '));
            }
            line.push(Cell::from_value(render_byte((0xF0 & j) >> 4)));
            line.push(Cell::from_value(render_byte(0x0F & j)));
        }
        line.push(Cell::from_value('\n'));

        // Render them to actual lines
        Some(line)
            .into_iter()
            .chain(lines.into_iter().enumerate().map(|(i, s)| {
                let mut line = Line::with_capacity(2 * self.line_len + self.line_len + 1);
                line.set_line_number(format!("{:X}", start + i * self.line_len));
                for (j, b) in s.as_bytes().iter().copied().enumerate() {
                    if j > 0 {
                        line.push(Cell::from_value(' '));
                    }
                    line.push(Cell::from_source(span.source_id(), start + i * self.line_len + j, render_byte((0xF0 & b) >> 4)));
                    line.push(Cell::from_source(span.source_id(), start + i * self.line_len + j, render_byte(0x0F & b)));
                }
                line.push(Cell::from_value('\n'));
                line
            }))
            .collect()
    }
}





/***** TESTS *****/
#[cfg(test)]
mod tests {
    use super::*;
    use crate::themes::Plain;

    #[test]
    fn test_text_layouter() {
        let span = Span::new("Hello, world!\nGoodbye, world!\nEpic\n");
        let buffer = TextLayouter.layout(span).apply_annotations();
        assert_eq!(buffer.display(Plain).to_string(), "1| Hello, world!\n2| Goodbye, world!\n3| Epic\n");

        let line = span.slice(14..29);
        let buffer = TextLayouter.layout(line).apply_annotations();
        assert_eq!(buffer.display(Plain).to_string(), "2| Goodbye, world!\n");

        let part = line.slice(7..8);
        let buffer = TextLayouter.layout(part).apply_annotations();
        assert_eq!(buffer.display(Plain).to_string(), "2| Goodbye, world!\n");
    }

    #[test]
    fn test_bytes_layouter() {
        let span = Span::new("Hello, world!\nGoodbye, world!\nEpic\n");
        let buffer = BytesLayouter::default().layout(span).apply_annotations();
        assert_eq!(
            buffer.display(Plain).to_string(),
            "  | 00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F\n 0| 48 65 6C 6C 6F 2C 20 77 6F 72 6C 64 21 0A 47 6F\n10| 6F 64 62 79 65 2C 20 77 \
             6F 72 6C 64 21 0A 45 70\n20| 69 63 0A\n"
        );

        let line = span.slice(16..32);
        let buffer = BytesLayouter::default().layout(line).apply_annotations();
        assert_eq!(
            buffer.display(Plain).to_string(),
            "  | 00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F\n10| 6F 64 62 79 65 2C 20 77 6F 72 6C 64 21 0A 45 70\n"
        );

        let part = line.slice(7..8);
        let buffer = BytesLayouter::default().layout(part).apply_annotations();
        assert_eq!(
            buffer.display(Plain).to_string(),
            "  | 00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F\n17| 6F 64 62 79 65 2C 20 77 6F 72 6C 64 21 0A 45 70\n"
        );
    }
}
