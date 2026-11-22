//  LIB.rs
//    by Lut99
//
//  Description:
//!   Defines procedural macros for the ast-toolkit2 crate.
//

// Modules
#[cfg(any(feature = "diag", feature = "loc", feature = "tree"))]
mod common;
#[cfg(feature = "diag")]
mod derive_diagnostic;
#[cfg(feature = "loc")]
mod derive_located;
#[cfg(feature = "tree")]
mod derive_node;
#[cfg(feature = "nibble")]
mod derive_parsable;
#[cfg(feature = "visit")]
mod derive_visit;

// Imports
#[cfg(any(feature = "loc", feature = "tree"))]
use proc_macro::TokenStream;


/***** LIBRARY *****/
// ast_toolkit2::diag
#[cfg(feature = "diag")]
#[doc = include_str!("../docs/diagnostic.md")]
#[proc_macro_derive(Diagnostic, attributes(diag, annot))]
pub fn derive_diagnostic(item: TokenStream) -> TokenStream {
    match derive_diagnostic::handle(item.into()) {
        Ok(res) => res.into(),
        Err(err) => err.into_compile_error().into(),
    }
}



// ast_toolkit2::loc
#[cfg(feature = "loc")]
#[doc = include_str!("../docs/located.md")]
#[proc_macro_derive(Located, attributes(loc))]
pub fn derive_located(item: TokenStream) -> TokenStream {
    match derive_located::handle(item.into()) {
        Ok(res) => res.into(),
        Err(err) => err.into_compile_error().into(),
    }
}



// ast_toolkit2::nibble
#[cfg(feature = "nibble")]
#[doc = include_str!("../docs/parsable.md")]
#[proc_macro_derive(Parsable, attributes(node, parse))]
pub fn derive_parsable(item: TokenStream) -> TokenStream {
    match derive_parsable::handle(item.into()) {
        Ok(res) => res.into(),
        Err(err) => err.into_compile_error().into(),
    }
}



// ast_toolkit2::tree
#[cfg(feature = "tree")]
#[doc = include_str!("../docs/node.md")]
#[proc_macro_derive(Node, attributes(node))]
pub fn derive_node(item: TokenStream) -> TokenStream {
    match derive_node::handle(item.into()) {
        Ok(res) => res.into(),
        Err(err) => err.into_compile_error().into(),
    }
}



// ast_toolkit2::visit
#[cfg(feature = "visit")]
#[doc = include_str!("../docs/visit.md")]
#[proc_macro_derive(Visit, attributes(visit))]
pub fn derive_visit(item: TokenStream) -> TokenStream {
    match derive_visit::handle(item.into()) {
        Ok(res) => res.into(),
        Err(err) => err.into_compile_error().into(),
    }
}
