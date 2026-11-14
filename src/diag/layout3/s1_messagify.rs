//  STEP 1 - MESSAGIFY.rs
//    by Lut99
//
//  Description:
//!   Defines the first step in the [layouting algorithm](super).
//

use super::super::{Diag, Severity};
use crate::loc::Range;


/***** HELPERS *****/
/// Defines all possible messages that we will render.
pub enum Message<'d, E> {
    /// A "standard" message containing a main message and one or more annotations, but with a
    /// source text.
    Main(MessageMain<'d>),
    /// A message with a source text but also that replaces source text.
    Replace(MessageReplace<'d, E>),
    /// A message without any source text.
    Sourceless(MessageSourceless<'d>),
}



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

pub struct Snippet<'d> {
    /// The source ID of the snippet to show.
    pub source_id: u64,
    /// A single snippet to show. It has one or more annotations to show within it.
    pub annots:    Vec<Annot<'d>>,
}

pub struct Annot<'d> {
    /// The message of this annotation, if any.
    pub msg:   Option<&'d str>,
    /// The severity of this annotation.
    pub sev:   Severity,
    /// The range of this annotation.
    pub range: Range,
}



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



pub struct MessageSourceless<'d> {
    /// The severity of the message.
    pub sev: Severity,
    /// The message of the message.
    pub msg: &'d str,
}





/***** LIBRARY *****/
/// Runs the first step in the [layouting algorithm](super).
///
/// # Arguments
/// - `diag`: The [`Diag`] to start processing.
///
/// # Returns
/// A collection of [`Snippet`]s, each of which representing a coherent error message to show.
pub fn s1_messagify<'d, E>(diag: &'d Diag<E>) -> Vec<Message<'d, E>> {
    // Create the main snippet
    let mut main = MessageMain {
        sev:      diag.sev,
        code:     diag.code.as_ref().map(String::as_str),
        msg:      diag.msg.as_str(),
        notes:    diag.notes.as_slice(),
        snippets: Vec::new(),
    };

    // Go through all annotations to amend the main message/cook up more messages
    let mut messages = Vec::<Message<'d, E>>::new();
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
    messages
}
