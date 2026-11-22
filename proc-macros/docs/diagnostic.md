A procedural macro for automatically deriving the `Diagnostic`-trait.

# Usage
To use this macro, add it to your struct with the `derive`-attribute and  then specify
annotations using Rust attributes.

For example, to annotate an error with a simple message:
```ignore
use ast_toolkit2::diag::Diagnostic;

#[derive(Diagnostic)]
#[diag(error, msg = "Failed to do a thing")]
struct Error;
```
When you call `Diagnostic::into_diag()`, the following generates:
```ignore
Diag {
    theme:  Theme::PLAIN,
    sev:    Severity::Error,
    code:   None,
    msg:    "Failed to do a thing".into(),
    notes:  Vec::new(),
    annots: Vec::new(),
}
```

You can also omit `msg = ...` to rely on a type's [`Display`]-implementation instead:
```ignore
use ast_toolkit2::diag::{Diag, Diagnostic};

#[derive(Diagnostic)]
#[diag(error)]
struct Error;
impl Display for Error {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> FResult { write!(f, "Failed to do some other thing") }
}

assert_eq!(Error.into_diag().msg, "Failed to do some other thing");
```

The same works for error variants:
```ignore
use ast_toolkit2::diag::{Diag, Diagnostic};

#[derive(Diagnostic)]
enum Error {
    #[diag(error, msg = "Custom error message")]
    Foo,
    #[diag(error)]
    Bar,
}
impl Display for Error {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> FResult { write!(f, "Display error message") }
}

assert_eq!(Error::Foo.into_diag().msg, "Custom error message");
assert_eq!(Error::Bar.into_diag().msg, "Display error message");
```

## Severities
Note the `error`-attribute: it actually signifies the severity of the annotation that will be generated.
Possible options are `error`, `help`, `suggestion` or `warn`/`warning`.
For example:
```ignore
use ast_toolkit2::diag::{Diag, Diagnostic, Severity};

#[derive(Diagnostic)]
#[diag(warn, msg = "Something might go wrong anytime now")]
struct Warning;

let diag = Error.into_diag();
assert_eq!(diag.severity, Severity::Warning);
assert_eq!(diag.msg, "Something might go wrong anytime now");
```

## Formatting
The `msg`-attribute is quite powerful, as it allows you to write format expressions.
A format expression is what you would give as arguments to the [`format!()`]-macro: parenthesis
with first the formatting literatl, then any arguments.

For example:
```ignore
use ast_toolkit2::diag::{Diag, Diagnostic};

#[derive(Diagnostic)]
#[diag(error, msg = ("{:?} failed!", what))]
struct Error { what: String }

assert_eq!(Error { what: "Foo".into() }.into_diag().msg, "\"Foo\" failed!");
```
You can assume that the names of the fields are available in scope for your formatting arguments.
For tuple structs/variants, refer to fields by their index in curly brackets (`{0}`) or prepend  the index by `f` (`f0`).

## Notes
Another feature of the main error is that it displays notes, to give additional context after the user read
the entire error.

To add them, use `#[diag(note = ...)]`:
```ignore
use ast_toolkit2::diag::{Diag, Diagnostic};

#[derive(Diagnostic)]
#[diag(error, msg = "Main problem", note = "More information")]
struct Error;

assert_eq!(Error.into_diag().notes, vec!["More information".into()]);
```
Like `msg`, `notes` support formatting. In addition, you can give the attribute multiple times for multiple notes.

## `Loc`ations
Diagnostics are mostly used to contextualize the error in the original source text.
As such, each diagnostic/annotation is usually accompanied with a `Loc` to refer to the original source text.

This can be referred to with the `loc`-attribute:
```ignore
use ast_toolkit2::diag::{Annotation, Diag, Diagnostic};
use ast_toolkit2::loc::Loc;

#[derive(Diagnostic)]
#[diag(error, msg = "Failed to parse this", loc)]
struct Error { loc: Loc }

let diag = Error(Loc::encapsulate_range(0, ..2)).into_diag();
assert_eq!(diag.msg, "Failed to parse this");
assert_eq!(diag.annots, vec![
    Annotation { repl: None, msg: None, sev: Severity::Error, loc: Loc::encapsulate_range(0, ..2) }
]);
```
Note how an `Annotation` has appeared: this represents the highlighting of a piece of source text.

`#[diag(loc)]` is actually shorthand for `#[diag(loc = loc)]`, where `loc` is the name of a field.
So this is also possible:
```ignore
use ast_toolkit2::diag::{Annotation, Diag, Diagnostic};
use ast_toolkit2::loc::Loc;

#[derive(Diagnostic)]
#[diag(error, msg = "Failed to parse this", loc = reason)]
struct Error { reason: Loc }

let diag = Error(Loc::encapsulate_range(0, ..2)).into_diag();
assert_eq!(diag.msg, "Failed to parse this");
assert_eq!(diag.annots, vec![
    Annotation { repl: None, msg: None, sev: Severity::Error, loc: Loc::encapsulate_range(0, ..2) }
]);
```
More general, `#[diag(loc = ...)]` expects any expression at `...`, so you could also build more complex
`Loc`s on the fly. For structs/variants in tuple syntax, use `{0}` to refer to a particular index. E.g.,
```ignore
use ast_toolkit2::diag::{Annotation, Diag, Diagnostic};
use ast_toolkit2::loc::Loc;

#[derive(Diagnostic)]
#[diag(error, msg = "Failed to parse this string", loc = {1})]
struct Error(String, Loc);

let diag = Error("Howdy".into(), Loc::encapsulate_range(0, ..2)).into_diag();
assert_eq!(diag.msg, "Failed to parse this string");
assert_eq!(diag.annots, vec![
    Annotation { repl: None, msg: None, sev: Severity::Error, loc: Loc::encapsulate_range(0, ..2) }
]);
```

## Codes
A small feature is that you can define codes to display separately to a main error message:
```ignore
use ast_toolkit2::diag::{Diag, Diagnostic};

#[derive(Diagnostic)]
#[diag(error, code = "E000", msg = "Failed to do a thing")]
struct Error;

let diag = Error.into_diag();
assert_eq!(diag.code, Some("E000"));
assert_eq!(diag.msg, "Failed to do a thing");
```
Like messages, `code` supports full format syntax if you want to make more complex codes.

## Annotations
One big part of this macro is that you can add multiple messages per struct/variant.

For example:
```ignore
use ast_toolkit2::diag::{Diag, Diagnostic};

#[derive(Diagnostic)]
#[diag(help, msg = "Some error occurred", loc)]
#[annot(error, msg = "...and here's why!", loc = reason)]
struct Error { loc: Loc, reason: Loc }

let diag = Error { loc: Loc::encapsulate(0), reason: Loc::encapsulate(1) }.into_diag();
assert_eq!(diag.msg, "Some error occurred");
assert_eq!(diag.annots, vec![
    Annotation { repl: None, msg: None, sev: Severity::Help, loc: Loc::encapsulate(0) },
    Annotation { repl: None, msg: Some("...and here's why!".into()), sev: Severity::Error, loc: Loc::encapsulate(1) }
]);
```
When rendering the diag, this will now show two highlights, where the second features additional hints
and/or has different highlighting properties than the main message.

You can give any number of annotations. Just remember that annotations need to have a `loc` associated with them.

## Replacing
One feature unique to annotations (i.e., not on the main `diag`) is to define a suggested replacement.

To replace something in the source text, give the type of each element (usually bytes - [`u8`]). Then, you can give an
array of those elements to store as replacement.

For example:
```ignore
use ast_toolkit2::diag::{Diag, Diagnostic};

#[derive(Diagnostic)]
#[diag(help, msg = "Some error occurred", loc)]
#[annot(error, msg = "Why don't you try this?", repl<u8> = b"Foo", loc = reason)]
struct Error { loc: Loc, reason: Loc }

let diag = Error { loc: Loc::encapsulate(0), reason: Loc::encapsulate(1) }.into_diag();
assert_eq!(diag.msg, "Some error occurred");
assert_eq!(diag.annots, vec![
    Annotation { repl: None, msg: None, sev: Severity::Help, loc: Loc::encapsulate(0) },
    Annotation { repl: Some(b"Foo"), msg: Some("Why don't you try this?".into()), sev: Severity::Error, loc: Loc::encapsulate(1) }
]);
```

# Generics
Like most derive macros, this macro will automatically add a `Diagnostic`-bound on all generic
types declared on the implemented type.

If you need other generic behaviour, you should implement `Diagnostic` yourself.
