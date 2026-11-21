//  HEX RENDERER.rs
//    by Lut99
//
//  Description:
//!   Provides the builtin [`HexRenderer`].
//

use std::fmt;
use std::hash::BuildHasher;

use super::super::{Renderer, StyledWrite, replace, sourceless};
use crate::loc::Range;
use crate::sources::Sources;


/***** LIBRARY *****/
/// A renderer for showing binary source code in a hexgrid.
pub struct HexRenderer<'s, 'r, R> {
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
    fn range_to_2d(&self, source_id: u64, range: Range) -> Vec<(u64, (usize, usize))> {
        // Get the source's length and use that to resolve the range
        let source_len: u64 = self.sources.source_len(source_id);
        let (pos, len): (u64, u64) = (range.pos, range.len.fixed_or(source_len.saturating_sub(range.pos)));

        // Split the range into lines
        let mut line: u64 = pos / 16;
        let mut pos: usize = (pos % 16) as usize;
        let mut len: usize = len as usize;

        // Start by the first, does it only touch this line?
        if pos + len <= 16 {
            return vec![(line, (pos, len))];
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
        lines
    }

    /// Writes a source line to some formatter.
    fn write_line(&self, mut f: impl StyledWrite, source_id: u64, line: u64, ranges: Vec<(usize, usize)>) -> fmt::Result {
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
                if ranges.iter().find(|(pos, len)| i >= *pos && i < pos + len).is_some() {
                    f.write_highlighted(format_args!("{b:02x}"))?;
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
                let c = char::from_u32(*b as u32).unwrap_or('.');
                if ranges.iter().find(|(pos, len)| i >= *pos && i < pos + len).is_some() {
                    f.write_highlighted(c)?;
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
}

// Renderer
impl<'s, 'r, R: BuildHasher> Renderer<u8> for HexRenderer<'s, 'r, R> {
    fn render_main(&self, f: (), msg: ()) -> fmt::Result { Ok(()) }

    #[track_caller]
    fn render_replace(&self, mut f: replace::Formatter<'_, '_>, msg: replace::Message<'_, u8>) -> fmt::Result {
        // First, get the relevant source as zero or more lines
        let ranges = self.range_to_2d(msg.source_id, msg.range);

        // Compute the loc and relevant information
        let max_line_no = ranges.iter().map(|(l, _)| l).copied().max().unwrap_or(0);
        let max_line_no_width = if max_line_no > 0 { 1 + max_line_no.ilog2() as usize / 4 } else { 1 };
        let name: &str = self.sources.get_name_of(msg.source_id).unwrap();
        let loc: String = format!("{} at {:X}", name, msg.range.pos);

        // Then, write the message & snippet headers
        f.set_max_line_width(self.max_line_width);
        let mut message = f.start_message(msg.sev, msg.msg)?;
        let mut snippet = message.start_snippet(max_line_no_width, &loc)?;

        // First off, write the old source text...
        let mut pre = snippet.start_pre_replace();
        for (l, (pos, len)) in ranges {
            let mut line = pre.start_line(&format!("{l:X}"))?;
            self.write_line(&mut line, msg.source_id, l, vec![(pos, len)])?;
            line.finish()?;
        }
        pre.finish();

        // ...and then the new

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
