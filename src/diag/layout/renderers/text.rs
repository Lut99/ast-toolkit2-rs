//  TEXT.rs
//    by Lut99
//
//  Description:
//!   Implements a [`Layout`] that renders a sequence of bytes as if it was
//!   source text.
//

use std::fmt::Display;
use std::hash::{BuildHasher, RandomState};
use std::io::{BufReader, Read, Seek, SeekFrom};

use super::super::{Layout, Renderer, View};
use crate::loc::{Length, Range};
use crate::sources::Sources;


/***** CONSTANTS *****/
/// The size that we load around the initial chunk to check for line start- and endings.
///
/// In bytes.
const LINE_SEARCH_CHUNK_SIZE: usize = 4096;





/***** HELPERS *****/
/// Iterator that does buffered reads on a `R`eader in reverse.
struct RevIter<'n, 'r, 'b, R> {
    name:    &'n str,
    source:  &'r mut R,
    chunk_i: usize,
    chunk_j: usize,
    buf:     &'b mut [u8],
    buf_i:   usize,
}
impl<'n, 'r, 'b, R> RevIter<'n, 'r, 'b, R> {
    fn new(name: &'n str, source: &'r mut R, buf: &'b mut [u8], pos: usize) -> Self {
        // We start with: chunk starting at the current pos. This is wrong, but we need to do a
        // first read, still; so think of this as the chunk _after_ the first
        Self { name, source, chunk_i: pos, chunk_j: pos.saturating_add(1), buf, buf_i: 1 }
    }
}
impl<'n, 'r, 'b, R: Seek + Read> Iterator for RevIter<'n, 'r, 'b, R> {
    type Item = Result<(usize, u8), crate::sources::Error>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Stop condition: the end of the readable chunk is 0
        let chunk_len: usize = self.chunk_j - self.chunk_i;
        if chunk_len == 0 {
            return None;
        }

        // Check if we need to read a new chunk
        if self.buf_i == self.chunk_j - self.chunk_i {
            // Move one chunk to the left
            self.buf_i = 0;
            self.chunk_j = self.chunk_i;
            self.chunk_i = self.chunk_i.saturating_sub(self.buf.len());
            if self.chunk_i == self.chunk_j {
                // OK nothing more to read
                return None;
            }

            // Then populate the buffer with that chunk
            if let Err(err) = self.source.seek(SeekFrom::Start(self.chunk_i as u64)) {
                return Some(Err(crate::sources::Error::ReaderSeek { name: self.name.into(), err }));
            }
            if let Err(err) = self.source.read_exact(&mut self.buf[..self.chunk_j - self.chunk_i]) {
                return Some(Err(crate::sources::Error::ReaderRead { name: self.name.into(), err }));
            }
        }

        // Now we can read the current character in the buf
        let buf_i: usize = chunk_len.saturating_sub(1) - self.buf_i;
        let res: u8 = self.buf[buf_i];
        self.buf_i += 1;
        Some(Ok((self.chunk_i + buf_i, res)))
    }
}





/***** LIBRARY *****/
/// Defines a renderer for source texts consisting of bytes as if it is readable UTF-8 text.
///
/// If your bytes aren't necessarily valid UTF-8, consider using the
/// [`HexRenderer`](super::HexRenderer) instead.
pub struct TextRenderer<'s, 'r, H = RandomState> {
    /// The maximum width of a single line.
    max_line_width: u64,
    /// A collection of sources that we use to map IDs back to text.
    sources: &'s mut Sources<'r, H>,
}

// Constructors
impl<'s, 'r, H> TextRenderer<'s, 'r, H> {
    /// Constructor for the TextRenderer that defines it without any sources to read from.
    ///
    /// # Arguments
    /// - `max_line_width`: The maximum line width to render. Any line longer than this will be
    ///   truncated.
    /// - `sources`: A collection of [`Sources`] that we use to read context source text to render.
    ///
    /// # Returns
    /// A new TextRenderer.
    #[inline]
    pub const fn new(max_line_width: u64, sources: &'s mut Sources<'r, H>) -> Self { Self { max_line_width, sources } }
}

// Auxillary
impl<'s, 'r, H: BuildHasher> TextRenderer<'s, 'r, H> {
    /// Given a `range`, attempts to extend it with the given source information s.t. it
    /// encapsulates whole lines.
    fn get_snippet_range<'a>(&mut self, id: u64, name: &str, source_len: u64, range: Range) -> Result<Range, crate::sources::Error> {
        // First we find the start position - it is the first position we find before a newline
        // (or START)
        let source: &mut _ = self.sources.source(id);
        let mut buf: Vec<u8> = vec![0; LINE_SEARCH_CHUNK_SIZE];
        let mut pos: u64 = range.pos;
        for b in RevIter::new(name, source, &mut buf, range.pos as usize) {
            let (i, b): (usize, u8) = b?;

            // Check if it's a newline
            if b == b'\n' {
                // Done
                break;
            }
            pos = i as u64;
        }

        // Then scan the reader for newline boundaries on the RIGHT
        let mut snippet_end: u64 = range.pos + range.len.fixed();
        chunk_start = snippet_end;
        'end: while snippet_end < source_len {
            // Load the chunk first
            if let Err(err) = source.seek(SeekFrom::Start(chunk_start)) {
                return Err(crate::sources::Error::ReaderSeek { name: name.into(), err });
            }
            let chunk_end: u64 = chunk_start
                + match source.read(&mut chunk) {
                    Ok(len) => len as u64,
                    Err(err) => {
                        return Err(crate::sources::Error::ReaderRead { name: name.into(), err });
                    },
                };

            // Search it
            while snippet_end < chunk_end {
                if chunk[snippet_end as usize] == b'\n' {
                    // We found a newline!
                    break 'end;
                }
                if snippet_end >= source_len {
                    break 'end;
                }
                snippet_end += 1;
            }

            // Else, prepare to load the next chunk
            chunk_start = chunk_end;
        }

        // The found range should be the text including newlines, now...
        Ok(Range { pos: snippet_start, len: Length::Fixed(snippet_end - snippet_start) })
    }
}

// Renderer
impl<'s, 'r, H: BuildHasher> Renderer<u8> for TextRenderer<'s, 'r, H> {
    type Error = crate::sources::Error;

    #[inline]
    #[track_caller]
    fn display_id(&self, id: u64) -> impl Display { self.sources.get_name_of(id).unwrap_or_else(|| panic!("Unknown ID {id}")) }

    #[inline]
    fn display_seq(&self, seq: &[u8]) -> impl Display { String::from_utf8_lossy(seq) }

    #[inline]
    #[track_caller]
    fn layout<'a>(&mut self, id: u64, mut view: View<'a, u8>) -> Result<Layout<'a>, Self::Error> {
        // Ensure that our range is contextualized
        let name: String = self.sources.get_name_of(id).unwrap_or_else(|| panic!("Unknown ID {id}")).into();
        let source_len: u64 = self.sources.source_len(id);
        view.range = view.range.contextualize(source_len);

        // Find the source text of the range but up until newline boundaries in either direction
        let snippet_range: Range = self.get_snippet_range(id, &name, source_len, view.range)?;

        todo!()
    }
}





/***** TESTS *****/
#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_text_renderer_rev_iter() {
        let mut source: Cursor<Vec<u8>> = Cursor::new(b"Hello, world!".into());
        let mut buf: Vec<u8> = vec![0; 3];
        let rev: Vec<u8> = RevIter::new("test", &mut source, &mut buf, 13).map(|r| r.map(|(_, b)| b)).collect::<Result<Vec<u8>, _>>().unwrap();
        assert_eq!(rev, b"!dlrow ,olleH");

        let rev: Vec<u8> = RevIter::new("test", &mut source, &mut buf, 5).map(|r| r.map(|(_, b)| b)).collect::<Result<Vec<u8>, _>>().unwrap();
        assert_eq!(rev, b"olleH");

        let rev: Vec<u8> = RevIter::new("test", &mut source, &mut buf, 0).map(|r| r.map(|(_, b)| b)).collect::<Result<Vec<u8>, _>>().unwrap();
        assert_eq!(rev, b"");

        let rev: Vec<usize> = RevIter::new("test", &mut source, &mut buf, 13).map(|r| r.map(|(i, _)| i)).collect::<Result<Vec<usize>, _>>().unwrap();
        assert_eq!(rev, vec![12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0]);
    }

    #[test]
    fn test_text_renderer_get_snippet_range() {
        // Setup
        let mut sources = Sources::new();
        let id: u64 = sources.add_str("test", "Hello, world!\nGoodbye, world!\nChill out, world!\n");
        let mut renderer = TextRenderer::new(100, &mut sources);

        // Try to get some ranges
        assert_eq!(renderer.get_snippet_range(id, "test", 48, Range::from(14..31)).unwrap(), Range::from(15..31));
    }
}
