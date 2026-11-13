//  HEXADECIMAL LAYOUTER.rs
//    by Lut99
//
//  Description:
//!   Defines a layouter that lays out bytes as hexadecimal.
//

use std::hash::{BuildHasher, RandomState};

use super::super::{Layouter, LineRange, Range2D};
use crate::loc::Range;
use crate::sources::Sources;


/***** LIBRARY *****/
/// A layouter that lays out bytes as a hexviewer grid.
pub struct HexLayouter<'s, 'r, R = RandomState> {
    sources: &'s Sources<'r, R>,
}

// Constructors
impl<'s, 'r, R> HexLayouter<'s, 'r, R> {
    /// Constructor for the HexLayouter.
    ///
    /// # Arguments
    /// - `sources`: A [`Sources`] that can map source IDs to actual source text.
    ///
    /// # Returns
    /// A new HexLayouter.
    #[inline(always)]
    pub const fn new(sources: &'s Sources<'r, R>) -> Self { Self { sources } }
}

// Layouter
impl<'s, 'r, R: BuildHasher> Layouter<u8> for HexLayouter<'s, 'r, R> {
    fn layout(&self, source_id: u64, ranges: Vec<Range>) -> Vec<Range2D> {
        // Example row - 00 01 02 03  04 05 06 07  08 09 0A 0B  0C 0D 0E 0F    ..Hello,world!.-

        let mut res = Vec::with_capacity(ranges.len());
        for range in ranges {
            // Split the range into lines and keep generating layouters
            let mut line: u64 = range.pos / 16;
            let mut pos: u64 = range.pos % 16;
            let mut len: u64 = range.len.fixed_or_else(|| self.sources.source_len(source_id));
            let mut lines = Vec::with_capacity(len as usize / 16);
            while len > 16 - pos {
                lines.push(LineRange { line, pos, len: 16 });
                line += 1;
                pos += 16;
                len -= 16;
            }
            lines.push(LineRange { line, pos, len });

            // That's what happens to this range.
            res.push(Range2D { lines });
        }
        res
    }

    fn render(&self) {}
}




/***** TESTS *****/
#[cfg(test)]
mod tests {
    use super::*;
    use crate::loc::Length;

    #[test]
    fn test_hex_layouter_layout() {
        // Prepare an example source text
        let mut sources = Sources::new();
        let id: u64 = sources.add_bytes("ex1", b"0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF");
        let layouter = HexLayouter::new(&sources);

        let range = Range { pos: 0, len: Length::Fixed(16) };
        assert_eq!(layouter.layout(id, vec![range]), vec![Range2D { lines: vec![LineRange { line: 0, pos: 0, len: 16 }] }]);
    }
}
