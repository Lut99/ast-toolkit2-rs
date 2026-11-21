//  MESSAGES.rs
//    by Lut99
//
//  Description:
//!   Implements the iterator yielding [`Message`]s.
//

use super::super::annots::Severity;
use super::super::diag::Diag;
use crate::loc::Range;


/***** AST *****/
/// Defines all possible messages that we will render.
pub(super) enum Message<'d, E> {
    /// A "standard" message containing a main message and one or more annotations, but with a
    /// source text.
    Main(MessageMain<'d>),
    /// A message with a source text but also that replaces source text.
    Replace(MessageReplace<'d, E>),
    /// A message without any source text.
    Sourceless(MessageSourceless<'d>),
}



/// A "main" message contains the original error message and most annotations.
#[derive(Clone, Debug)]
pub struct MessageMain<'d> {
    /// The severity of the message.
    pub sev:      Severity,
    /// Any code in the message.
    pub code:     Option<&'d str>,
    /// The message of the message.
    pub msg:      &'d str,
    /// The notes in the message
    pub notes:    &'d [String],
    /// The snippets to show for this message.
    pub snippets: Vec<Snippet<'d>>,
}

/// A group of annotations beloning to the (same area of the) same source.
#[derive(Clone, Debug)]
pub struct Snippet<'d> {
    /// The source ID of the snippet to show.
    pub source_id: u64,
    /// A single snippet to show. It has one or more annotations to show within it.
    pub annots:    Vec<Annot<'d>>,
}

/// A single highlighting of source text.
#[derive(Clone, Copy, Debug)]
pub struct Annot<'d> {
    /// The message of this annotation, if any.
    pub msg:   Option<&'d str>,
    /// The severity of this annotation.
    pub sev:   Severity,
    /// The range of this annotation.
    pub range: Range,
}
impl<'d> Annot<'d> {
    /// Returns a set of LineAnnots that represent this annotation split over multiple lines.
    ///
    /// # Arguments
    /// - `ranges`: The new ranges that this annotation will be split over. Note that it is your
    ///   responsibility to ensure these are correct and do not exceed the original annot's
    ///   `range`.
    #[track_caller]
    pub fn split_over_lines(self, ranges: impl IntoIterator<Item = (usize, usize)>) -> Vec<LineAnnot<'d>> {
        // Get the first to carry the message
        let mut ranges = ranges.into_iter();
        let size_hint = ranges.size_hint();
        let (first_pos, first_len) = ranges.next().unwrap_or_else(|| panic!("You must give at least one range to Annot::split_over_lines()"));
        let mut res = Vec::with_capacity(size_hint.1.unwrap_or(size_hint.0));
        res.push(LineAnnot { msg: self.msg, sev: self.sev, range: (first_pos, first_len) });

        // The rest does not, only highlights
        for (rem_pos, rem_len) in ranges {
            res.push(LineAnnot { msg: None, sev: self.sev, range: (rem_pos, rem_len) });
        }
        res
    }
}

/// A single [`Annot`]atation scoped to a single line.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub struct LineAnnot<'d> {
    /// The message of this annotation, if any.
    pub msg:   Option<&'d str>,
    /// The severity of this annotation.
    pub sev:   Severity,
    /// The range of this annotation, relative to a line.
    pub range: (usize, usize),
}



/// A "replace" message contains a single annotation showing a suggested replacement.
#[derive(Clone, Copy, Debug)]
pub struct MessageReplace<'d, E> {
    /// The source ID of the (only) snippet to show.
    pub source_id: u64,
    /// The severity of the message/(only) annotation.
    pub sev: Severity,
    /// The message of the message/(only) annotation.
    pub msg: &'d str,
    /// The replacement to make.
    pub repl: &'d [E],
    /// The range of the (only) annotation.
    pub range: Range,
}



/// A "sourceless" message has no annotations whatsoever and, therefore, no source text.
#[derive(Clone, Copy, Debug)]
pub struct MessageSourceless<'d> {
    /// The severity of the message.
    pub sev: Severity,
    /// The message of the message.
    pub msg: &'d str,
}





/***** LIBRARY *****/
/// Takes a [`Diag`] and outputs a set of [`Message`]s.
pub(super) fn as_messages<'d, E>(diag: &'d Diag<E>) -> std::vec::IntoIter<Message<'d, E>> {
    // Create the main snippet
    let mut main = MessageMain {
        sev:      diag.sev,
        code:     diag.code.as_ref().map(String::as_str),
        msg:      diag.msg.as_str(),
        notes:    diag.notes.as_slice(),
        snippets: Vec::new(),
    };

    // Go through all annotations to amend the main message/cook up more messages
    let mut messages = Vec::<Message<E>>::new();
    'annots: for annot in &diag.annots {
        // Is it sourceless?
        let Some(source_id) = annot.loc.source else {
            // It is, create a dedicated message for it.
            messages.push(Message::Sourceless(MessageSourceless {
                sev: annot.sev,
                msg: annot.msg.as_ref().map(String::as_str).unwrap_or("<no error message>"),
            }));
            continue 'annots;
        };

        // Does it replace?
        if let Some(repl) = annot.repl.as_ref().map(Vec::as_slice) {
            // It is, create a dedicated message for it
            messages.push(Message::Replace(MessageReplace {
                source_id,
                sev: annot.sev,
                msg: annot.msg.as_ref().map(String::as_str).unwrap_or("Consider this instead"),
                repl,
                range: annot.loc.range,
            }));
            continue 'annots;
        }

        // Else, see if we can add this annotation to an existing snippet in main
        let annot = Annot { msg: annot.msg.as_ref().map(String::as_str), sev: annot.sev, range: annot.loc.range };
        for snippet in &mut main.snippets {
            if snippet.source_id == source_id {
                snippet.annots.push(annot);
                continue 'annots;
            }
        }

        // Else else, add a new snippet
        main.snippets.push(Snippet { source_id, annots: vec![annot] });
    }

    // OK, done
    messages.insert(0, Message::Main(main));
    messages.into_iter()
}
