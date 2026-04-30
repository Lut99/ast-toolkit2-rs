//  DERIVE DIAGNOSTIC.rs
//    by Lut99
//
//  Description:
//!   Showcases the use of the [`Diagnostic`](derive@Diagnostic)-macro.
//

use ast_toolkit2::diag::{Annotation, Diag, Diagnostic, Severity, Theme};
use ast_toolkit2::loc::Loc;


#[test]
fn test_derive_diagnostic_structs() {
    #[derive(Diagnostic)]
    #[diag(error, msg = "Foo is not doing so well...", loc)]
    struct Foo {
        loc: Loc,
    }

    #[derive(Diagnostic)]
    #[diag(error, msg = "Bar is not doing so well...")]
    #[annot(error, loc)]
    struct Bar {
        loc: Loc,
    }

    #[derive(Diagnostic)]
    #[diag(help, msg = "Baz is not doing so well...", loc)]
    #[annot(error, msg = "...and here's why!", loc = reason)]
    struct Baz {
        loc:    Loc,
        reason: Loc,
    }


    const ID: u64 = 0;
    assert_eq!(Foo { loc: Loc::encapsulate(ID) }.into_diag(), Diag {
        theme:  Theme::PLAIN,
        sev:    Severity::Error,
        code:   None,
        msg:    "Foo is not doing so well...".into(),
        annots: vec![Annotation { repl: None, msg: None, sev: Severity::Error, loc: Loc::encapsulate(ID) }],
    });
    assert_eq!(Bar { loc: Loc::encapsulate(ID) }.into_diag(), Diag {
        theme:  Theme::PLAIN,
        sev:    Severity::Error,
        code:   None,
        msg:    "Bar is not doing so well...".into(),
        annots: vec![Annotation { repl: None, msg: None, sev: Severity::Error, loc: Loc::encapsulate(ID) }],
    });
}
