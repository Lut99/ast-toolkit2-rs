//  LIB.rs
//    by Lut99
//
//  Description:
//!   Defines procedural macros for the ast-toolkit2 crate.
//

// Modules
#[cfg(any(feature = "loc", feature = "tree"))]
mod common;
#[cfg(feature = "diag")]
mod derive_diagnostic;
#[cfg(feature = "loc")]
mod derive_located;
#[cfg(feature = "tree")]
mod derive_node;
#[cfg(feature = "nibble")]
mod derive_parsable;

// Imports
#[cfg(any(feature = "loc", feature = "tree"))]
use proc_macro::TokenStream;


/***** LIBRARY *****/
// ast_toolkit2::diag
/// A procedural macro for automatically deriving the `Diagnostic`-trait.
///
/// TODO
#[cfg(feature = "diag")]
#[proc_macro_derive(Diagnostic, attributes(diag, annot))]
pub fn derive_diagnostic(item: TokenStream) -> TokenStream {
    match derive_diagnostic::handle(item.into()) {
        Ok(res) => res.into(),
        Err(err) => err.into_compile_error().into(),
    }
}



// ast_toolkit2::loc
/// A procedural macro for automatically deriving the `Located`-trait.
///
/// By default, this will just return one of the fields of your type when `Located::loc()` is
/// called. But you can define different behaviour (see below).
///
/// # Usage
/// To use this macro, add it to your struct with the `derive`-attribute:
/// ```ignore
/// use ast_toolkit2::loc::{Loc, Located};
///
/// #[derive(Located)]
/// struct Foo {
///     foo: String,
///     loc: Loc,
/// }
///
/// assert_eq!(Foo { foo: "Hello, world!".into(), loc: Loc::encapsulate(0) }.loc(), Loc::encapsulate(0));
/// ```
///
/// As you can see, the macro does some derivation to see which fields to return. Specifically, it
/// will derive:
/// - If you have something (a struct or a variant) in struct-syntax, it will look for the only field
///   called `loc`; or
/// - If you have something (a struct or a variant) in tuple-syntax, it will look for the only
///   field.
///
/// Else, you will manually have to annotate it, like so:
/// ```ignore
/// use ast_toolkit2::loc::{Loc, Located};
///
/// #[derive(Located)]
/// struct Bar {
///     foo: String,
///     #[loc]
///     bar: Loc,
/// }
///
/// assert_eq!(Bar { foo: "Hello, world!".into(), bar: Loc::encapsulate(0) }.loc(), Loc::encapsulate(0));
/// ```
///
/// Note that you can select multiple fields to return, in which case they will be `Loc::extend()`ed
/// (in the order they are specified):
/// ```ignore
/// use ast_toolkit2::loc::{Loc, Located};
///
/// #[derive(Located)]
/// struct Baz {
///     #[loc]
///     foo: Loc,
///     #[loc]
///     bar: Loc,
/// }
///
/// assert_eq!(Bar { foo: Loc::encapsulate_range(0, ..2), bar: Loc::encapsulate_range(0, 2..4) }.loc(), Loc::encapsulate_range(0, ..4));
/// ```
///
/// If you want to use all fields anyway, you can also use:
/// ```ignore
/// use ast_toolkit2::loc::{Loc, Located};
///
/// #[derive(Located)]
/// #[loc(all)]
/// struct Quz {
///     foo: Loc,
///     bar: Loc,
/// }
///
/// assert_eq!(Quz { foo: Loc::encapsulate_range(0, ..2), bar: Loc::encapsulate_range(0, 2..4) }.loc(), Loc::encapsulate_range(0, ..4));
/// ```
/// where you can then exclude individual fields using `#[loc(skip)]`:
/// ```ignore
/// use ast_toolkit2::loc::{Loc, Located};
///
/// #[derive(Located)]
/// #[loc(all)]
/// struct Quz {
///     #[loc(skip)]
///     foo: String,
///     bar: Loc,
/// }
///
/// assert_eq!(Quz { foo: "Hello, world!".into(), bar: Loc::encapsulate_range(0, 2..4) }.loc(), Loc::encapsulate_range(0, 2..4));
/// ```
///
/// All of the above also works on enums:
/// ```ignore
/// use ast_toolkit2::loc::{Loc, Located};
///
/// #[derive(Located)]
/// enum Qux {
///     Foo { loc: Loc },
///     Bar { #[loc] bar: Loc },
///     Baz(Loc),
///     #[loc(all)]
///     Quz(Loc, Loc),
/// }
///
/// assert_eq!(Qux::Foo { loc: Loc::encapsulate(0) }.loc(), Loc::encapsulate(0));
/// assert_eq!(Qux::Bar { bar: Loc::encapsulate(1) }.loc(), Loc::encapsulate(1));
/// assert_eq!(Qux::Baz(Loc::encapsulate(2)).loc(), Loc::encapsulate(2));
/// assert_eq!(Qux::Quz(Loc::encapsulate_range(3, ..2), Loc::encapsulate_range(3, 2..4)).loc(), Loc::encapsulate_range(3, ..4));
/// ```
///
/// Finally, you can use `#loc(new)` to ignore all the fields and simply generate an empty, new
/// `Loc` using `Loc::new()`:
/// ```ignore
/// use ast_toolkit2::loc::{Loc, Located};
///
/// #[derive(Located)]
/// #[loc(new)]
/// struct Quux {
///     foo: String,
/// }
///
/// assert_eq!(Quux { foo: "Hello, world!".into() }.loc(), Loc::new());
/// ```
///
/// ## Deriving nested
/// Note that the implementation actually makes use of the `Located::loc()`-implementation of your
/// field. So, this is possible:
/// ```ignore
/// use ast_toolkit2::loc::{Loc, Located};
///
/// #[derive(Located)]
/// struct Foo(Loc);
/// #[derive(Located)]
/// struct Bar(Foo);
///
/// assert_eq!(Bar(Foo(Loc::encapsulate(0))).loc(), Loc::encapsulate(0));
///
/// ## A note on generics
/// Like most derive macros, this macro will automatically add a `Located`-bound on all generic
/// types declared on the implemented type.
///
/// Note, however, that this is _not_ the case if you declare `#loc(new)` (in which case no field
/// is used for the impl) or if you have an empty enum.
///
/// If you need other generic behaviour, you should implement `Located` yourself.
/// ```
#[cfg(feature = "loc")]
#[proc_macro_derive(Located, attributes(loc))]
pub fn derive_located(item: TokenStream) -> TokenStream {
    match derive_located::handle(item.into()) {
        Ok(res) => res.into(),
        Err(err) => err.into_compile_error().into(),
    }
}



// ast_toolkit2::nibble
/// TODO
/// A lot
///
/// Including:
/// - type generation name collisions for sequences;
/// - also using `node`
#[cfg(feature = "nibble")]
#[proc_macro_derive(Parsable, attributes(node, parse))]
pub fn derive_parsable(item: TokenStream) -> TokenStream {
    match derive_parsable::handle(item.into()) {
        Ok(res) => res.into(),
        Err(err) => err.into_compile_error().into(),
    }
}



// ast_toolkit2::tree
/// A procedural macro for automatically deriving the `Node`-, `NonTerm`- and `Term`-traits.
///
/// # Usage
/// There is three flavours to deriving your type.
///
/// ## Nonterminals
/// Nonterminals are branches in your tree. Derive them using the `#[node(nonterm)]`-attribute.
///
/// ## Literals
/// Literals are leaves in your tree that do not parse a static piece of text, but rather a value.
/// Derive them using the `#[node(lit = "...")]`-attribute, where `...` is a regular expression
/// defining the syntax of values parsed.
///
/// ## Terminals
/// Terminals are leaves in your tree that parse static pieces of text. Derive them using the
/// `#[node(term = "...")]` attribute, where `...` is the text to parse.
///
/// # Examples
/// All of them together:
/// ```ignore
/// use ast_toolkit2::tree::{Node, NonTerm, Term};
///
/// #[derive(Node)]
/// #[node(nonterm)]
/// struct Sentence(LitInt, Dot);
///
/// #[derive(Node)]
/// #[node(lit = "^[0-9]+")]
/// struct LitInt;
///
/// #[derive(Node)]
/// #[node(term = ".")]
/// struct Dot;
///
///
/// fn assert_node<T: Node>() {}
/// fn assert_nonterm<T: NonTerm>() {}
/// fn assert_term<T: Term>() {}
///
/// assert_node::<Sentence>();
/// assert_nonterm::<Sentence>();
/// // assert_term::<Sentence>();
/// assert_node::<LitInt>();
/// // assert_nonterm::<LitInt>();
/// assert_term::<LitInt>();
/// assert_node::<Dot>();
/// // assert_nonterm::<Dot>();
/// assert_term::<Dot>();
/// ```
///
/// # A note on generics
/// All generic types of your struct will require `Node` to have `Node`, `NonTerm` or `Term`
/// implemented.
#[cfg(feature = "tree")]
#[proc_macro_derive(Node, attributes(node))]
pub fn derive_node(item: TokenStream) -> TokenStream {
    match derive_node::handle(item.into()) {
        Ok(res) => res.into(),
        Err(err) => err.into_compile_error().into(),
    }
}
