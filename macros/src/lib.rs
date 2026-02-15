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
#[proc_macro_derive(Located, attributes(loc))]
pub fn derive_located(item: TokenStream) -> TokenStream {
    match derive_located::handle(item.into()) {
        Ok(res) => res.into(),
        Err(err) => err.into_compile_error().into(),
    }
}
