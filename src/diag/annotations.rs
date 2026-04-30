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





/***** LIBRARY *****/
/// Defines annotations that can be given in a snippet.
#[derive(Clone, Debug)]
pub struct Annotation {
    /// Defines any visual suggested replacement for the underlying `S`ource text.
    pub replacement: Option<String>,
    /// Defines any message to give with this annotation.
    pub message: Option<String>,
    /// The severity (= color and markers like "error" or "help") of this annotation.
    pub severity: Severity,
    /// Defines the place in the source describing what this annotation highlights.
    pub loc: Loc,
}

// Loc
impl Located for Annotation {
    #[inline]
    fn loc(&self) -> Loc { self.loc }
}
