//  STEP 3 - PLACEMENT.rs
//    by Lut99
//
//  Description:
//!   Defines the third step in the [layouting algorithm](super).
//

use std::borrow::Cow;

use console::Style;
use unicode_segmentation::{Graphemes as _, UnicodeSegmentation};

use super::super::{Severity, Theme};
use super::layouter::Layouter;
use super::s1_messagify::MessageSourceless;
use super::s2_layout::{Annot2D, Message2D};
use crate::diag::layout3::s2_layout::MessageReplace2D;


/***** HELPERS *****/
/// Defines all possible messages that we will render.
pub enum AnnotatedMessage<'t, 'd, E> {
    /// A "standard" message containing a main message and one or more annotations, but with a
    /// source text.
    Main(AnnotatedMessageMain<'t, 'd>),
    /// A message with a source text but also that replaces source text.
    Replace(MessageReplace2D<'d, E>),
    /// A message without any source text.
    Sourceless(MessageSourceless<'d>),
}



pub struct AnnotatedMessageMain<'t, 'd> {
    /// The severity of the message.
    pub sev:      Severity,
    /// Any code in the message.
    pub code:     Option<&'d str>,
    /// The message of the message.
    pub msg:      &'d str,
    /// The notes in the message.
    pub notes:    Vec<Cow<'d, str>>,
    /// The snippets to show for this message.
    pub snippets: Vec<AnnotatedSnippet<'t, 'd>>,
}

pub struct AnnotatedSnippet<'t, 'd> {
    /// The source ID of the snippet to show.
    pub source_id: u64,
    /// A single snippet to show. It has one or more annotations to show within it.
    pub lines:     Vec<AnnotatedSnippetLine<'t, 'd>>,
}

pub struct AnnotatedSnippetLine<'t, 'd> {
    /// The line number.
    pub line:     u64,
    /// The annotations on this line.
    pub annots:   Vec<Annot2D<'d>>,
    /// The rendered annotations on this line.
    pub rendered: Vec<RenderedLine<'t, 'd>>,
}



pub struct RenderedLine<'t, 'd> {
    /// The sequence of things to draw.
    pub draw: Vec<Draw<'t, 'd>>,
}
impl<'t, 'd> RenderedLine<'t, 'd> {
    #[inline(always)]
    fn new(max_width: usize) -> Self { Self { draw: vec![Draw::Empty(max_width)] } }

    fn consolidate_free_space(&mut self) {
        let mut i: usize = 0;
        while i < self.draw.len().saturating_sub(1) {
            // See if the LHS is empty
            let Draw::Empty(lhs) = &self.draw[i] else {
                i += 1;
                continue;
            };
            let lhs = *lhs;

            // Merge any RHS with it
            let Draw::Empty(rhs) = &self.draw[i + 1] else {
                i += 1;
                continue;
            };
            let rhs = *rhs;

            // Do the merge! And don't advance, there may be more
            self.draw.remove(i + 1);
            self.draw[i] = Draw::Empty(lhs + rhs);
        }
    }

    fn try_insert_draw_at(&mut self, draw_pos: usize, draw: Draw<'t, 'd>) -> bool {
        // Find the slot
        let at_end: usize = draw_pos + draw.len();
        let mut pos: usize = 0;
        for i in 0..self.draw.len() {
            if at_end < pos {
                // It'll never work, bucko
                return false;
            }
            match &mut self.draw[i] {
                Draw::Empty(n) => {
                    // Test if it's within range
                    let end: usize = pos + *n;
                    if at_end > end {
                        // It's not
                        pos += *n;
                        continue;
                    }

                    // Decide what to keep
                    let new_pos: usize = if draw_pos == pos {
                        // Remove the space altogether
                        self.draw.remove(i);
                        i
                    } else {
                        // Shrink the current empty range
                        *n = draw_pos - pos;
                        i + 1
                    };

                    // Insert the marker
                    self.draw.insert(new_pos, draw);

                    // Need to inject anything at the end?
                    if end - at_end > 0 {
                        self.draw.insert(new_pos + 1, Draw::Empty(end - at_end));
                    }

                    // Consolidate the free space and return
                    self.consolidate_free_space();
                    return true;
                },

                Draw::Marker(_, _, n) => pos += *n,

                Draw::Text(_, _, n) => pos += *n,
            }
        }

        // Not found
        false
    }

    fn mutate_first_marker_into_text(&mut self, draw_pos: usize, text: impl Into<Cow<'d, str>>, text_len: usize) {
        // Find the slot
        let at_end: usize = draw_pos + text_len;
        let mut pos: usize = 0;
        for i in 0..self.draw.len() {
            if at_end < pos {
                panic!("Did not find the start of a marker at {draw_pos}")
            }
            match &mut self.draw[i] {
                Draw::Marker(_, style, n) => {
                    // We should now precisely be at the start
                    if draw_pos != pos {
                        pos += *n;
                        continue;
                    }
                    if text_len > *n {
                        panic!("Ain't gonna fit, chief, the text is too large...");
                    }

                    // Make the marker ooooooone shorter
                    *n -= text_len;

                    // Insert the text just before
                    let style = *style;
                    self.draw.insert(i, Draw::Text(text.into(), style, text_len));
                    return;
                },

                Draw::Empty(n) => pos += *n,
                Draw::Text(_, _, n) => pos += *n,
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Draw<'t, 'd> {
    /// It's an empty space of `N` cells.
    Empty(usize),
    /// It's `N` markers with the given color.
    Marker(char, &'t Style, usize),
    /// It's text of `N` cells with the given style.
    Text(Cow<'d, str>, &'t Style, usize),
}
impl<'t, 'd> Draw<'t, 'd> {
    /// The length of the thing drawn, in cells.
    #[inline]
    const fn len(&self) -> usize {
        match self {
            Self::Empty(n) => *n,
            Self::Marker(_, _, n) => *n,
            Self::Text(_, _, n) => *n,
        }
    }
}





/***** LIBRARY *****/
/// Runs the third step in the [layouting algorithm](super).
///
/// # Arguments
/// - `theme`: The [`Theme`] deciding how to render markers and which colours belong to which severity.
/// - `layouter`: A [`Layouter`] which determines the maximum line length.
/// - `msgs`: A list of message to compute annotation placement in.
///
/// # Returns
/// A new list of messages with rendered annotations attached.
pub fn s3_place<'t, 'd, E>(theme: &'t Theme, layouter: &impl Layouter<E>, msgs: Vec<Message2D<'d, E>>) -> Vec<AnnotatedMessage<'t, 'd, E>> {
    let max_width: usize = layouter.max_width();

    // Go through each message...
    let mut res = Vec::with_capacity(msgs.len());
    for msg in msgs {
        match msg {
            Message2D::Main(m) => {
                let mut extra: usize = 1;
                let mut notes: Vec<Cow<'d, str>> = m.notes.into_iter().cloned().map(Cow::Owned).collect();

                // ...each snippet...
                let mut snippets = Vec::with_capacity(m.snippets.len());
                for s in m.snippets {
                    // ...and each line...
                    let mut lines = Vec::with_capacity(s.lines.len());
                    for mut line in s.lines {
                        // ...in order to render the annotations on the special lines
                        line.annots.sort_by_key(|a| a.range.0);

                        // Start by writing marker lines
                        let mut markers = Vec::<usize>::with_capacity(line.annots.len());
                        let mut rendered = vec![RenderedLine::new(max_width)];
                        for annot in &line.annots {
                            let (marker, marker_style) = theme.marker_style(annot.sev);
                            for (l, line) in rendered.iter_mut().enumerate() {
                                // See if there's place here
                                if line.try_insert_draw_at(annot.range.0, Draw::Marker(*marker, marker_style, annot.range.1.max(1))) {
                                    markers.push(l);
                                }
                            }
                            rendered.push({
                                let mut line = RenderedLine::new(max_width);
                                line.try_insert_draw_at(annot.range.0, Draw::Marker(*marker, marker_style, annot.range.1.max(1)));
                                line
                            });
                            markers.push(rendered.len());
                        }

                        // `rendered` now has lines with markers, `markers` maps each annotation to
                        // on which line its markers are placed.
                        // Now it's time to crime!

                        for (i, annot) in line.annots.iter().enumerate() {
                            // Doing more than a marker is not necessary if there is no message
                            let Some(msg) = &annot.msg else { continue };
                            if msg.is_empty() {
                                continue;
                            }
                            let (_, marker_style) = theme.marker_style(annot.sev);
                            let msg_len: usize = msg.graphemes(true).count();

                            // Try placing the message _right_ first
                            let text_line = markers[i];
                            let text_pos = annot.range.0 + annot.range.1 + 1;
                            if rendered[text_line].try_insert_draw_at(text_pos, Draw::Text(Cow::Borrowed(msg), marker_style, msg_len)) {
                                continue;
                            }

                            // Else, try placing the message _down_
                            let text_line = markers[i] + 1;
                            let text_pos = annot.range.0;
                            if text_line >= rendered.len() {
                                rendered.push(RenderedLine::new(max_width));
                                // No need to clean it up, if this happens, the draw call is guaranteed to succeed
                            }
                            if rendered[text_line].try_insert_draw_at(text_pos, Draw::Text(Cow::Borrowed(msg), marker_style, msg_len)) {
                                continue;
                            }

                            // If _that_ fails, then attempt to place it left??
                            let text_line = markers[i];
                            let text_pos = annot.range.0.saturating_sub(msg_len).saturating_sub(1);
                            if rendered[text_line].try_insert_draw_at(text_pos, Draw::Text(Cow::Borrowed(msg), marker_style, msg_len)) {
                                continue;
                            }

                            // All is lost. Let's make this a reference for later...
                            let sextra = extra.to_string();
                            let sextra_len = sextra.len();
                            let text_line = markers[i];
                            let text_pos = annot.range.0;
                            rendered[text_line].mutate_first_marker_into_text(text_pos, sextra, sextra_len);
                            notes.push(Cow::Owned(format!("1. {msg}")));
                            extra += 1;
                        }

                        // With all that placed, build the new line
                        lines.push(AnnotatedSnippetLine { line: line.line, annots: line.annots, rendered });
                    }
                    snippets.push(AnnotatedSnippet { source_id: s.source_id, lines });
                }
                res.push(AnnotatedMessage::Main(AnnotatedMessageMain { sev: m.sev, code: m.code, notes, msg: m.msg, snippets }))
            },

            // Nothing much to be done here
            Message2D::Replace(m) => res.push(AnnotatedMessage::Replace(m)),
            Message2D::Sourceless(m) => res.push(AnnotatedMessage::Sourceless(m)),
        }
    }

    // Done
    res
}





/***** TESTS *****/
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rendered_line_consolidate_free_space() {
        let style = Style::new();

        let mut line = RenderedLine { draw: Vec::new() };
        line.consolidate_free_space();
        assert_eq!(line.draw, Vec::new());

        let mut line = RenderedLine { draw: vec![Draw::Empty(42)] };
        line.consolidate_free_space();
        assert_eq!(line.draw, vec![Draw::Empty(42)]);

        let mut line = RenderedLine { draw: vec![Draw::Empty(21), Draw::Empty(21)] };
        line.consolidate_free_space();
        assert_eq!(line.draw, vec![Draw::Empty(42)]);

        let mut line = RenderedLine { draw: vec![Draw::Empty(20), Draw::Empty(20), Draw::Empty(2)] };
        line.consolidate_free_space();
        assert_eq!(line.draw, vec![Draw::Empty(42)]);

        let mut line =
            RenderedLine { draw: vec![Draw::Empty(21), Draw::Empty(21), Draw::Text(Cow::Borrowed("Hello, world!"), &style, 13), Draw::Empty(2)] };
        line.consolidate_free_space();
        assert_eq!(line.draw, vec![Draw::Empty(42), Draw::Text(Cow::Borrowed("Hello, world!"), &style, 13), Draw::Empty(2)]);

        let mut line = RenderedLine {
            draw: vec![Draw::Empty(21), Draw::Empty(21), Draw::Text(Cow::Borrowed("Hello, world!"), &style, 13), Draw::Empty(2), Draw::Empty(40)],
        };
        line.consolidate_free_space();
        assert_eq!(line.draw, vec![Draw::Empty(42), Draw::Text(Cow::Borrowed("Hello, world!"), &style, 13), Draw::Empty(42)]);
    }

    #[test]
    fn test_rendered_line_try_insert_marker_at() {
        let style = Style::new();

        let mut line = RenderedLine { draw: vec![Draw::Empty(100)] };
        line.try_insert_draw_at(0, Draw::Marker('-', &style, 25));
        assert_eq!(line.draw, vec![Draw::Marker('-', &style, 25), Draw::Empty(75)]);

        let mut line = RenderedLine { draw: vec![Draw::Empty(100)] };
        line.try_insert_draw_at(75, Draw::Marker('-', &style, 25));
        assert_eq!(line.draw, vec![Draw::Empty(75), Draw::Marker('-', &style, 25)]);

        let mut line = RenderedLine { draw: vec![Draw::Empty(100)] };
        line.try_insert_draw_at(50, Draw::Marker('-', &style, 25));
        assert_eq!(line.draw, vec![Draw::Empty(50), Draw::Marker('-', &style, 25), Draw::Empty(25)]);

        let mut line = RenderedLine { draw: vec![Draw::Empty(10)] };
        assert!(!line.try_insert_draw_at(0, Draw::Marker('-', &style, 25)));
        let mut line = RenderedLine { draw: vec![Draw::Empty(20), Draw::Marker('-', &style, 25), Draw::Empty(20)] };
        assert!(!line.try_insert_draw_at(0, Draw::Marker('-', &style, 25)));

        let mut line = RenderedLine { draw: vec![Draw::Empty(20), Draw::Marker('-', &style, 25), Draw::Empty(45)] };
        line.try_insert_draw_at(50, Draw::Marker('-', &style, 25));
        assert_eq!(line.draw, vec![Draw::Empty(20), Draw::Marker('-', &style, 25), Draw::Empty(5), Draw::Marker('-', &style, 25), Draw::Empty(15)]);
    }

    #[test]
    fn test_rendered_line_mutate_first_marker_into_text() {
        let style = Style::new();

        let mut line = RenderedLine { draw: vec![Draw::Marker('-', &style, 25)] };
        line.mutate_first_marker_into_text(0, "1", 1);
        assert_eq!(line.draw, vec![Draw::Text(Cow::Owned("1".into()), &style, 1), Draw::Marker('-', &style, 24)]);

        let mut line = RenderedLine { draw: vec![Draw::Marker('-', &style, 25)] };
        line.mutate_first_marker_into_text(0, "10", 2);
        assert_eq!(line.draw, vec![Draw::Text(Cow::Owned("10".into()), &style, 2), Draw::Marker('-', &style, 23)]);

        let mut line = RenderedLine { draw: vec![Draw::Marker('-', &style, 25), Draw::Empty(25), Draw::Marker('-', &style, 25)] };
        line.mutate_first_marker_into_text(50, "1", 1);
        assert_eq!(line.draw, vec![
            Draw::Marker('-', &style, 25),
            Draw::Empty(25),
            Draw::Text(Cow::Owned("1".into()), &style, 1),
            Draw::Marker('-', &style, 24)
        ]);
    }
}
