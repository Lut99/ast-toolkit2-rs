//  STEP 2 - LAYOUTING.rs
//    by Lut99
//
//  Description:
//!   Defines the second step in the [layouting algorithm](super).
//

use super::super::Severity;
use super::layouter::Layouter;
use super::s1_messagify::{Message, MessageSourceless};


/***** HELPERS *****/
/// Defines all possible messages that we will render.
pub enum Message2D<'d, E> {
    /// A "standard" message containing a main message and one or more annotations, but with a
    /// source text.
    Main(MessageMain2D<'d>),
    /// A message with a source text but also that replaces source text.
    Replace(MessageReplace2D<'d, E>),
    /// A message without any source text.
    Sourceless(MessageSourceless<'d>),
}



pub struct MessageMain2D<'d> {
    /// The severity of the message.
    pub sev:      Severity,
    /// Any code in the message.
    pub code:     Option<&'d str>,
    /// The message of the message.
    pub msg:      &'d str,
    /// The notes in the message.
    pub notes:    &'d [String],
    /// The snippets to show for this message.
    pub snippets: Vec<Snippet2D<'d>>,
}

pub struct Snippet2D<'d> {
    /// The source ID of the snippet to show.
    pub source_id: u64,
    /// A single snippet to show. It has one or more annotations to show within it.
    pub lines:     Vec<SnippetLine<'d>>,
}

pub struct SnippetLine<'d> {
    /// The line number.
    pub line:   u64,
    /// The annotations on this line.
    pub annots: Vec<Annot2D<'d>>,
}

pub struct Annot2D<'d> {
    /// The message of this annotation, if any.
    pub msg:   Option<&'d str>,
    /// The severity of this annotation.
    pub sev:   Severity,
    /// The range of this annotation, local to the line we're in.
    pub range: (usize, usize),
}



pub struct MessageReplace2D<'d, E> {
    /// The source ID of the (only) snippet to show.
    pub source_id: u64,
    /// The severity of the message/(only) annotation.
    pub sev: Severity,
    /// The message of the message/(only) annotation.
    pub msg: &'d str,
    /// The replacement to make.
    pub repl: &'d [E],
    /// The lines in this annotation.
    pub lines: Vec<ReplaceLine>,
    /// The original position of the replacement.
    pub pos: u64,
}

pub struct ReplaceLine {
    /// The line number.
    pub line:  u64,
    /// The annotation on this line.
    pub range: (usize, usize),
}





/***** LIBRARY *****/
/// Runs the second step in the [layouting algorithm](super).
///
/// # Arguments
/// - `layouter`: A [`Layouter`] capable of turning 1D ranges into 2D.
/// - `msgs`: A list of [`Message`]s resulting from the first step to process.
///
/// # Returns
/// A collection of [`Message2D`]s, each of which has its ranges turned into 2D positions.
#[track_caller]
pub fn s2_layout<'d, L: Layouter<E>, E>(layouter: &L, msgs: Vec<Message<'d, E>>) -> Vec<Message2D<'d, E>> {
    // Go through each message to translate it
    let mut res = Vec::new();
    for msg in msgs {
        match msg {
            Message::Main(m) => {
                // Do a replacement within each snippet
                res.push(Message2D::Main(MessageMain2D {
                    sev:      m.sev,
                    code:     m.code,
                    msg:      m.msg,
                    notes:    m.notes,
                    snippets: m
                        .snippets
                        .into_iter()
                        .map(|s| {
                            // Compute the 2D ranges for all annotations
                            let ranges = layouter.range_to_2d(s.source_id, s.annots.iter().map(|a| a.range));
                            #[cfg(debug_assertions)]
                            if ranges.len() != s.annots.len() {
                                panic!("{}::range_to_2d() gave a different number of `Range2D`s than `Range`s given", std::any::type_name::<L>())
                            }

                            // Order them by-line
                            let mut lines = Vec::<SnippetLine<'d>>::new();
                            for (annot, range) in s.annots.into_iter().zip(ranges) {
                                fn insert_at<'d>(lines: &mut Vec<SnippetLine<'d>>, l: u64, annot: Annot2D<'d>) {
                                    for line in &mut *lines {
                                        if line.line == l {
                                            line.annots.push(annot);
                                            return;
                                        }
                                    }
                                    lines.push(SnippetLine { line: l, annots: vec![annot] });
                                }

                                // The first gets inserted with the message
                                let mut iter = range.lines.into_iter();
                                let first = iter
                                    .next()
                                    .unwrap_or_else(|| panic!("{}::range_to_2d() gave zero lines for a Range2D", std::any::type_name::<L>()));
                                insert_at(&mut lines, first.line, Annot2D { msg: annot.msg, sev: annot.sev, range: (first.pos, first.len) });

                                // The rest without
                                for line in iter {
                                    insert_at(&mut lines, line.line, Annot2D { msg: None, sev: annot.sev, range: (line.pos, line.len) });
                                }
                            }

                            // Build the snippet
                            Snippet2D { source_id: s.source_id, lines }
                        })
                        .collect(),
                }))
            },

            Message::Replace(m) => {
                // Simply do a one-range replacement
                let ranges = layouter.range_to_2d(m.source_id, [m.range]);
                #[cfg(debug_assertions)]
                if ranges.len() != 1 {
                    panic!("{}::range_to_2d() gave more than one `Range2D` for a single `Range` given", std::any::type_name::<L>())
                }
                let range = ranges.into_iter().next().unwrap();

                // Turn that into the lines
                res.push(Message2D::Replace(MessageReplace2D {
                    source_id: m.source_id,
                    sev: m.sev,
                    msg: m.msg,
                    repl: m.repl,
                    lines: range.lines.into_iter().map(|l| ReplaceLine { line: l.line, range: (l.pos, l.len) }).collect(),
                    pos: m.range.pos,
                }))
            },

            // Sourceless messaging need nothing done, no ranges
            Message::Sourceless(m) => res.push(Message2D::Sourceless(m)),
        }
    }

    // Done
    res
}
