//  UTF8 RENDERER.rs
//    by Lut99
//
//  Description:
//!   
//

use std::fmt;
use std::hash::{BuildHasher, RandomState};
use std::io::BufRead as _;

use super::super::Renderer;
use super::super::formatter::{main, replace, sourceless};
use crate::loc::Range;
use crate::sources::Sources;


/***** LIBRARY *****/
/// A renderer for showing binary source code in a hexgrid.
pub struct Utf8Renderer<'s, 'r, R = RandomState> {
    max_line_width: usize,
    sources: &'s Sources<'r, R>,
}

// Constructors
impl<'s, 'r, R> Utf8Renderer<'s, 'r, R> {
    /// Constructor for the Utf8Renderer.
    ///
    /// # Arguments
    /// - `max_line_width`: The maximum line width to render.
    /// - `sources`: A [`Sources]` able to map source IDs to actual texts.
    #[inline(always)]
    pub const fn new(max_line_width: usize, sources: &'s Sources<'r, R>) -> Self { Self { max_line_width, sources } }
}

// Auxillary
impl<'s, 'r, R: BuildHasher> Utf8Renderer<'s, 'r, R> {
    /// Converts a range into the correct lines and positions relative to the lines.
    #[track_caller]
    fn range_to_2d(&self, source_id: u64, ranges: impl IntoIterator<Item = Range>) -> Vec<Vec<(u64, (usize, usize))>> {
        // Order the sources incrementally from start position
        let mut ranges: Vec<(usize, Range)> = ranges.into_iter().enumerate().collect();
        ranges.sort_by_key(|(_, r)| r.pos);

        // Find the last possible range end and get the source text up to that point
        let source_len = self.sources.source_len(source_id);
        let total_end = ranges.iter().map(|(_, r)| r.pos + r.len.fixed_or(source_len.saturating_sub(r.pos))).max().unwrap_or(0);
        let source: &[u8] = self
            .sources
            .slice(source_id, ..total_end)
            .unwrap_or_else(|| panic!("Please call `Sources::load({source_id}, ..{total_end})` first before rendering"));

        // Go through the source text and split ranges are we encounted them
        let mut res: Vec<Vec<(u64, (usize, usize))>> = (0..ranges.len()).map(|_| Vec::new()).collect();
        let mut ranges_buf: Vec<(usize, Range)> = Vec::with_capacity(ranges.len());
        let mut doing: Vec<(usize, u64, usize, u64)> = Vec::with_capacity(ranges.len());
        let mut doing_buf: Vec<(usize, u64, usize, u64)> = Vec::with_capacity(ranges.len());
        let mut line: u64 = 1;
        let mut col: usize = 0;
        for (i, b) in source.into_iter().copied().enumerate() {
            // Consider adding new ranges to track
            ranges_buf.clear();
            for (pos, range) in ranges.drain(..) {
                if range.pos == i as u64 {
                    doing.push((pos, line, col, range.len.fixed_or(source_len.saturating_sub(range.pos))));
                }
                ranges_buf.push((pos, range));
            }
            std::mem::swap(&mut ranges, &mut ranges_buf);

            // Process all ranges that are being tracked
            doing_buf.clear();
            for (pos, line, start, mut len) in doing.drain(..) {
                len -= 1;
                if len == 0 {
                    res[pos].push((line, (start, col - start + 1)));
                    continue;
                }
                doing_buf.push((pos, line, start, len));
            }
            std::mem::swap(&mut doing, &mut doing_buf);

            // Handle newlines occurring
            if b == b'\n' {
                // Linebreak all still-in-progress ranges
                for (pos, line, start, _) in &mut doing {
                    res[*pos].push((*line, (*start, col - *start + 1)));
                    *line += 1;
                    *start = 0;
                }

                // Reset the counters
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }

        // Done
        res
    }
}

// Renderer
impl<'s, 'r, R: BuildHasher> Renderer<u8> for Utf8Renderer<'s, 'r, R> {
    fn render_main<'d>(&self, mut f: main::Formatter<'d, '_>, msg: main::Message<'d>) -> fmt::Result { todo!() }

    fn render_replace<'d>(&self, mut f: replace::Formatter<'d, '_>, msg: replace::Message<'d, u8>) -> fmt::Result {
        //         // First, get the relevant source as zero or more lines
        //         let ranges = self.range_to_2d(msg.source_id, [msg.range]);
        //         let ranges = ranges.into_iter().next().unwrap();
        //
        //         // Compute the loc and relevant information
        //         let max_line_no = ranges.iter().map(|(l, _)| l).copied().max().unwrap_or(0);
        //         let max_line_no_width = if max_line_no > 0 { 1 + max_line_no.ilog2() as usize / 4 } else { 1 };
        //         let name: &str = self.sources.get_name_of(msg.source_id).unwrap();
        //         let loc: String = format!("{} at 0x{:X}", name, msg.range.pos);
        //
        //         // Then, write the message & snippet headers
        //         f.set_max_line_width(self.max_line_width);
        //         let mut message = f.start_message(msg.sev, msg.msg)?;
        //         let mut snippet = message.start_snippet(max_line_no_width, &loc)?;
        //
        //         // First off, write the old source text...
        //         let mut pre = snippet.start_pre_replace();
        //         for (l, (pos, len)) in &ranges {
        //             let mut line = pre.start_line(&format!("{l:X}"))?;
        //             self.write_line(&mut line, msg.source_id, *l, vec![(Severity::Error, (*pos, *len))])?;
        //             line.finish()?;
        //         }
        //         pre.finish();
        //
        //         // ...and then the new
        //         let mut pre = snippet.start_aft_replace();
        //         for (l, (pos, len)) in ranges {
        //             self.write_replaced_lines(&mut pre, msg.source_id, l, (pos, len), msg.repl)?;
        //         }
        //         pre.finish();
        //
        //         // Finish everything up and done is Cees
        //         snippet.finish()?;
        //         message.finish();

        Ok(())
    }

    fn render_sourceless<'d>(&self, mut f: sourceless::Formatter<'d, '_>, msg: sourceless::Message<'d>) -> fmt::Result {
        f.set_max_line_width(self.max_line_width);
        f.write_message(msg.sev, msg.msg)
    }
}





/***** TESTS *****/
#[cfg(test)]
mod tests {
    use super::*;
    use crate::loc::Length;

    #[test]
    fn test_utf8_renderer_range_to_2d() {
        // Prepare an example source text
        let mut sources = Sources::new();
        let id: u64 = sources.add_bytes("ex1", b"Hello, world!\nHallo, wereld!\nGuten tag, werld!\n");
        let layouter = Utf8Renderer::new(100, &sources);

        let range = Range { pos: 0, len: Length::Fixed(13) };
        assert_eq!(layouter.range_to_2d(id, vec![range]), vec![vec![(1, (0, 13))]]);
        let range = Range { pos: 11, len: Length::Fixed(2) };
        assert_eq!(layouter.range_to_2d(id, vec![range]), vec![vec![(1, (11, 2))]]);
        let range = Range { pos: 14, len: Length::Fixed(14) };
        assert_eq!(layouter.range_to_2d(id, vec![range]), vec![vec![(2, (0, 14))]]);
        let range = Range { pos: 11, len: Length::Fixed(17) };
        assert_eq!(layouter.range_to_2d(id, vec![range]), vec![vec![(1, (11, 3)), (2, (0, 14))]]);
        let range = Range { pos: 11, len: Length::Fixed(35) };
        assert_eq!(layouter.range_to_2d(id, vec![range]), vec![vec![(1, (11, 3)), (2, (0, 15)), (3, (0, 17))]]);
        let range = Range { pos: 0, len: Length::Indefinite };
        assert_eq!(layouter.range_to_2d(id, vec![range]), vec![vec![(1, (0, 14)), (2, (0, 15)), (3, (0, 18))]]);
        let range = Range { pos: 11, len: Length::Indefinite };
        assert_eq!(layouter.range_to_2d(id, vec![range]), vec![vec![(1, (11, 3)), (2, (0, 15)), (3, (0, 18))]]);
    }
}
