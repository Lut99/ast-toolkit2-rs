//  PLACEMENT.rs
//    by Lut99
//
//  Description:
//!   Implements the annotation placement algorithm.
//

use std::borrow::Cow;

use console::Style;
use unicode_segmentation::UnicodeSegmentation as _;

use super::super::super::themes::Theme;
use super::super::messages::LineAnnot;


/***** LIBRARY *****/
/// Abstracts the resulting annotation lines
pub struct RenderedLine<'d> {
    /// The sequence of things to draw.
    pub draw: Vec<Draw<'d>>,
}
impl<'d> RenderedLine<'d> {
    #[inline(always)]
    fn new(max_width: usize) -> Self { Self { draw: vec![Draw::Empty(max_width)] } }

    pub(super) fn consolidate_free_space(&mut self) {
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

    fn try_insert_draw_at(&mut self, draw_pos: usize, draw: Draw<'d>) -> bool {
        // Find the slot
        let at_end: usize = draw_pos + draw.len();
        let mut pos: usize = 0;
        for i in 0..self.draw.len() {
            if pos > draw_pos {
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

    pub(super) fn all_empty(&self) -> bool {
        for draw in &self.draw {
            if !matches!(draw, Draw::Empty(_)) {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Draw<'d> {
    /// It's an empty space of `N` cells.
    Empty(usize),
    /// It's `N` markers with the given color.
    Marker(char, &'d Style, usize),
    /// It's text of `N` cells with the given style.
    Text(Cow<'d, str>, &'d Style, usize),
}
impl<'d> Draw<'d> {
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
/// - `max_line_width`: The maximum line length.
/// - `no_extra_notes`: The number of extra notes already added, to keep the note numbering
///   consistent.
/// - `msgs`: A list of message to compute annotation placement in.
///
/// # Returns
/// A new list of messages with rendered annotations attached and a list of additional notes to attach.
pub fn place_annots<'d>(
    theme: &'d Theme,
    max_line_width: usize,
    mut no_extra_notes: usize,
    annots: impl IntoIterator<Item = LineAnnot<'d>>,
) -> (Vec<RenderedLine<'d>>, Vec<String>) {
    // Ensure the annotations are sorted by start position
    let mut annots: Vec<LineAnnot<'d>> = annots.into_iter().collect();
    annots.sort_by_key(|a| a.range.0);

    // Start by writing marker lines
    let mut markers = Vec::<usize>::with_capacity(annots.len());
    let mut rendered = vec![RenderedLine::new(max_line_width)];
    'annots: for annot in &annots {
        let (marker, marker_style) = theme.marker_style(annot.sev);
        for (l, line) in rendered.iter_mut().enumerate() {
            // See if there's place here
            if line.try_insert_draw_at(annot.range.0, Draw::Marker(*marker, marker_style, annot.range.1.max(1))) {
                markers.push(l);
                continue 'annots;
            }
        }
        // No place found, we'll have to add a new rendered line
        rendered.push({
            let mut line = RenderedLine::new(max_line_width);
            line.try_insert_draw_at(annot.range.0, Draw::Marker(*marker, marker_style, annot.range.1.max(1)));
            line
        });
        markers.push(rendered.len());
    }

    // `rendered` now has lines with markers, `markers` maps each annotation to
    // on which line its markers are placed.
    // Now it's time to crime!

    let mut notes = Vec::new();
    for (i, annot) in annots.iter().enumerate() {
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
            rendered.push(RenderedLine::new(max_line_width));
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
        let sextra = (no_extra_notes + 1).to_string();
        let sextra_len = sextra.len();
        let text_line = markers[i];
        let text_pos = annot.range.0;
        notes.push(format!("{sextra}. {msg}"));
        rendered[text_line].mutate_first_marker_into_text(text_pos, sextra, sextra_len);
        no_extra_notes += 1;
    }

    // Done
    (rendered, notes)
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
