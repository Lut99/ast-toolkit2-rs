//  DERIVE VISIT.rs
//    by Lut99
//
//  Description:
//!   Implements the derive macro for `Visit`.
//

use proc_macro2::TokenStream as TokenStream2;
use syn::ext::IdentExt as _;
use syn::parse::{Parse, ParseStream};
use syn::{Data, DeriveInput, Error, Ident};


/***** ATTRIBUTES *****/
/// Defines field-level attributes for `#[visit(...)]`.
pub enum FieldAttr {
    Skip,
}
impl FieldAttr {
    #[inline]
    fn has_skip(attrs: &[Self]) -> bool { attrs.into_iter().any(|a| matches!(a, Self::Skip)) }

    fn check_consistency(_attrs: &[Self]) -> Result<(), Error> { Ok(()) }
}
impl Parse for FieldAttr {
    #[inline]
    fn parse(input: ParseStream) -> Result<Self, Error> {
        // The stream must begin with an identifier
        let ident: Ident = Ident::parse_any(input)?;
        if ident == "skip" {
            Ok(Self::Skip)
        } else {
            Err(Error::new(ident.span(), &format!("Unknown `skip` field attribute {:?}", ident.to_string())))
        }
    }
}





/***** LIBRARY *****/
/// Main handler for the macro.
pub fn handle(item: TokenStream2) -> Result<TokenStream2, Error> {
    let DeriveInput { attrs, ident, data, mut generics, .. } = syn::parse2(item)?;
    if let Data::Union(u) = data {
        return Err(Error::new(u.union_token.span, "Can only derive `Node` on structs or enums"));
    }

    Ok(TokenStream2::new())
}
