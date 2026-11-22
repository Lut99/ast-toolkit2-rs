//  HEX RENDERER.rs
//    by Lut99
//
//  Description:
//!   Provides the builtin [`HexRenderer`].
//

use std::fmt;
use std::fmt::Write as _;
use std::hash::{BuildHasher, RandomState};
use std::ops::Index;

use super::super::super::annots::Severity;
use super::super::formatter::replace::AftReplaceFormatter;
use super::super::messages::LineAnnot;
use super::super::{StyledWrite, main, replace, sourceless};
use super::Renderer;
use crate::loc::Range;
use crate::sources::Sources;


/***** CONSTANTS *****/
/// The char to write in the ascii part if it's not UTF-8 or a control character.
pub const REPLACEMENT_CHAR: char = '•';





/***** HELPERS *****/
/// A wrapper around three slices pretending they are one.
#[derive(Clone, Copy)]
struct VirtualSlice<'a>(&'a [u8], &'a [u8], &'a [u8], usize);
impl<'a> VirtualSlice<'a> {
    #[inline]
    fn remain(self) -> std::range::legacy::Range<usize> { 0..self.3 }

    #[inline]
    const fn len(&self) -> usize { self.0.len() + self.1.len() + self.2.len() + self.3 }
}
impl<'a> Index<usize> for VirtualSlice<'a> {
    type Output = u8;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        if index < self.0.len() {
            &self.0[index]
        } else if index < self.1.len() {
            &self.1[index]
        } else if index < self.2.len() {
            &self.2[index]
        } else {
            panic!("Index {} out-of-bounds for virtualslice with length {}", index, self.len())
        }
    }
}
impl<'a> IntoIterator for VirtualSlice<'a> {
    type IntoIter = std::iter::Chain<
        std::iter::Chain<
            std::iter::Zip<std::iter::Repeat<bool>, std::iter::Copied<std::slice::Iter<'a, u8>>>,
            std::iter::Zip<std::iter::Repeat<bool>, std::iter::Copied<std::slice::Iter<'a, u8>>>,
        >,
        std::iter::Zip<std::iter::Repeat<bool>, std::iter::Copied<std::slice::Iter<'a, u8>>>,
    >;
    type Item = (bool, u8);

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        std::iter::repeat(false)
            .zip(self.0.into_iter().copied())
            .chain(std::iter::repeat(true).zip(self.1.into_iter().copied()))
            .chain(std::iter::repeat(false).zip(self.2.into_iter().copied()))
    }
}





/***** LIBRARY *****/
/// A renderer for showing binary source code in a hexgrid.
pub struct HexRenderer<'s, 'r, R = RandomState> {
    max_line_width: usize,
    sources: &'s Sources<'r, R>,
}

// Constructors
impl<'s, 'r, R> HexRenderer<'s, 'r, R> {
    /// Constructor for the HexRenderer.
    ///
    /// # Arguments
    /// - `max_line_width`: The maximum line width to render.
    /// - `sources`: A [`Sources]` able to map source IDs to actual texts.
    #[inline(always)]
    pub const fn new(max_line_width: usize, sources: &'s Sources<'r, R>) -> Self { Self { max_line_width, sources } }
}

// Auxillary
impl<'s, 'r, R: BuildHasher> HexRenderer<'s, 'r, R> {
    /// Converts a range into the correct lines and positions relative to the lines.
    #[track_caller]
    fn range_to_2d(&self, source_id: u64, ranges: impl IntoIterator<Item = Range>) -> Vec<Vec<(u64, (usize, usize))>> {
        let ranges = ranges.into_iter();
        let size_hint: (usize, Option<usize>) = ranges.size_hint();
        let mut res = Vec::with_capacity(size_hint.1.unwrap_or(size_hint.0));
        for range in ranges {
            // Get the source's length and use that to resolve the range
            let source_len: u64 = self.sources.source_len(source_id);
            let (pos, len): (u64, u64) = (range.pos, range.len.fixed_or(source_len.saturating_sub(range.pos)));

            // Split the range into lines
            let mut line: u64 = pos / 16;
            let mut pos: usize = (pos % 16) as usize;
            let mut len: usize = len as usize;

            // Start by the first, does it only touch this line?
            if pos + len <= 16 {
                res.push(vec![(line, (pos, len))]);
                continue;
            }

            let mut lines = Vec::with_capacity(1 + len as usize / 16);
            lines.push((line, (pos, 16 - pos)));
            line += 1;
            len -= 16 - pos;
            pos = 0;

            // Then keep popping full ranges
            while len > 16 {
                lines.push((line, (pos, 16)));
                line += 1;
                len -= 16;
            }

            // Finally, pop whatever remains
            lines.push((line, (pos, len)));
            res.push(lines);
        }
        res
    }

    /// Given a set of [`LineAnnot`]s, computes for each their equivalent ranges in the rendered
    /// snippet.
    ///
    /// Since we're actually doing it twice (raw binary data + hex), this means there's _two_ of
    /// those ranges per input annotation.
    #[track_caller]
    fn compute_rendered_ranges<'d>(annots: impl IntoIterator<Item = LineAnnot<'d>>) -> Vec<(LineAnnot<'d>, (usize, usize), (usize, usize))> {
        let annots = annots.into_iter();
        let size_hint = annots.size_hint();
        let mut res = Vec::with_capacity(size_hint.1.unwrap_or(size_hint.0));
        for annot in annots {
            // Example row - 00 01 02 03  04 05 06 07  08 09 0A 0B  0C 0D 0E 0F    ..Hello,world!.-

            // Compute the binary range position: every byte is exploded over two hex characters...
            let mut bin_pos = 2 * annot.range.0; // 0001020304050607
            // ...plus a space _in between_ every character...
            bin_pos += annot.range.0; // 00 01 02 03 04 05 06 07
            // ...plus one space in between every four bytes
            bin_pos += annot.range.0 / 4; // 00 01 02 03  04 05 06 07

            // Compute the binary range length: every byte is ecploded over two hex characters...
            let mut bin_len = 2 * annot.range.1; // 0001020304050607
            // ...plus a space _in between_ every character covered;
            bin_len += annot.range.1.saturating_sub(1); // 00 01 02 03 04 05 06 07
            // ...plus one space in between every four bytes _from the start_
            bin_len += (annot.range.0 % 4 + annot.range.1).saturating_sub(1) / 4; // 00 01 02 03  04 05 06 07

            // Compute the second range: it's just the normal one, but boosted with the entire space.
            let ascii_pos = 54 + annot.range.0;
            let ascii_len = annot.range.1;

            // Done, return
            res.push((annot, (bin_pos, bin_len), (ascii_pos, ascii_len)));
        }
        res
    }

    /// Writes a source line to some formatter.
    fn write_line(&self, mut f: impl StyledWrite, source_id: u64, line: u64, ranges: Vec<(Severity, (usize, usize))>) -> fmt::Result {
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
                if let Some((sev, _)) = ranges.iter().find(|(_, (pos, len))| i >= *pos && i < pos + len) {
                    f.write_highlighted(*sev, format_args!("{b:02x}"))?;
                } else {
                    f.write_base(format_args!("{b:02x}"))?;
                }
            } else {
                write!(f, "  ")?;
            }
        }

        // Write the ascii part
        write!(f, "    ")?;
        for i in 0..16 {
            if let Some(b) = line.get(i) {
                let mut c = char::from_u32(*b as u32).unwrap_or(REPLACEMENT_CHAR);
                if c.is_control() {
                    c = REPLACEMENT_CHAR;
                }
                if let Some((sev, _)) = ranges.iter().find(|(_, (pos, len))| i >= *pos && i < pos + len) {
                    f.write_highlighted(*sev, c)?;
                } else {
                    f.write_base(c)?;
                }
            } else {
                write!(f, " ")?;
            }
        }

        // Done
        Ok(())
    }


    /// Writes a replaced source line to some formatter.
    fn write_replaced_lines(&self, f: &mut AftReplaceFormatter, source_id: u64, mut l: u64, range: (usize, usize), repl: &[u8]) -> fmt::Result {
        // Example row - 00 01 02 03  04 05 06 07  08 09 0A 0B  0C 0D 0E 0F    ..Hello,world!.-

        // First, load the part of the line that is before the range
        let (range_pos, range_len) = range;
        let pre_line_start = 16 * l;
        let pre_line_end = pre_line_start + range_pos as u64;
        let pre_line: &[u8] = self
            .sources
            .slice(source_id, pre_line_start..pre_line_end)
            .unwrap_or_else(|| panic!("Please call `Sources::load({source_id}, {pre_line_start}..{pre_line_end})` first before rendering"));

        // Now compute how much we need to load after the range to get a mod of 16 again
        let padded_repl_len = range_pos + repl.len();
        let aft_line_len = (16 - padded_repl_len % 16) % 16;
        // Load that much end line
        let aft_line_start = range_pos + range_len;
        let aft_line_end = aft_line_start + aft_line_len;
        let aft_line: &[u8] = self
            .sources
            .slice(source_id, aft_line_start..aft_line_end)
            .unwrap_or_else(|| panic!("Please call `Sources::load({source_id}, {aft_line_start}..{aft_line_end})` first before rendering"));

        // Put them in one struct
        let total_len: usize = pre_line.len() + repl.len() + aft_line.len();
        let slice = VirtualSlice(pre_line, repl, aft_line, (16 - total_len % 16) % 16);
        #[cfg(debug_assertions)]
        assert!(slice.len() % 16 == 0);

        // Write them all as if it's one text
        let mut i: usize = 0;
        let mut buf: [(bool, u8); 16] = [(false, 0); 16];
        let mut line = f.start_line(&format!("{l:X}"))?;
        for (highlighted, b) in slice.into_iter() {
            // Newline time!
            if i > 0 && i % 16 == 0 {
                // Write the ascii part
                write!(&mut line, "    ")?;
                for (highlighted, b) in buf {
                    let mut c = char::from_u32(b as u32).unwrap_or(REPLACEMENT_CHAR);
                    if c.is_control() {
                        c = REPLACEMENT_CHAR;
                    }
                    if highlighted {
                        line.write_highlighted(Severity::Suggestion, c)?;
                    } else {
                        line.write_base(c)?;
                    }
                }
                line.finish()?;
                l += 1;
                line = f.start_line(&format!("{l:X}"))?;
            }

            // Write the preceding space
            if i > 0 {
                if i % 4 == 0 {
                    write!(&mut line, " ")?;
                }
                write!(&mut line, " ")?;
            }

            // Write with the correct styling
            if highlighted {
                line.write_highlighted(Severity::Suggestion, format_args!("{b:02x}"))?;
            } else {
                line.write_base(format_args!("{b:02x}"))?;
            }
            buf[i % 16] = (highlighted, b);
            i += 1;
        }

        // Write the remaining line and a final newline
        for j in slice.remain() {
            if i + j > 0 {
                if (i + j) % 4 == 0 {
                    write!(&mut line, " ")?;
                }
                write!(&mut line, " ")?;
            }
            write!(&mut line, "  ")?;
        }
        // Write the ascii part
        write!(&mut line, "    ")?;
        for (highlighted, b) in &buf[..i % 16] {
            let mut c = char::from_u32(*b as u32).unwrap_or(REPLACEMENT_CHAR);
            if c.is_control() {
                c = REPLACEMENT_CHAR;
            }
            if *highlighted {
                line.write_highlighted(Severity::Suggestion, c)?;
            } else {
                line.write_base(c)?;
            }
        }
        line.finish()?;

        // Done
        Ok(())
    }
}

// Renderer
impl<'s, 'r, R: BuildHasher> Renderer<u8> for HexRenderer<'s, 'r, R> {
    fn render_main<'d>(&self, mut f: main::Formatter<'d, '_>, msg: main::Message<'d>) -> fmt::Result {
        // Estimate the overall largest line number
        let max_line_no: u64 = msg.snippets.iter().flat_map(|s| s.annots.iter().map(|a| a.range.pos / 16)).max().unwrap_or(0);
        let max_line_no_width = if max_line_no > 0 { 1 + max_line_no.ilog2() as usize / 4 } else { 1 };

        // Start with the main header
        f.set_max_line_width(self.max_line_width);
        let mut message = f.start_message(msg.sev, msg.code, msg.msg, msg.notes)?;
        message.set_max_line_no_width(max_line_no_width);

        // Dive into the snippets
        let mut extra_notes = Vec::new();
        for mut snip in msg.snippets {
            // Filter out annotations that would be out-of-range
            let source_len: u64 = self.sources.source_len(snip.source_id);
            snip.annots = snip.annots.into_iter().filter(|a| a.range.pos < source_len).collect();

            // Skip the snippet if there are no annotations
            let name: &str = self.sources.get_name_of(snip.source_id).unwrap();
            if snip.annots.is_empty() {
                message.write_unannot_snippet(name)?;
                continue;
            }

            // First, get the relevant source as zero or more lines
            // A bit of gymnastics has to happen here to group the annotations together on the same
            // lines.
            let loc: String = format!("{} at 0x{:X}", name, snip.annots.first().map(|a| a.range.pos).unwrap());
            let ranges = self.range_to_2d(snip.source_id, snip.annots.iter().map(|a| a.range));
            let mut annots_by_line = Vec::<(Option<u64>, Vec<LineAnnot>)>::new();
            for (annot, ranges) in snip.annots.into_iter().zip(ranges) {
                for (annot, (line, _)) in annot.split_over_lines(ranges.iter().map(|(_, range)| *range)).into_iter().zip(ranges) {
                    match annots_by_line.iter_mut().find(|(l, _)| l.unwrap() == line) {
                        Some(annots) => annots.1.push(annot),
                        None => annots_by_line.push((Some(line), vec![annot])),
                    }
                }
            }
            annots_by_line.sort_by_key(|(l, _)| *l);

            // Insert missing lines, OR insert a "break" line without annotations.
            let mut prev = annots_by_line.first().map(|(l, _)| l.unwrap());
            let mut i: usize = 0;
            while i < annots_by_line.len() {
                let l = annots_by_line[i].0.unwrap();
                let p = prev.unwrap();
                if p + 2 == l {
                    // There's precisely one line missing. Add it without annotations!
                    annots_by_line.insert(i, (Some(p + 1), Vec::new()));
                    i += 1;
                } else if p + 3 == l {
                    // There's precisely two lines missing. Add them without annotations!
                    annots_by_line.insert(i, (Some(p + 1), Vec::new()));
                    annots_by_line.insert(i + 1, (Some(p + 2), Vec::new()));
                    i += 2;
                } else if p + 3 < l {
                    // There's more than two lines missing. Add the break
                    annots_by_line.insert(i, (None, Vec::new()));
                    i += 2;
                }
                i += 1;
                prev = Some(l);
            }

            // Start a snippet and go over all lines to draw them
            let mut snippet = message.start_snippet(&loc)?;
            for (l, mut annots) in annots_by_line {
                annots.sort_by_key(|a| a.range.0);

                if let Some(l) = l {
                    // Write the source line
                    let mut line = snippet.start_source_line(&format!("{l:X}"))?;
                    self.write_line(&mut line, snip.source_id, l, annots.iter().map(|a| (a.sev, a.range)).collect())?;
                    line.finish()?;

                    // Compute the rendered annotations to show on the resulting line
                    let annots = Self::compute_rendered_ranges(annots);
                    let mut rendered_annots = Vec::with_capacity(2 * annots.len());
                    for (annot, bin_range, ascii_range) in annots {
                        rendered_annots.push(LineAnnot { msg: annot.msg, sev: annot.sev, range: bin_range });
                        rendered_annots.push(LineAnnot { msg: None, sev: annot.sev, range: ascii_range });
                    }

                    // Write the annotation line
                    let mut lines = snippet.start_annot_lines();
                    let extra_notes_len = extra_notes.len();
                    extra_notes.extend(lines.write_annots(extra_notes_len, rendered_annots)?);
                    lines.finish();
                } else {
                    // Write the break line to indicate there's source skipped
                    snippet.write_break_line()?;
                }
            }

            // Finish writing the snippet
            snippet.finish()?;
        }

        message.finish(&extra_notes)?;
        Ok(())
    }

    #[track_caller]
    fn render_replace(&self, mut f: replace::Formatter<'_, '_>, msg: replace::Message<'_, u8>) -> fmt::Result {
        // First, get the relevant source as zero or more lines
        let ranges = self.range_to_2d(msg.source_id, [msg.range]);
        let ranges = ranges.into_iter().next().unwrap();

        // Compute the loc and relevant information
        let max_line_no = ranges.iter().map(|(l, _)| l).copied().max().unwrap_or(0);
        let max_line_no_width = if max_line_no > 0 { 1 + max_line_no.ilog2() as usize / 4 } else { 1 };
        let name: &str = self.sources.get_name_of(msg.source_id).unwrap();
        let loc: String = format!("{} at 0x{:X}", name, msg.range.pos);

        // Then, write the message & snippet headers
        f.set_max_line_width(self.max_line_width);
        let mut message = f.start_message(msg.sev, msg.msg)?;
        let mut snippet = message.start_snippet(max_line_no_width, &loc)?;

        // First off, write the old source text...
        let mut pre = snippet.start_pre_replace();
        for (l, (pos, len)) in &ranges {
            let mut line = pre.start_line(&format!("{l:X}"))?;
            self.write_line(&mut line, msg.source_id, *l, vec![(Severity::Error, (*pos, *len))])?;
            line.finish()?;
        }
        pre.finish();

        // ...and then the new
        let mut pre = snippet.start_aft_replace();
        for (l, (pos, len)) in ranges {
            self.write_replaced_lines(&mut pre, msg.source_id, l, (pos, len), msg.repl)?;
        }
        pre.finish();

        // Finish everything up and done is Cees
        snippet.finish()?;
        message.finish();
        Ok(())
    }

    fn render_sourceless(&self, mut f: sourceless::Formatter<'_, '_>, msg: sourceless::Message<'_>) -> fmt::Result {
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
    fn test_hex_renderer_range_to_2d() {
        // Prepare an example source text
        let mut sources = Sources::new();
        let id: u64 = sources.add_bytes("ex1", b"0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF");
        let layouter = HexRenderer::new(100, &sources);

        let range = Range { pos: 0, len: Length::Fixed(16) };
        assert_eq!(layouter.range_to_2d(id, vec![range]), vec![vec![(0, (0, 16))]]);
        let range = Range { pos: 14, len: Length::Fixed(2) };
        assert_eq!(layouter.range_to_2d(id, vec![range]), vec![vec![(0, (14, 2))]]);
        let range = Range { pos: 16, len: Length::Fixed(16) };
        assert_eq!(layouter.range_to_2d(id, vec![range]), vec![vec![(1, (0, 16))]]);
        let range = Range { pos: 14, len: Length::Fixed(16) };
        assert_eq!(layouter.range_to_2d(id, vec![range]), vec![vec![(0, (14, 2)), (1, (0, 14))]]);
        let range = Range { pos: 14, len: Length::Fixed(32) };
        assert_eq!(layouter.range_to_2d(id, vec![range]), vec![vec![(0, (14, 2)), (1, (0, 16)), (2, (0, 14))]]);
        let range = Range { pos: 0, len: Length::Indefinite };
        assert_eq!(layouter.range_to_2d(id, vec![range]), vec![vec![(0, (0, 16)), (1, (0, 16)), (2, (0, 16))]]);
        let range = Range { pos: 14, len: Length::Indefinite };
        assert_eq!(layouter.range_to_2d(id, vec![range]), vec![vec![(0, (14, 2)), (1, (0, 16)), (2, (0, 16))]]);
    }

    #[test]
    fn test_hex_renderer_compute_rendered_ranges() {
        // Example row - 00 01 02 03  04 05 06 07  08 09 0A 0B  0C 0D 0E 0F    ..Hello,world!.-

        let annot = LineAnnot { msg: None, sev: Severity::Error, range: (0, 4) };
        assert_eq!(HexRenderer::<RandomState>::compute_rendered_ranges([annot]), [(annot, (0, 11), (54, 4))]);
        let annot = LineAnnot { msg: None, sev: Severity::Error, range: (0, 6) };
        assert_eq!(HexRenderer::<RandomState>::compute_rendered_ranges([annot]), [(annot, (0, 18), (54, 6))]);
        let annot = LineAnnot { msg: None, sev: Severity::Error, range: (0, 8) };
        assert_eq!(HexRenderer::<RandomState>::compute_rendered_ranges([annot]), [(annot, (0, 24), (54, 8))]);
        let annot = LineAnnot { msg: None, sev: Severity::Error, range: (0, 1) };
        assert_eq!(HexRenderer::<RandomState>::compute_rendered_ranges([annot]), [(annot, (0, 2), (54, 1))]);
        let annot = LineAnnot { msg: None, sev: Severity::Error, range: (2, 3) };
        assert_eq!(HexRenderer::<RandomState>::compute_rendered_ranges([annot]), [(annot, (6, 9), (56, 3))]);
        let annot = LineAnnot { msg: None, sev: Severity::Error, range: (0, 16) };
        assert_eq!(HexRenderer::<RandomState>::compute_rendered_ranges([annot]), [(annot, (0, 50), (54, 16))]);
        let annot = LineAnnot { msg: None, sev: Severity::Error, range: (5, 11) };
        assert_eq!(HexRenderer::<RandomState>::compute_rendered_ranges([annot]), [(annot, (16, 34), (59, 11))]);
    }
}
