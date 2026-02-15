//  LIB.rs
//    by Lut99
//
//  Description:
//!   Defines procedural macros for the ast-toolkit2 crate.
//

// Modules
mod derive_located;

// Imports
use proc_macro::TokenStream;


/***** LIBRARY *****/
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
/// Finally, if you want to use all fields anyway, you can also use:
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
/// ```
#[proc_macro_derive(Located, attributes(loc))]
pub fn derive_located(item: TokenStream) -> TokenStream {
    match derive_located::handle(item.into()) {
        Ok(res) => res.into(),
        Err(err) => err.into_compile_error().into(),
    }
}
