//  DIAGNOSTICS.rs
//    by Lut99
//
//  Description:
//!   A module that provides the [`Diagnostic`]-trait, which is something that
//!   can be rendered into pretty, verbose and annotated errors.
//!
//!   This is what the crate is all about! Originally...
//

// Declare the nested modules
mod annotations;
mod diag;
// mod layout;
mod themes;

// Use 'em
pub use diag::Diag;


/***** SPECIFICATIONS *****/
/// The trait which it is all about; the Diagnostic!
///
/// Note that it is rather simplistic, as the complexity is on a case-by-case basis that builds a
/// [`Diag`] for every error.
///
/// Luckily you won't have to write it yourself. Simply enable the `proc-macros`-feature and use
/// the derive trait!
pub trait Diagnostic {
    /// Returns a fresh [`Diag`] from this Diagnostic.
    ///
    /// The [`Diag`] is eventually what is being rendered to the user, with all its information and
    /// annotations.
    ///
    /// You rarely have to call- or implement this function yourself (the latter is if you have
    /// `proc-macros` enabled).
    ///
    /// # Returns
    /// A fresh [`Diag`] that represents the diagnostic information of this type condensed down to
    /// something we can render.
    fn into_diag(self) -> Diag;
}
