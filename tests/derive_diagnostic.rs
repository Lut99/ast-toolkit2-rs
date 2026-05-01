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
    // Simple syntax sugar case
    #[derive(Diagnostic)]
    #[diag(error, msg = "SimpleSyntaxSugar is not doing so well...", loc)]
    struct SimpleSyntaxSugar {
        loc: Loc,
    }

    // Simple verbose case
    #[derive(Diagnostic)]
    #[diag(error, msg = "SimpleVerbose is not doing so well...")]
    #[annot(error, loc)]
    struct SimpleVerbose {
        loc: Loc,
    }

    // Additional annotation
    #[derive(Diagnostic)]
    #[diag(help, msg = "ExtraAnnot is not doing so well...", loc)]
    #[annot(error, msg = "...and here's why!", loc = reason)]
    struct ExtraAnnot {
        loc:    Loc,
        reason: Loc,
    }

    // Main message with code
    #[derive(Diagnostic)]
    #[diag(warn, code = "W001", msg = "WithCode is not doing so well...")]
    struct WithCode {
        loc: Loc,
    }

    // Many annotations with replacement
    #[derive(Diagnostic)]
    #[diag(error, msg = "Some error occurred!", loc)]
    #[annot(error, msg = "...because of this error", loc = reason1)]
    #[annot(error, msg = "...and this error", loc = reason2)]
    #[annot(error, msg = "Try doing this instead!", repl = "foo", loc = reason3)]
    struct ManyAnnotsWithRepl {
        loc:     Loc,
        reason1: Loc,
        reason2: Loc,
        reason3: Loc,
    }

    // Complex formatting
    #[derive(Diagnostic)]
    #[diag(error, code = ("E{}", code), msg = ("Hello, {}", world))]
    #[annot(warning, repl = ("{foo}"), msg = ("Goodbye, {:?}", world), loc)]
    struct ComplexFormat {
        code:  u32,
        world: &'static str,
        foo:   &'static str,
        loc:   Loc,
    }

    // Tuple formatting
    #[derive(Diagnostic)]
    #[diag(error, code = ("E{f0}"), msg = ("Hello, {f1}"))]
    #[annot(warning, repl = ("{f2}"), msg = ("Goodbye, {f1:?}"), loc = f3)]
    struct TupleFormat(u32, &'static str, &'static str, Loc);


    const ID: u64 = 0;
    assert_eq!(SimpleSyntaxSugar { loc: Loc::encapsulate(ID) }.into_diag(), Diag {
        theme:  Theme::PLAIN,
        sev:    Severity::Error,
        code:   None,
        msg:    "SimpleSyntaxSugar is not doing so well...".into(),
        annots: vec![Annotation { repl: None, msg: None, sev: Severity::Error, loc: Loc::encapsulate(ID) }],
    });
    assert_eq!(SimpleVerbose { loc: Loc::encapsulate(ID) }.into_diag(), Diag {
        theme:  Theme::PLAIN,
        sev:    Severity::Error,
        code:   None,
        msg:    "SimpleVerbose is not doing so well...".into(),
        annots: vec![Annotation { repl: None, msg: None, sev: Severity::Error, loc: Loc::encapsulate(ID) }],
    });
    assert_eq!(ExtraAnnot { loc: Loc::encapsulate_range(ID, ..3), reason: Loc::encapsulate_range(ID, 3..) }.into_diag(), Diag {
        theme:  Theme::PLAIN,
        sev:    Severity::Help,
        code:   None,
        msg:    "ExtraAnnot is not doing so well...".into(),
        annots: vec![Annotation { repl: None, msg: None, sev: Severity::Help, loc: Loc::encapsulate_range(ID, ..3) }, Annotation {
            repl: None,
            msg:  Some("...and here's why!".into()),
            sev:  Severity::Error,
            loc:  Loc::encapsulate_range(ID, 3..),
        }],
    });
    assert_eq!(WithCode { loc: Loc::encapsulate(ID) }.into_diag(), Diag {
        theme:  Theme::PLAIN,
        sev:    Severity::Warning,
        code:   Some("W001".into()),
        msg:    "WithCode is not doing so well...".into(),
        annots: Vec::new(),
    });
    assert_eq!(
        ManyAnnotsWithRepl {
            loc:     Loc::encapsulate_range(ID, ..3),
            reason1: Loc::encapsulate_range(ID, 3..6),
            reason2: Loc::encapsulate_range(ID, 6..9),
            reason3: Loc::encapsulate_range(ID, 9..),
        }
        .into_diag(),
        Diag {
            theme:  Theme::PLAIN,
            sev:    Severity::Error,
            code:   None,
            msg:    "Some error occurred!".into(),
            annots: vec![
                Annotation { repl: None, msg: None, sev: Severity::Error, loc: Loc::encapsulate_range(ID, ..3) },
                Annotation {
                    repl: None,
                    msg:  Some("...because of this error".into()),
                    sev:  Severity::Error,
                    loc:  Loc::encapsulate_range(ID, 3..6),
                },
                Annotation { repl: None, msg: Some("...and this error".into()), sev: Severity::Error, loc: Loc::encapsulate_range(ID, 6..9) },
                Annotation {
                    repl: Some("foo".into()),
                    msg:  Some("Try doing this instead!".into()),
                    sev:  Severity::Error,
                    loc:  Loc::encapsulate_range(ID, 9..),
                },
            ],
        }
    );
    assert_eq!(ComplexFormat { code: 42, world: "world!", foo: "foo", loc: Loc::encapsulate(ID) }.into_diag(), Diag {
        theme:  Theme::PLAIN,
        sev:    Severity::Error,
        code:   Some("E42".into()),
        msg:    "Hello, world!".into(),
        annots: vec![Annotation {
            repl: Some("foo".into()),
            msg:  Some("Goodbye, \"world!\"".into()),
            sev:  Severity::Warning,
            loc:  Loc::encapsulate(ID),
        }],
    });
    assert_eq!(TupleFormat(42, "world!", "foo", Loc::encapsulate(ID)).into_diag(), Diag {
        theme:  Theme::PLAIN,
        sev:    Severity::Error,
        code:   Some("E42".into()),
        msg:    "Hello, world!".into(),
        annots: vec![Annotation {
            repl: Some("foo".into()),
            msg:  Some("Goodbye, \"world!\"".into()),
            sev:  Severity::Warning,
            loc:  Loc::encapsulate(ID),
        }],
    });
}



#[test]
fn test_derive_diagnostic_enums() {
    #[derive(Diagnostic)]
    enum Test {
        // Simple syntax sugar case
        #[diag(error, msg = "SimpleSyntaxSugar is not doing so well...", loc)]
        SimpleSyntaxSugar { loc: Loc },

        // Simple verbose case
        #[diag(error, msg = "SimpleVerbose is not doing so well...")]
        #[annot(error, loc)]
        SimpleVerbose { loc: Loc },

        // Additional annotation
        #[diag(help, msg = "ExtraAnnot is not doing so well...", loc)]
        #[annot(error, msg = "...and here's why!", loc = reason)]
        ExtraAnnot { loc: Loc, reason: Loc },

        // Main message with code
        #[diag(warn, code = "W001", msg = "WithCode is not doing so well...")]
        WithCode { loc: Loc },

        // Many annotations with replacement
        #[diag(error, msg = "Some error occurred!", loc)]
        #[annot(error, msg = "...because of this error", loc = reason1)]
        #[annot(error, msg = "...and this error", loc = reason2)]
        #[annot(error, msg = "Try doing this instead!", repl = "foo", loc = reason3)]
        ManyAnnotsWithRepl { loc: Loc, reason1: Loc, reason2: Loc, reason3: Loc },

        // Complex formatting
        #[diag(error, code = ("E{}", code), msg = ("Hello, {}", world))]
        #[annot(warning, repl = ("{foo}"), msg = ("Goodbye, {:?}", world), loc)]
        ComplexFormat { code: u32, world: &'static str, foo: &'static str, loc: Loc },

        #[diag(error, code = ("E{f0}"), msg = ("Hello, {f1}"))]
        #[annot(warning, repl = ("{f2}"), msg = ("Goodbye, {f1:?}"), loc = f3)]
        TupleFormat(u32, &'static str, &'static str, Loc),
    }


    const ID: u64 = 0;
    assert_eq!(Test::SimpleSyntaxSugar { loc: Loc::encapsulate(ID) }.into_diag(), Diag {
        theme:  Theme::PLAIN,
        sev:    Severity::Error,
        code:   None,
        msg:    "SimpleSyntaxSugar is not doing so well...".into(),
        annots: vec![Annotation { repl: None, msg: None, sev: Severity::Error, loc: Loc::encapsulate(ID) }],
    });
    assert_eq!(Test::SimpleVerbose { loc: Loc::encapsulate(ID) }.into_diag(), Diag {
        theme:  Theme::PLAIN,
        sev:    Severity::Error,
        code:   None,
        msg:    "SimpleVerbose is not doing so well...".into(),
        annots: vec![Annotation { repl: None, msg: None, sev: Severity::Error, loc: Loc::encapsulate(ID) }],
    });
    assert_eq!(Test::ExtraAnnot { loc: Loc::encapsulate_range(ID, ..3), reason: Loc::encapsulate_range(ID, 3..) }.into_diag(), Diag {
        theme:  Theme::PLAIN,
        sev:    Severity::Help,
        code:   None,
        msg:    "ExtraAnnot is not doing so well...".into(),
        annots: vec![Annotation { repl: None, msg: None, sev: Severity::Help, loc: Loc::encapsulate_range(ID, ..3) }, Annotation {
            repl: None,
            msg:  Some("...and here's why!".into()),
            sev:  Severity::Error,
            loc:  Loc::encapsulate_range(ID, 3..),
        }],
    });
    assert_eq!(Test::WithCode { loc: Loc::encapsulate(ID) }.into_diag(), Diag {
        theme:  Theme::PLAIN,
        sev:    Severity::Warning,
        code:   Some("W001".into()),
        msg:    "WithCode is not doing so well...".into(),
        annots: Vec::new(),
    });
    assert_eq!(
        Test::ManyAnnotsWithRepl {
            loc:     Loc::encapsulate_range(ID, ..3),
            reason1: Loc::encapsulate_range(ID, 3..6),
            reason2: Loc::encapsulate_range(ID, 6..9),
            reason3: Loc::encapsulate_range(ID, 9..),
        }
        .into_diag(),
        Diag {
            theme:  Theme::PLAIN,
            sev:    Severity::Error,
            code:   None,
            msg:    "Some error occurred!".into(),
            annots: vec![
                Annotation { repl: None, msg: None, sev: Severity::Error, loc: Loc::encapsulate_range(ID, ..3) },
                Annotation {
                    repl: None,
                    msg:  Some("...because of this error".into()),
                    sev:  Severity::Error,
                    loc:  Loc::encapsulate_range(ID, 3..6),
                },
                Annotation { repl: None, msg: Some("...and this error".into()), sev: Severity::Error, loc: Loc::encapsulate_range(ID, 6..9) },
                Annotation {
                    repl: Some("foo".into()),
                    msg:  Some("Try doing this instead!".into()),
                    sev:  Severity::Error,
                    loc:  Loc::encapsulate_range(ID, 9..),
                },
            ],
        }
    );
    assert_eq!(Test::ComplexFormat { code: 42, world: "world!", foo: "foo", loc: Loc::encapsulate(ID) }.into_diag(), Diag {
        theme:  Theme::PLAIN,
        sev:    Severity::Error,
        code:   Some("E42".into()),
        msg:    "Hello, world!".into(),
        annots: vec![Annotation {
            repl: Some("foo".into()),
            msg:  Some("Goodbye, \"world!\"".into()),
            sev:  Severity::Warning,
            loc:  Loc::encapsulate(ID),
        }],
    });
    assert_eq!(Test::TupleFormat(42, "world!", "foo", Loc::encapsulate(ID)).into_diag(), Diag {
        theme:  Theme::PLAIN,
        sev:    Severity::Error,
        code:   Some("E42".into()),
        msg:    "Hello, world!".into(),
        annots: vec![Annotation {
            repl: Some("foo".into()),
            msg:  Some("Goodbye, \"world!\"".into()),
            sev:  Severity::Warning,
            loc:  Loc::encapsulate(ID),
        }],
    });
}
