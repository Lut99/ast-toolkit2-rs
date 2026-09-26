//  LITERAL.rs
//    by Lut99
//
//  Description:
//!   Implements the logic for literals, i.e., regex parsers.
//

use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens as _, quote};
use syn::{DataStruct, Error, Generics, Ident, LitStr, Visibility};

use super::OuterAttr;


/***** LIBRARY *****/
/// Handles the implementation.
pub fn handle_impl(
    outer_attrs: &[OuterAttr],
    regex: &LitStr,
    vis: Visibility,
    ident: Ident,
    gens: Generics,
    data: DataStruct,
) -> Result<TokenStream2, Error> {
    // Prepare stuff
    let fmt: Ident = Ident::new(&format!("{ident}Formatter"), ident.span());
    let expects: TokenStream2 = outer_attrs
        .into_iter()
        .find_map(|a| match a {
            OuterAttr::Expects { what } => Some(what.to_token_stream()),
            _ => None,
        })
        .unwrap_or_else(|| LitStr::new(&format!("a terminal matching the regex {:?}", regex.value()), ident.span()).to_token_stream());
    let (impl_gen, ty_gen, where_bounds) = gens.split_for_impl();

    // Figure out how to build the terminal
    let _ = ();

    // Generate the impl
    Ok(quote! {
        #[doc = ::std::concat!("A formatter for the ", ::std::stringify!(#ident), " terminal.")]
        #[derive(::std::Clone, ::std::Copy, ::std::Debug, ::std::Eq, ::std::PartialEq)]
        #vis struct #fmt;
        impl ::std::fmt::Display for #fmt {
            #[inline]
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result { ::std::write!(#expects) }
        }

        impl #impl_gen ::ast_toolkit2::nibble::Parsable<u8> for #ident #ty_gen #where_bounds {
            type Formatter = ::ast_toolkit2::nibble::error::fmt::TermFormatter;

            type Error = ::ast_toolkit2::nibble::error::TermError;

            fn expects() -> Self::Formatter { <::std::string::String as ::std::convert::From>::from(#expects) }

            fn parse(input: ::ast_toolkit2::nibble::Slice<E>) -> ::std::result::Result<(Self, ::ast_toolkit2::nibble::Slice<E>), ::ast_toolkit2::nibble::error::NibbleError<Self::Formatter, Self::Error>> {
                // Make sure we compile the regex only once.
                type Regex = ::ast_toolkit2::nibble::__macros::Regex;
                static REGEX: ::std::sync::LazyLock<Regex> = ::std::sync::LazyLock::new(|| {
                    Regex::new(#regex)
                });
                let regex = &*REGEX;

                // Match on the slice
                match Regex::find(regex, ::ast_toolkit2::nibble::Slice::as_slice(input)) {
                    Some(m) =>
                    None => Err(::)
                }
            }
        }
    })
}
