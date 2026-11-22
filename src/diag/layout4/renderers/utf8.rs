//  UTF8 RENDERER.rs
//    by Lut99
//
//  Description:
//!   
//

use std::fmt;
use std::hash::{BuildHasher, RandomState};

use unicode_segmentation::UnicodeSegmentation;

use super::super::super::annots::Severity;
use super::super::StyledWrite;
use super::super::formatter::replace::AftReplaceFormatter;
use super::super::formatter::{main, replace, sourceless};
use super::super::messages::LineAnnot;
use super::Renderer;
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
        let ssource = String::from_utf8_lossy(source);

        // Go through the source text and split ranges are we encounted them
        let mut res: Vec<Vec<(u64, (usize, usize))>> = (0..ranges.len()).map(|_| Vec::new()).collect();
        let mut ranges_buf: Vec<(usize, Range)> = Vec::with_capacity(ranges.len());
        let mut doing: Vec<(usize, u64, usize, u64)> = Vec::with_capacity(ranges.len());
        let mut doing_buf: Vec<(usize, u64, usize, u64)> = Vec::with_capacity(ranges.len());
        let mut line: u64 = 1;
        let mut col: usize = 0;
        for (i, c) in ssource.graphemes(true).enumerate() {
            // Consider adding new ranges to track
            ranges_buf.clear();
            for (pos, range) in ranges.drain(..) {
                if range.pos == i as u64 {
                    doing.push((pos, line, col, range.len.fixed_or(source_len.saturating_sub(range.pos))));
                    continue;
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
            if c == "\r\n" || c == "\n" {
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

        // Flush remaining items
        for (pos, line, start, _) in doing {
            res[pos].push((line, (start, col - start + 1)));
        }

        // Done
        res
    }

    /// Writes a source line to some formatter.
    fn write_line(&self, mut f: impl StyledWrite, source_id: u64, mut line: u64, ranges: Vec<(Severity, (usize, usize))>) -> fmt::Result {
        // Load the entire source (to look for the entire line)
        let mut source: &[u8] =
            self.sources.slice(source_id, ..).unwrap_or_else(|| panic!("Please call `Sources::load({source_id}, ..)` first before rendering"));

        // Quickly search up until the correct line number
        let mut line_start: usize = 0;
        while line_start < source.len() {
            if line == 1 {
                break;
            }
            if source[line_start] == b'\n' {
                line -= 1;
            }
            line_start += 1;
        }
        source = &source[line_start..];

        // Search until line's end
        let mut line_end: usize = 0;
        while line_end < source.len() {
            if source[line_end] == b'\n' {
                break;
            }
            line_end += 1;
        }
        source = &source[..line_end];

        // Now write that source line with correct parts highlighted
        for (col, c) in String::from_utf8_lossy(source).graphemes(true).enumerate() {
            if let Some((sev, _)) = ranges.iter().find(|(_, (pos, len))| col >= *pos && col < *pos + *len) {
                f.write_highlighted(*sev, c)?;
            } else {
                f.write_base(c)?;
            }
        }

        // Done
        Ok(())
    }

    /// Writes a source line to some formatter with replacements.
    fn write_replaced_lines(
        &self,
        f: &mut AftReplaceFormatter<'_, '_, '_>,
        source_id: u64,
        mut line: u64,
        range: (usize, usize),
        repl: &[u8],
    ) -> fmt::Result {
        // Load the entire source (to look for the entire line)
        let mut source: &[u8] =
            self.sources.slice(source_id, ..).unwrap_or_else(|| panic!("Please call `Sources::load({source_id}, ..)` first before rendering"));

        // Quickly search up until the correct line number
        let mut l = line;
        let mut line_start: usize = 0;
        while line_start < source.len() {
            if l == 1 {
                break;
            }
            if source[line_start] == b'\n' {
                l -= 1;
            }
            line_start += 1;
        }
        source = &source[line_start..];

        // Search until line's end
        let mut line_end: usize = 0;
        while line_end < source.len() {
            if source[line_end] == b'\n' {
                line_end += 1;
                break;
            }
            line_end += 1;
        }
        source = &source[..line_end];

        // Now first write the first source
        let ssource = String::from_utf8_lossy(source);
        let mut col: usize = 0;
        let mut iter = ssource.graphemes(true);
        let mut fline = f.start_line(&line.to_string())?;
        while col < range.0 {
            fline.write_base(iter.next().unwrap())?;
            col += 1;
        }
        // Write everything replaced
        for &b in repl {
            if b == b'\n' {
                fline.finish()?;
                line += 1;
                fline = f.start_line(&line.to_string())?;
            } else {
                fline.write_highlighted(Severity::Suggestion, char::from_u32(b as u32).unwrap_or(char::REPLACEMENT_CHARACTER))?;
            }
        }
        if range.1 > 0 {
            iter.nth(range.1 - 1);
        }
        // Write the aftermath
        for c in &mut iter {
            fline.write_base(c)?;
        }
        fline.finish()?;

        // Done
        Ok(())
    }
}

// Renderer
impl<'s, 'r, R: BuildHasher> Renderer<u8> for Utf8Renderer<'s, 'r, R> {
    fn render_main<'d>(&self, mut f: main::Formatter<'d, '_>, msg: main::Message<'d>) -> fmt::Result {
        // Count the overall largest line number
        // Yes this is very expensive. No there's not a better way to do this. Screw UTF-8!
        let max_line_no: u64 = msg
            .snippets
            .iter()
            .map(|s| {
                let source: &[u8] = self
                    .sources
                    .slice(s.source_id, ..)
                    .unwrap_or_else(|| panic!("Please call `Sources::load({}, ..)` first before rendering", s.source_id));
                (source.into_iter().copied().filter(|b| *b == b'\n').count() + 1) as u64
            })
            .max()
            .unwrap_or(0);
        let max_line_no_width = if max_line_no > 0 { 1 + max_line_no.ilog10() as usize } else { 1 };

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
            let ranges = self.range_to_2d(snip.source_id, snip.annots.iter().map(|a| a.range));
            let loc: String = format!("{}:{}:{}", name, ranges[0][0].0, ranges[0][0].1.0 + 1);
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
                    let mut line = snippet.start_source_line(&l.to_string())?;
                    self.write_line(&mut line, snip.source_id, l, annots.iter().map(|a| (a.sev, a.range)).collect())?;
                    line.finish()?;

                    // Write the annotation line
                    let mut lines = snippet.start_annot_lines();
                    let extra_notes_len = extra_notes.len();
                    extra_notes.extend(lines.write_annots(extra_notes_len, annots)?);
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

    fn render_replace<'d>(&self, mut f: replace::Formatter<'d, '_>, msg: replace::Message<'d, u8>) -> fmt::Result {
        // First, get the relevant source as zero or more lines
        let ranges = self.range_to_2d(msg.source_id, [msg.range]);
        let ranges = ranges.into_iter().next().unwrap();

        // Compute the loc and relevant information
        let max_line_no = ranges.iter().map(|(l, _)| l).copied().max().unwrap_or(0);
        let max_line_no_width = if max_line_no > 0 { 1 + max_line_no.ilog10() as usize } else { 1 };
        let name: &str = self.sources.get_name_of(msg.source_id).unwrap();
        let loc: String = format!("{}:{}:{}", name, ranges[0].0, ranges[0].1.0 + 1);

        // Then, write the message & snippet headers
        f.set_max_line_width(self.max_line_width);
        let mut message = f.start_message(msg.sev, msg.msg)?;
        let mut snippet = message.start_snippet(max_line_no_width, &loc)?;

        // First off, write the old source text...
        let mut pre = snippet.start_pre_replace();
        for (l, (pos, len)) in &ranges {
            let mut line = pre.start_line(&l.to_string())?;
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
        let ranges = [Range { pos: 0, len: Length::Fixed(13) }, Range { pos: 29, len: Length::Fixed(17) }];
        assert_eq!(layouter.range_to_2d(id, ranges), vec![vec![(1, (0, 13))], vec![(3, (0, 17))]]);
        let range = Range { pos: 0, len: Length::Fixed(47) };
        assert_eq!(layouter.range_to_2d(id, vec![range]), vec![vec![(1, (0, 14)), (2, (0, 15)), (3, (0, 18))]]);
        let range = Range { pos: 0, len: Length::Fixed(48) };
        assert_eq!(layouter.range_to_2d(id, vec![range]), vec![vec![(1, (0, 14)), (2, (0, 15)), (3, (0, 18)), (4, (0, 1))]]);
    }
}
