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
    fn range_to_2d(&self, source_id: u64, ranges: impl IntoIterator<Item = Range>) -> Vec<Vec<(u64, (usize, usize))>> { todo!() }
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
