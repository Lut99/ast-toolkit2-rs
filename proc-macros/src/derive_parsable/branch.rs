//  BRANCH.rs
//    by Lut99
//
//  Description:
//!   Implements the logic for enums, i.e., branching parsers.
//

use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens as _, quote};
use syn::spanned::Spanned as _;
use syn::{DataEnum, Error, Generics, Ident, Type, Visibility};

use super::seq::cook_up_error_mod_name;
use super::{OuterAttr, parse_parse_attrs, whitespace_pop_impl};
use crate::derive_parsable::VariantAttr;


/***** HELPERS *****/
struct VariantInfo {
    /// The name of the variant.
    name:     Ident,
    /// Tokens representing the correct formatter type when using this variant.
    fmt_ty:   TokenStream2,
    /// Tokens representing the instantiation of the formatting type
    fmt_inst: TokenStream2,
    /// Tokens representing the correct error type when using this variant.
    err_ty:   TokenStream2,
}





/***** LIBRARY *****/
/// Handles the implementation.
pub fn handle_impl(outer_attrs: &[OuterAttr], vis: Visibility, ident: Ident, mut gens: Generics, data: DataEnum) -> Result<TokenStream2, Error> {
    // Cook up the error module name
    let error_mod = cook_up_error_mod_name(outer_attrs, &ident);

    // For every variant, get the formatter- and error type references
    let mut variants = Vec::<VariantInfo>::new();
    for variant in &data.variants {
        // Parse its attributes and check if we should skip this
        let var_attrs: Vec<VariantAttr> = parse_parse_attrs(&variant.attrs)?;
        if VariantAttr::has_skip(&var_attrs) {
            continue;
        }

        // Ensure it has exactly one (1) field
        if variant.fields.len() != 1 {
            return Err(Error::new(
                variant.fields.span(),
                "Can only derive `Parsable` for (unskipped) enum variants that have precisely one field.\n\nConsider putting your fields in a \
                 separate struct.",
            ));
        }
        let field = variant.fields.iter().next().unwrap();
        let parsed_ty: &Type = match (VariantAttr::has_as(&var_attrs), VariantAttr::has_try_as(&var_attrs)) {
            (Some(other), None) => other,
            (None, Some(other)) => other,
            (None, None) => &field.ty,
            _ => unreachable!(),
        };

        // Build our info around that
        let (fmt_ty, fmt_inst): (TokenStream2, TokenStream2) = if let Some(what) = VariantAttr::has_expects(&var_attrs) {
            (quote! { &'static ::std::primitive::str }, what.to_token_stream())
        } else {
            (
                quote! { <#parsed_ty as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::Formatter },
                quote! { <#parsed_ty as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::expects() },
            )
        };
        variants.push(VariantInfo {
            name: variant.ident.clone(),
            fmt_ty,
            fmt_inst,
            err_ty: quote! { <#parsed_ty as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::Error },
        })
    }

    // Get some whitespace info
    let (whitespace_init, whitespace_pop) = whitespace_pop_impl("rem", outer_attrs);

    // Generate the module with the error- and formatter types
    let mut res = TokenStream2::new();
    res.extend(quote! {
        #[doc = ::std::concat!("Helper module for [`", ::std::stringify!(#ident), "`] containing auxillary types.")]
        #vis mod #error_mod {
            use super::*;

            #[doc = ::std::concat!("Formatting type for genering the [expects](::ast_toolkit2::nibble::Parsable::expects())-implementation of [`", ::std::stringify!(#ident), "`].")]
            #[derive(::std::clone::Clone, ::std::marker::Copy, ::std::fmt::Debug, ::std::cmp::Eq, ::std::cmp::PartialEq)]
            pub struct Formatter #gens {
                #fields_def
            }

            #[automatically_derived]
            impl #impl_gen ::std::fmt::Display for Formatter #ty_gen #where_bounds {
                #[inline]
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    #fmt_impl
                    ::std::fmt::Result::Ok(())
                }
            }
        }
    });

    // Now run the generation ~~
    quote! {}
}
