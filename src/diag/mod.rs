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
// pub mod layout3;
pub mod layout4;
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
    use layout4::renderers::{HexRenderer, Utf8Renderer};

    use super::*;
    #[cfg(feature = "sources")]
    use crate::{loc::Loc, sources::Sources};

    #[cfg(feature = "sources")]
    #[test]
    fn test_diag_display_utf8() {
        let mut sources = Sources::new();
        let id1 = sources.add_bytes("example1", b"Hello, world!");
        let id2 = sources.add_bytes("example2", "Hello, world!\nHallo, wereld!\nGuten tag, werld!\nHej världen!\nBonjour le monde!\n".as_bytes());

        let diag = Diag::<u8>::error("Oh no, something went wrong!")
            .with_code("E000")
            .with_annot(Annotation::error(Loc::encapsulate_range(id1, ..13)))
            .with_annot(Annotation::suggestion(Loc::encapsulate_range(id1, ..5)).with_replace(b"Goodbye"))
            .with_note("Remember this");
        let sdiag = diag.display(Utf8Renderer::new(100, &sources)).to_string();
        println!("{sdiag}");
        assert_eq!(
            sdiag,
            r#"error[E000]: Oh no, something went wrong!
 --> example1:1:1
  |
1 | Hello, world!
  | -------------
  |
  = note: Remember this
suggestion: Consider this instead
 --> example1:1:1
  |
1 - Hello, world!
1 + Goodbye, world!
  |
"#
        );

        let diag = Diag::<u8>::error("Two seperate lines are the problem")
            .with_annot(Annotation::error(Loc::encapsulate_range(id2, ..13)))
            .with_annot(Annotation::error(Loc::encapsulate_range(id2, 29..46)));
        let sdiag = diag.display(Utf8Renderer::new(100, &sources)).to_string();
        println!("{sdiag}");
        assert_eq!(
            sdiag,
            r#"error: Two seperate lines are the problem
 --> example2:1:1
  |
1 | Hello, world!
  | -------------
2 | Hallo, wereld!
3 | Guten tag, werld!
  | -----------------
  |
"#
        );

        let diag = Diag::<u8>::error("Two, more seperate lines are the problem")
            .with_annot(Annotation::error(Loc::encapsulate_range(id2, ..13)))
            .with_annot(Annotation::error(Loc::encapsulate_range(id2, 47..59)));
        let sdiag = diag.display(Utf8Renderer::new(100, &sources)).to_string();
        println!("{sdiag}");
        assert_eq!(
            sdiag,
            r#"error: Two, more seperate lines are the problem
 --> example2:1:1
  |
1 | Hello, world!
  | -------------
2 | Hallo, wereld!
3 | Guten tag, werld!
4 | Hej världen!
  | ------------
  |
"#
        );

        let diag = Diag::<u8>::error("Two, way more seperate lines are the problem")
            .with_annot(Annotation::error(Loc::encapsulate_range(id2, ..13)))
            .with_annot(Annotation::error(Loc::encapsulate_range(id2, 60..77)));
        let sdiag = diag.display(Utf8Renderer::new(100, &sources)).to_string();
        println!("{sdiag}");
        assert_eq!(
            sdiag,
            r#"error: Two, way more seperate lines are the problem
 --> example2:1:1
  |
1 | Hello, world!
  | -------------
...
5 | Bonjour le monde!
  | -----------------
  |
"#
        );
    }

    #[cfg(feature = "sources")]
    #[test]
    fn test_diag_display_hex() {
        let mut sources = Sources::new();
        let id1 = sources.add_bytes("example1", b"Hello, world!");
        let id2 = sources.add_bytes("example2", b"01234567890123456789012345678901234567890123456789012345678901234567890123456789");

        let diag = Diag::<u8>::error("Oh no, something went wrong!")
            .with_code("E000")
            .with_annot(Annotation::error(Loc::encapsulate_range(id1, ..13)))
            .with_annot(Annotation::suggestion(Loc::encapsulate_range(id1, ..5)).with_replace(b"Goodbye"))
            .with_note("Remember this");
        let sdiag = diag.display(HexRenderer::new(100, &sources)).to_string();
        println!("{sdiag}");
        assert_eq!(
            sdiag,
            r#"error[E000]: Oh no, something went wrong!
 --> example1 at 0x0
  |
0 | 48 65 6c 6c  6f 2c 20 77  6f 72 6c 64  21             Hello, world!   
  | -----------------------------------------             -------------
  |
  = note: Remember this
suggestion: Consider this instead
 --> example1 at 0x0
  |
0 - 48 65 6c 6c  6f 2c 20 77  6f 72 6c 64  21             Hello, world!   
0 + 47 6f 6f 64  62 79 65 2c  20 77 6f 72  6c 64 21       Goodbye, world!
  |
"#
        );

        let diag = Diag::<u8>::error("Two seperate lines are the problem")
            .with_annot(Annotation::error(Loc::encapsulate_range(id2, ..8)))
            .with_annot(Annotation::error(Loc::encapsulate_range(id2, 32..40)));
        let sdiag = diag.display(HexRenderer::new(100, &sources)).to_string();
        println!("{sdiag}");
        assert_eq!(
            sdiag,
            r#"error: Two seperate lines are the problem
 --> example2 at 0x0
  |
0 | 30 31 32 33  34 35 36 37  38 39 30 31  32 33 34 35    0123456789012345
  | ------------------------                              --------
1 | 36 37 38 39  30 31 32 33  34 35 36 37  38 39 30 31    6789012345678901
2 | 32 33 34 35  36 37 38 39  30 31 32 33  34 35 36 37    2345678901234567
  | ------------------------                              --------
  |
"#
        );

        let diag = Diag::<u8>::error("Two, more seperate lines are the problem")
            .with_annot(Annotation::error(Loc::encapsulate_range(id2, ..8)))
            .with_annot(Annotation::error(Loc::encapsulate_range(id2, 48..56)));
        let sdiag = diag.display(HexRenderer::new(100, &sources)).to_string();
        println!("{sdiag}");
        assert_eq!(
            sdiag,
            r#"error: Two, more seperate lines are the problem
 --> example2 at 0x0
  |
0 | 30 31 32 33  34 35 36 37  38 39 30 31  32 33 34 35    0123456789012345
  | ------------------------                              --------
1 | 36 37 38 39  30 31 32 33  34 35 36 37  38 39 30 31    6789012345678901
2 | 32 33 34 35  36 37 38 39  30 31 32 33  34 35 36 37    2345678901234567
3 | 38 39 30 31  32 33 34 35  36 37 38 39  30 31 32 33    8901234567890123
  | ------------------------                              --------
  |
"#
        );

        let diag = Diag::<u8>::error("Two, way more seperate lines are the problem")
            .with_annot(Annotation::error(Loc::encapsulate_range(id2, ..8)))
            .with_annot(Annotation::error(Loc::encapsulate_range(id2, 64..72)));
        let sdiag = diag.display(HexRenderer::new(100, &sources)).to_string();
        println!("{sdiag}");
        assert_eq!(
            sdiag,
            r#"error: Two, way more seperate lines are the problem
 --> example2 at 0x0
  |
0 | 30 31 32 33  34 35 36 37  38 39 30 31  32 33 34 35    0123456789012345
  | ------------------------                              --------
...
4 | 34 35 36 37  38 39 30 31  32 33 34 35  36 37 38 39    4567890123456789
  | ------------------------                              --------
  |
"#
        );
    }
}
