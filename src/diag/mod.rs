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
mod annots;
mod diag;
// pub mod layout;
// pub mod layout2;
pub mod layout3;
mod themes;

// Use 'em
pub use annots::{Annotation, Severity};
#[cfg(feature = "proc-macros")]
pub use ast_toolkit2_proc_macros::Diagnostic;
pub use diag::Diag;
// pub use layout::{RenderedDiag, Renderer};
pub use themes::Theme;


/***** SPECIFICATIONS *****/
/// The trait which it is all about; the Diagnostic!
///
/// Note that it is rather simplistic, as the complexity is on a case-by-case basis that builds a
/// [`Diag`] for every error.
///
/// Luckily you won't have to write it yourself. Simply enable the `proc-macros`-feature and use
/// the derive trait!
pub trait Diagnostic<E> {
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
    fn into_diag(self) -> Diag<E>;
}





/***** TESTS *****/
#[cfg(test)]
mod tests {
    #[cfg(feature = "sources")]
    use layout3::layouter::HexLayouter;

    use super::*;
    #[cfg(feature = "sources")]
    use crate::{loc::Loc, sources::Sources};

    #[cfg(feature = "sources")]
    #[test]
    fn test_diag_display() {
        let mut sources = Sources::new();
        let id = sources.add_bytes("example", b"Hello, world!");

        let diag = Diag::<u8>::error("Oh no, something went wrong!")
            .with_annot(Annotation::suggestion(Loc::encapsulate_range(id, ..5)).with_replace(b"Goodbye"));
        assert_eq!(
            diag.display(HexLayouter::new(100, &sources)).to_string(),
            r#"suggestion: Consider this instead
 --> example at 0
  |
"#
        );
    }
}
