//  HEX LAYOUTER.rs
//    by Lut99
//
//  Description:
//!   A battery-included layouter for binary data, showing it in a hexgrid.
//

use std::fmt::{Formatter, Result as FResult};
use std::hash::{BuildHasher, RandomState};

use console::Style;

use super::super::super::themes::SourceTheme;
use super::{Layouter, LineRange, Range2D};
use crate::loc::Range;
use crate::sources::Sources;


/***** LIBRARY *****/
/// A battery-included layouter for binary data, showing it in a hexgrid.
pub struct HexLayouter<'s, 'r, R = RandomState> {
    sources:   &'s Sources<'r, R>,
    max_width: usize,
}

// Constructors
impl<'s, 'r, R> HexLayouter<'s, 'r, R> {
    /// Constructor for the HexLayouter.
    ///
    /// # Arguments
    /// - `max_width`: The maximum number of characters that a line is allowed to be wide.
    /// - `sources`: A [`Sources`] that can map source IDs to actual source text.
    ///
    /// # Returns
    /// A new HexLayouter.
    #[inline(always)]
    pub const fn new(max_width: usize, sources: &'s Sources<'r, R>) -> Self { Self { max_width, sources } }
}

// Layouter
impl<'s, 'r, R> HexLayouter<'s, 'r, R> {
    /// Computes the size of a line number.
    #[inline(always)]
    const fn line_number_width(line: u64) -> usize { if line > 0 { 1 + line.ilog2() as usize / 4 } else { 1 } }
}
impl<'s, 'r, R: BuildHasher> Layouter<u8> for HexLayouter<'s, 'r, R> {
    #[inline(always)]
    fn line_no_width(&self, line: u64) -> usize { if line > 0 { 1 + line.ilog2() as usize / 4 } else { 1 } }

    #[inline(always)]
    fn max_width(&self) -> usize { self.max_width }

    #[track_caller]
    fn range_to_2d(&self, source_id: u64, ranges: impl IntoIterator<Item = Range>) -> Vec<Range2D> {
        // Example row - 00 01 02 03  04 05 06 07  08 09 0A 0B  0C 0D 0E 0F    ..Hello,world!.-

        let ranges = ranges.into_iter();
        let size_hint: (usize, Option<usize>) = ranges.size_hint();
        let mut res = Vec::with_capacity(size_hint.1.unwrap_or(size_hint.0));
        for range in ranges {
            // Split the range into lines
            let mut line: u64 = range.pos / 16;
            let mut pos: usize = (range.pos % 16) as usize;
            let mut len: usize = range.len.fixed_or_else(|| self.sources.source_len(source_id).saturating_sub(range.pos)) as usize;

            // Start by the first, is it overflowing?
            if pos + len <= 16 {
                return vec![Range2D { lines: vec![LineRange { line, pos, len }] }];
            }
            let mut lines = Vec::with_capacity(1 + len as usize / 16);
            lines.push(LineRange { line, pos, len: 16 - pos });
            line += 1;
            len -= 16 - pos;
            pos = 0;

            // Then keep popping full ranges
            while len > 16 {
                lines.push(LineRange { line, pos, len: 16 });
                line += 1;
                len -= 16;
            }

            // Finally, pop whatever remains
            lines.push(LineRange { line, pos, len });

            // That's what happens to this range.
            res.push(Range2D { lines });
        }
        res
    }

    #[inline]
    #[track_caller]
    fn render_loc(&self, f: &mut Formatter<'_>, source_id: u64, pos: u64) -> FResult {
        let name: &str = self.sources.get_name_of(source_id).unwrap_or_else(|| panic!("No ID {source_id} found in sources list"));
        write!(f, "{} at {:X}", name, pos)
    }

    #[inline]
    #[track_caller]
    fn render_line(&self, f: &mut Formatter<'_>, theme: SourceTheme, source_id: u64, line: u64, range: (usize, usize)) -> FResult {
        // Example row - 00 01 02 03  04 05 06 07  08 09 0A 0B  0C 0D 0E 0F    ..Hello,world!.-

        // Find the line
        let line_start = 16 * line;
        let line_end = line_start + 16;
        let line: &[u8] = self
            .sources
            .slice(source_id, line_start..line_end)
            .unwrap_or_else(|| panic!("Please call `Sources::load({source_id}, {line_start}..{line_end})` first before rendering"));

        // Write the lines with correct spacing and whatnot
        for i in 0..16 {
            // Write the preceding space
            if i > 0 {
                if i % 4 == 0 {
                    write!(f, " ")?;
                }
                write!(f, " ")?;
            }

            // Write with the correct styling
            if let Some(b) = line.get(i) {
                let style = if i >= range.0 && i < range.0 + range.1 { theme.highlight_style() } else { theme.base_style() };
                write!(f, "{:02x}", style.apply_to(b))?;
            } else {
                write!(f, "  ")?;
            }
        }

        // Write the ascii part
        write!(f, "    ")?;
        for i in 0..16 {
            if let Some(b) = line.get(i) {
                let c = char::from_u32(*b as u32).unwrap_or('.');
                let style = if i >= range.0 && i < range.0 + range.1 { theme.highlight_style() } else { theme.base_style() };
                write!(f, "{}", style.apply_to(c))?;
            } else {
                write!(f, " ")?;
            }
        }

        // Done
        Ok(())
    }

    #[inline]
    fn render_line_no(&self, f: &mut Formatter<'_>, style: &Style, line: u64) -> FResult { write!(f, "{}", style.apply_to(format!("{line:X}"))) }

    //     #[inline]
    //     fn render_line_replaced(
    //         &self,
    //         f: &mut Formatter<'_>,
    //         theme: SourceTheme,
    //         source_id: u64,
    //         line: u64,
    //         range: (usize, usize),
    //         repl: &[u8],
    //     ) -> FResult {
    //         // Example row - 00 01 02 03  04 05 06 07  08 09 0A 0B  0C 0D 0E 0F    ..Hello,world!.-
    //
    //         // Find the line
    //         let line_start = 16 * line;
    //         let line_end = (line_start as i64 + 16 + diff as i64).max(line_start as i64) as u64;
    //         let line: &[u8] = self
    //             .sources
    //             .slice(source_id, line_start..line_end)
    //             .unwrap_or_else(|| panic!("Please call `Sources::load({source_id}, {line_start}..{line_end})` first before rendering"));
    //
    //         // Write the lines with correct spacing and whatnot
    //         for i in 0..16 {
    //             // Write the preceding space
    //             if i > 0 {
    //                 if i % 4 == 0 {
    //                     write!(f, " ")?;
    //                 }
    //                 write!(f, " ")?;
    //             }
    //
    //             // Write with the correct styling
    //             if let Some(b) = line.get(i) {
    //                 let style = if i >= range.0 && i < range.0 + range.1 { theme.highlight_style() } else { theme.base_style() };
    //                 write!(f, "{:02x}", style.apply_to(b))?;
    //             } else {
    //                 write!(f, "  ")?;
    //             }
    //         }
    //
    //         // Write the ascii part
    //         write!(f, "    ")?;
    //         for i in 0..16 {
    //             if let Some(b) = line.get(i) {
    //                 let c = char::from_u32(*b as u32).unwrap_or('.');
    //                 let style = if i >= range.0 && i < range.0 + range.1 { theme.highlight_style() } else { theme.base_style() };
    //                 write!(f, "{}", style.apply_to(c))?;
    //             } else {
    //                 write!(f, " ")?;
    //             }
    //         }
    //
    //         // Done
    //         Ok(())
    //     }
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
        let layouter = HexLayouter::new(100, &sources);

        let range = Range { pos: 0, len: Length::Fixed(16) };
        assert_eq!(layouter.range_to_2d(id, vec![range]), vec![Range2D { lines: vec![LineRange { line: 0, pos: 0, len: 16 }] }]);
        let range = Range { pos: 14, len: Length::Fixed(2) };
        assert_eq!(layouter.range_to_2d(id, vec![range]), vec![Range2D { lines: vec![LineRange { line: 0, pos: 14, len: 2 }] }]);
        let range = Range { pos: 16, len: Length::Fixed(16) };
        assert_eq!(layouter.range_to_2d(id, vec![range]), vec![Range2D { lines: vec![LineRange { line: 1, pos: 0, len: 16 }] }]);
        let range = Range { pos: 14, len: Length::Fixed(16) };
        assert_eq!(layouter.range_to_2d(id, vec![range]), vec![Range2D {
            lines: vec![LineRange { line: 0, pos: 14, len: 2 }, LineRange { line: 1, pos: 0, len: 14 }],
        }]);
        let range = Range { pos: 14, len: Length::Fixed(32) };
        assert_eq!(layouter.range_to_2d(id, vec![range]), vec![Range2D {
            lines: vec![LineRange { line: 0, pos: 14, len: 2 }, LineRange { line: 1, pos: 0, len: 16 }, LineRange { line: 2, pos: 0, len: 14 }],
        }]);
        let range = Range { pos: 0, len: Length::Indefinite };
        assert_eq!(layouter.range_to_2d(id, vec![range]), vec![Range2D {
            lines: vec![LineRange { line: 0, pos: 0, len: 16 }, LineRange { line: 1, pos: 0, len: 16 }, LineRange { line: 2, pos: 0, len: 16 }],
        }]);
        let range = Range { pos: 14, len: Length::Indefinite };
        assert_eq!(layouter.range_to_2d(id, vec![range]), vec![Range2D {
            lines: vec![LineRange { line: 0, pos: 14, len: 2 }, LineRange { line: 1, pos: 0, len: 16 }, LineRange { line: 2, pos: 0, len: 16 }],
        }]);
    }
}
