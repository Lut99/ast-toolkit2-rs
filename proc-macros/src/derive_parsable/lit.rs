//  LITERAL.rs
//    by Lut99
//
//  Description:
//!   Implements the logic for literals, i.e., regex parsers.
//

use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens as _, quote};
use syn::spanned::Spanned as _;
use syn::{DataStruct, Error, Fields, Generics, Ident, LitStr, Type};

use super::OuterAttr;


/***** LIBRARY *****/
/// Handles the implementation.
pub fn handle_impl(outer_attrs: &[OuterAttr], regex: &LitStr, ident: Ident, gens: Generics, data: DataStruct) -> Result<TokenStream2, Error> {
    // Prepare stuff
    let expects: TokenStream2 = outer_attrs
        .into_iter()
        .find_map(|a| match a {
            OuterAttr::Expects { what } => Some(what.to_token_stream()),
            _ => None,
        })
        .unwrap_or_else(|| LitStr::new(&format!("a literal matching the regex {:?}", regex.value()), ident.span()).to_token_stream());
    let (impl_gen, ty_gen, where_bounds) = gens.split_for_impl();

    // Search for the type of the 'value' field
    let value_ty: Type = match data.fields {
        Fields::Named(n) => {
            let n_span = n.span();
            if n.named.len() != 2 {
                return Err(Error::new(n_span, "Expected two fields, a `value` and a `loc`-field"));
            }
            n.named.into_iter().find(|f| f.ident.as_ref().unwrap() == "value").ok_or_else(|| Error::new(n_span, "Missing `value`-field"))?.ty
        },
        Fields::Unnamed(u) => {
            if u.unnamed.len() != 2 {
                return Err(Error::new(u.span(), "Expected two fields, a `value` and a `loc`-field"));
            }
            u.unnamed.into_iter().nth(1).unwrap().ty
        },
        Fields::Unit => return Err(Error::new(data.fields.span(), "Expected two fields, a `value` and a `loc`-field")),
    };

    // Generate the impl
    Ok(quote! {
        // #[doc = ::std::concat!("A formatter for the ", ::std::stringify!(#ident), " terminal.")]
        // #[derive(::std::Clone, ::std::Copy, ::std::Debug, ::std::Eq, ::std::PartialEq)]
        // #vis struct #fmt;
        // impl ::std::fmt::Display for #fmt {
        //     #[inline]
        //     fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result { ::std::write!(#expects) }
        // }

        impl #impl_gen ::ast_toolkit2::nibble::Parsable<::std::primitive::u8> for #ident #ty_gen #where_bounds {
            type Formatter = &'static ::std::primitive::str;

            type Error = <#value_ty as ::ast_toolkit2::nibble::Convert>::Error;

            fn expects() -> Self::Formatter { #expects }

            fn parse(input: ::ast_toolkit2::nibble::Slice<::std::primitive::u8>) -> ::std::result::Result<(Self, ::ast_toolkit2::nibble::Slice<::std::primitive::u8>), ::ast_toolkit2::nibble::error::NibbleError<Self::Formatter, Self::Error>> {
                type Match<'h> = ::ast_toolkit2::nibble::__macros::Match<'h>;
                type Regex = ::ast_toolkit2::nibble::__macros::Regex;

                // Make sure we compile the regex only once.
                static REGEX: ::std::sync::LazyLock<Regex> = ::std::sync::LazyLock::new(|| ::std::result::Result::unwrap_or_else(Regex::new(#regex), |err| ::std::panic!("Failed to compile regex {:?}: {err}", #regex)));
                let regex = &*REGEX;

                // Match on the slice
                let error_loc = ::ast_toolkit2::nibble::Slice::error_loc(&input);
                match Regex::find(regex, ::ast_toolkit2::nibble::Slice::as_slice(&input)) {
                    ::std::option::Option::Some(m) => match ::ast_toolkit2::nibble::Convert::try_parse(Match::as_bytes(&m)) {
                        Ok(value) => ::std::result::Result::Ok((Self { value, loc: ::std::convert::From::from(::ast_toolkit2::loc::Loc { source: error_loc.source, range: ::std::convert::From::from(Match::range(&m)) }) }, ::ast_toolkit2::nibble::Slice::slice(input, Match::end(&m)..))),
                        Err(err) => ::std::result::Result::Err(::ast_toolkit2::nibble::error::NibbleError::Error(err)),
                    },
                    ::std::option::Option::None => ::std::result::Result::Err(::ast_toolkit2::nibble::error::NibbleError::Unmatched(#expects, error_loc))
                }
            }
        }
    })
}
