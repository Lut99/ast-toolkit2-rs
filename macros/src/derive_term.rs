//  DERIVE TERM.rs
//    by Lut99
//
//  Description:
//!   Implements the derive macro for `Term`.
//

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DataUnion, DeriveInput, Error};

use crate::common::inject_trait_bound;


/***** LIBRARY *****/
/// Main handler for the macro.
pub fn handle(item: TokenStream2) -> Result<TokenStream2, Error> {
    let DeriveInput { ident, data, mut generics, .. } = syn::parse2(item)?;
    match data {
        Data::Struct(_) | Data::Enum(_) => {
            inject_trait_bound(["ast_toolkit2", "tree", "Node"], &mut generics);
            let (impl_gen, ty_gen, where_clauses) = generics.split_for_impl();
            Ok(quote! {
                impl #impl_gen ::ast_toolkit2::tree::Term for #ident #ty_gen #where_clauses {}
            })
        },
        Data::Union(DataUnion { union_token, .. }) => Err(Error::new(union_token.span, "Can only derive `Term` on structs or enums")),
    }
}
