//  ANNOTATIONS.rs
//    by Lut99
//
//  Created:
//    24 May 2024, 17:38:35
//  Last edited:
//    12 Sep 2024, 17:07:12
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines the annotations that flavour
//!   [`Diagnostic`](crate::Diagnostic)s.
//

use crate::loc::{Loc, Located};


/***** AUXILLARY *****/
/// The severity levels supported by the [`Diagnostic`].
///
/// This essentially determines some prompt and accent colouration.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Severity {
    /// Fatal errors.
    Error,
    /// Some other information auxillary to earlier diagnostics.
    Help,
    /// A suggestion to do something.
    Suggestion,
    /// Non-fatal warnings.
    Warning,
}

// Severity
impl Severity {
    /// Returns a static string name for this.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Help => "help",
            Self::Suggestion => "suggestion",
            Self::Warning => "warning",
        }
    }
}





/***** LIBRARY *****/
/// Defines annotations that can be given in a snippet.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Annotation<E> {
    /// Defines any visual suggested replacement for the underlying `S`ource text.
    pub repl: Option<Vec<E>>,
    /// Defines any message to give with this annotation.
    pub msg:  Option<String>,
    /// The severity (= color and markers like "error" or "help") of this annotation.
    pub sev:  Severity,
    /// Defines the place in the source describing what this annotation highlights.
    pub loc:  Loc,
}

// Constructors
impl<E> Annotation<E> {
    /// Creates a new annotation.
    ///
    /// # Arguments
    /// - `sev`: A [`Severity`] describing the type of annotation.
    /// - `loc`: A [`Loc`] describing the position in the source text that this annotation
    ///   annotates.
    ///
    /// # Returns
    /// A new Annotation ready to annotate.
    #[inline]
    pub const fn new(sev: Severity, loc: Loc) -> Self { Self { repl: None, msg: None, sev, loc } }

    /// Creates a new annotation describing an error.
    ///
    /// If you want to add a message separate from the main message, call
    /// [`Annotation::with_msg()`].
    ///
    /// # Arguments
    /// - `loc`: A [`Loc`] describing the position in the source text that this annotation
    ///   annotates.
    ///
    /// # Returns
    /// A new Annotation ready to annotate.
    #[inline]
    pub fn error(loc: impl Located) -> Self { Self::new(Severity::Error, loc.loc()) }

    /// Creates a new annotation describing a hint.
    ///
    /// If you want to add a message separate from the main message, call
    /// [`Annotation::with_msg()`].
    ///
    /// # Arguments
    /// - `msg`: A message to pair
    /// - `loc`: A [`Loc`] describing the position in the source text that this annotation
    ///   annotates.
    ///
    /// # Returns
    /// A new Annotation ready to annotate.
    #[inline]
    pub fn help(loc: impl Located) -> Self { Self::new(Severity::Help, loc.loc()) }

    /// Creates a new annotation describing a suggestion.
    ///
    /// If you want to add a message separate from the main message, call
    /// [`Annotation::with_msg()`]. Similarly, a suggested replacement can be added with
    /// [`Annotation::with_repl()`].
    ///
    /// # Arguments
    /// - `loc`: A [`Loc`] describing the position in the source text that this annotation
    ///   annotates.
    ///
    /// # Returns
    /// A new Annotation ready to annotate.
    #[inline]
    pub fn suggestion(loc: impl Located) -> Self { Self::new(Severity::Suggestion, loc.loc()) }

    /// Creates a new annotation describing a warning.
    ///
    /// If you want to add a message separate from the main message, call
    /// [`Annotation::with_msg()`].
    ///
    /// # Arguments
    /// - `loc`: A [`Loc`] describing the position in the source text that this annotation
    ///   annotates.
    ///
    /// # Returns
    /// A new Annotation ready to annotate.
    #[inline]
    pub fn warning(loc: impl Located) -> Self { Self::new(Severity::Warning, loc.loc()) }
}

// Annotation
impl<E> Annotation<E> {
    /// Attaches a replacement to this Annotation.
    ///
    /// # Arguments
    /// - `what`: What to suggest to replace the range with.
    ///
    /// # Returns
    /// Self but with an annotation.
    pub fn with_replace(mut self, what: impl Into<Vec<E>>) -> Self {
        self.repl = Some(what.into());
        self
    }
}

// Loc
impl<E> Located for Annotation<E> {
    #[inline]
    fn loc(&self) -> Loc { self.loc }
}
