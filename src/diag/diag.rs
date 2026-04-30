//  DIAG.rs
//    by Lut99
//
//  Description:
//!   Implements the [`Diag`]nostic type that abstracts over individual errors
//!   but is renderable.
//

use super::Diagnostic;
use super::annotations::{Annotation, Severity};
use super::themes::Theme;


/***** LIBRARY *****/
/// All the information from an error we need to render a useful diagnostic.
///
/// Usually, you create this in a type's
/// [`Diagnostic::into_diag()`](super::Diagnostic::into_diag())-implementation. However, you can
/// also call [`Diag::from_diagnostic()`] on such types (no worries).
///
/// If you enabled `proc-macros`, then likely, you never have interact with this type yourself. Use
/// the [`Diagnostic`](super::Diagnostic) derive macro instead.
#[derive(Clone, Debug)]
pub struct Diag {
    /// The style to apply while rendering.
    pub theme: Theme,

    /// The main severity of the Diagnostic.
    pub sev:  Severity,
    /// Any code to display with the main message.
    pub code: Option<String>,
    /// The main message of this Diagnostic.
    pub msg:  String,

    /// The annotations slapped onto this Diagnostic.
    pub annots: Vec<Annotation>,
}

// Constructors
impl Diag {
    /// Constructs a new Diag that has the given severity and the given message.
    ///
    /// Note that any other settings, like theme, are set to the default (e.g., [`Theme::PLAIN`]).
    ///
    /// # Returns
    /// A new Diag that can be further customized.
    #[inline]
    pub const fn new(sev: Severity, msg: String) -> Self { Self { theme: Theme::PLAIN, sev, code: None, msg, annots: Vec::new() } }

    /// Constructs a new Diag that shows as an error with the given message.
    ///
    /// Note that any other settings, like theme, are set to the default (e.g., [`Theme::PLAIN`]).
    ///
    /// # Returns
    /// A new Diag that can be further customized.
    #[inline]
    pub const fn error(msg: String) -> Self { Self::new(Severity::Error, msg) }

    /// Constructs a new Diag that shows as a hint with the given message.
    ///
    /// Note that any other settings, like theme, are set to the default (e.g., [`Theme::PLAIN`]).
    ///
    /// # Returns
    /// A new Diag that can be further customized.
    #[inline]
    pub const fn help(msg: String) -> Self { Self::new(Severity::Help, msg) }

    /// Constructs a new Diag that shows as a suggestion with the given message.
    ///
    /// Note that any other settings, like theme, are set to the default (e.g., [`Theme::PLAIN`]).
    ///
    /// # Returns

    /// A new Diag that can be further customized.
    #[inline]
    pub const fn suggestion(msg: String) -> Self { Self::new(Severity::Suggestion, msg) }

    /// Constructs a new Diag that shows as a warning with the given message.
    ///
    /// Note that any other settings, like theme, are set to the default (e.g., [`Theme::PLAIN`]).
    ///
    /// # Returns
    /// A new Diag that can be further customized.
    #[inline]
    pub const fn warning(msg: String) -> Self { Self::new(Severity::Warning, msg) }

    /// Construct a new Diag by letting another type build it for us.
    ///
    /// # Arguments
    /// - `ty`: Some type to turn into a Diag.
    ///
    /// # Returns
    /// A new Diag as described by the type.
    #[inline]
    pub fn from_diagnostic<T: Diagnostic>(diag: T) -> Self { diag.into_diag() }
}

// Factory methods
impl Diag {
    /// Consumes this Diag to create an equal one but with the given theme.
    ///
    /// To undo the effects of this function, call it again with the [`Theme::PLAIN`] theme.
    ///
    /// # Arguments
    /// - `theme`: The new [`Theme`] to use instead.
    ///
    /// # Returns
    /// The same Diag but with the given `theme`.
    #[inline]
    pub const fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }


    /// Consumes this Diag to create an equal one but with a given code.
    ///
    /// Codes are given as prefixes to messages to help users find them online.
    ///
    /// # Arguments
    /// - `code`: Some string describing the code itself.
    ///
    /// # Returns
    /// The same Diag but with the given `code`.
    #[inline]
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Consumes this Diag to create an equal one but without a given code.
    ///
    /// This can be used to undo the effects of [`Diag::with_code()`].
    ///
    /// # Returns
    /// The same Diag but without a given code.
    #[inline]
    pub fn without_code(mut self) -> Self {
        self.code = None;
        self
    }


    /// Reserves space for a given number of annotations in this Diag.
    ///
    /// This can be used to prepare adding multiple annotations with [`Diag::with_annot`] cheaper.
    ///
    /// # Arguments
    /// - `additional`: The minimum number of _additional_ annotations to prepare capacity for. This
    ///   is on top of any existing annotations. Also note that the backing allocator may decide to
    ///   reserve for more if it likes.
    ///
    /// # Returns
    /// The same Diag but with capacity for at least `additional` additional [`Annotations`].
    #[inline]
    pub fn reserve_annots(mut self, additional: usize) -> Self {
        self.annots.reserve(additional);
        self
    }

    /// Adds a new [`Annotation`] to this Diag.
    ///
    /// To remove it again, manually filter it from the [`Diag::annots`] vector.
    ///
    /// # Arguments
    /// - `annot`: The [`Annotation`] to add to this Diag.
    ///
    /// # Returns
    /// The same Diag but with the given `annot`ation added.
    #[inline]
    pub fn with_annot(mut self, annot: Annotation) -> Self {
        self.annots.push(annot);
        self
    }
}

// Rendering

// Uniformity
impl Diagnostic for Diag {
    #[inline]
    fn into_diag(self) -> Diag { self }
}
