//  SOURCELESS.rs
//    by Lut99
//
//  Description:
//!   Implements formatting for sourceless messages.
//

use std::fmt;

use unicode_segmentation::{GraphemeIndices, Graphemes, UWordBoundIndices, UnicodeSegmentation as _};

use super::super::messages::MessageSourceless;
use super::Formatter;


/***** ITERATORS *****/
trait NthOrSooner: Iterator {
    fn nth_or_sooner(&mut self, n: usize) -> Option<Self::Item>;
}
impl<I: Iterator> NthOrSooner for I {
    #[inline]
    fn nth_or_sooner(&mut self, mut n: usize) -> Option<Self::Item> {
        let mut peek = self.next()?;
        while n > 0 {
            match self.next() {
                Some(next) => peek = next,
                None => return Some(peek),
            }
            n -= 1;
        }
        Some(peek)
    }
}



struct WordIter<'a>(usize, UWordBoundIndices<'a>, Option<usize>);
impl<'a> WordIter<'a> {
    #[inline(always)]
    fn new(s: &'a str) -> Self { Self(s.len(), s.split_word_bound_indices(), None) }
}
impl<'a> Iterator for WordIter<'a> {
    type Item = (usize, usize);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.2.take() {
            Some(prev_pos) => {
                self.2 = self.1.next().map(|(p, _)| p);
                Some((prev_pos, self.2.unwrap_or(self.0) - prev_pos))
            },
            None => {
                self.2 = Some(self.1.next()?.0);
                self.next()
            },
        }
    }
}



struct WordChunkIter<'a>(usize, usize, Graphemes<'a>);
impl<'a> WordChunkIter<'a> {
    #[inline]
    fn new(max_line_width: usize, s: &'a str) -> Self { Self(max_line_width, 0, s.graphemes(true)) }
}
impl<'a> Iterator for WordChunkIter<'a> {
    type Item = (usize, usize);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Collect enough for a line
        let mut i = 0;
        let mut len = 0;
        while let Some(c) = self.2.next() {
            if i == self.0 {
                // End of this line reached, return everything so-far
                let pos = self.1;
                self.1 += len;
                return Some((pos, len));
            }
            i += 1;
            len += c.len();
        }
        if i == 0 {
            return None;
        }
    }
}



struct MaxLenIter<'a>(usize, &'a str, WordIter<'a>, Option<GraphemeIndices<'a>>);
impl<'a> MaxLenIter<'a> {
    #[inline(always)]
    fn new(max_line_width: usize, s: &'a str) -> Self { Self(max_line_width, s, WordIter::new(s), None) }
}
impl<'a> Iterator for MaxLenIter<'a> {
    type Item = (usize, usize);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // See if we're popping a too-long word
        if let Some(mut iter) = self.3.take() {
            if let Some((pos, _)) = iter.next() {
                match iter.nth_or_sooner(self.0) {
                    Some((end, _)) => return Some((pos, end - pos)),
                    None => return Some((pos, 0)),
                }
            }
        }

        // Else, business as usual
        let (pos, len) = self.2.next()?;
        if len > self.0 {
            self.3 = Some(self.1[pos..len].grapheme_indices(true));
        }
        todo!()
    }
}



/// Splits a string into lines, but properly line wrapped.
struct LineIter<'a>(usize, WordIter<'a>, Option<GraphemeIndices<'a>>);

impl<'a> LineIter<'a> {
    #[inline(always)]
    fn new(max_line_width: usize, s: &'a str) -> Self { Self(max_line_width, WordIter::new(s), None) }
}

impl<'a> Iterator for LineIter<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (pos, word, next_pos) = self.1.next()?;
        let Some(next_pos) = next_pos else {
            let mut iter = word.grapheme_indices(true);
            let mut first = iter.next();
            return;
        };
        todo!()
    }
}





/***** LIBRARY *****/
// Formatting
impl<'a> Formatter<'a> {
    /// Renders the given [`MessageSourceless`] to the internal formatter.
    ///
    /// # Arguments
    /// - `max_line_width`: The maximum line width to render. This counts "graphemes" and ignores
    ///   any styling characters. Think of it as "perceived" width.
    /// - `msg`: The [`MessageSourceless`] to render (i.e., one without source, i.e., requiring no
    ///   input from you!).
    ///
    /// # Errors
    /// This function fails if we failed to write to the internal writer.
    pub fn render_sourceless<'d>(&mut self, max_line_width: usize, msg: MessageSourceless<'d>) -> fmt::Result {
        let sev_style = self.theme.header_severity_style(msg.sev);
        let msg_style = &self.theme.header_extra_msg;

        // Write the initial severity
        let ssev: &str = msg.sev.as_str();
        writeln!(self.f, "{}{}", sev_style.apply_to(ssev), msg_style.apply_to(": "))?;

        // Write any other text, cutting off at words
        let mut col: usize = ssev.len() + 2;
        for (word_pos, word) in msg.msg.split_word_bound_indices() {
            // If the word would go out-of-bounds for the line, linewrap
            let mut letters = word.grapheme_indices(true).collect::<Vec<(usize, &str)>>();
            if col + letters.len() > max_line_width {
                writeln!(self.f)?;
                col = 0;
            }

            // However, if the word as a whole is too long, write chunks of it
            let mut start_pos = 0;
            while letters.len() > max_line_width {
                let term_pos = letters[max_line_width].0;
                writeln!(self.f, "{}", msg_style.apply_to(&word[start_pos..term_pos]));
                start_pos = term_pos;
                letters = letters.split_off(term_pos);
            }
        }
        writeln!(self.f)?;
    }
}
