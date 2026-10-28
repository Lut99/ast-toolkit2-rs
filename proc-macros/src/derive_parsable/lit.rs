//  LITERAL.rs
//    by Lut99
//
//  Description:
//!   Implements the logic for literals, i.e., regex parsers.
//

use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::spanned::Spanned as _;
use syn::{Error, Fields, Generics, Ident, LitStr, Type};

use super::{OuterAttr, whitespace_pop_impl};


/***** LIBRARY *****/
/// Handles the implementation.
pub fn handle_impl(outer_attrs: &[OuterAttr], regex: &LitStr, ident: &Ident, gens: &Generics, fields: &Fields) -> Result<TokenStream2, Error> {
    // Prepare stuff
    let expects: TokenStream2 = OuterAttr::has_expects(outer_attrs)
        .map(ToTokens::to_token_stream)
        .unwrap_or_else(|| LitStr::new(&format!("a literal matching the regex {:?}", regex.value()), ident.span()).to_token_stream());
    let (whitespace_init, whitespace_pop) = whitespace_pop_impl("input", outer_attrs);
    let (impl_gen, ty_gen, where_bounds) = gens.split_for_impl();

    // Search for the type of the 'value' field
    let (constr, value_ty): (TokenStream2, &Type) = match fields {
        Fields::Named(n) => {
            let n_span = n.span();
            if n.named.len() != 2 {
                return Err(Error::new(n_span, "Expected two fields, a `value` and a `loc`-field"));
            }
            (
                quote! { #ident { value, loc } },
                &n.named.iter().find(|f| f.ident.as_ref().unwrap() == "value").ok_or_else(|| Error::new(n_span, "Missing `value`-field"))?.ty,
            )
        },
        Fields::Unnamed(u) => {
            if u.unnamed.len() != 2 {
                return Err(Error::new(u.span(), "Expected two fields, a `value` and a `loc`-field"));
            }
            (quote! { #ident(value, loc) }, &u.unnamed.iter().nth(1).unwrap().ty)
        },
        Fields::Unit => return Err(Error::new(fields.span(), "Expected two fields, a `value` and a `loc`-field")),
    };

    // Generate the impl
    Ok(quote! {
        impl #impl_gen ::ast_toolkit2::nibble::Parsable<::std::primitive::u8> for #ident #ty_gen #where_bounds {
            type Formatter = &'static ::std::primitive::str;

            type Error = <#value_ty as ::ast_toolkit2::nibble::Convert>::Error;

            fn expects() -> Self::Formatter { #expects }

            fn parse(input: ::ast_toolkit2::nibble::Slice<::std::primitive::u8>) -> ::std::result::Result<(Self, ::ast_toolkit2::nibble::Slice<::std::primitive::u8>), ::ast_toolkit2::nibble::error::NibbleError<Self::Formatter, Self::Error>> {
                type Match<'h> = ::ast_toolkit2::nibble::__macros::Match<'h>;
                type Regex = ::ast_toolkit2::nibble::__macros::Regex;

                // Pope whitespace first
                #whitespace_init
                #whitespace_pop

                // Make sure we compile the regex only once.
                static REGEX: ::std::sync::LazyLock<Regex> = ::std::sync::LazyLock::new(|| ::std::result::Result::unwrap_or_else(Regex::new(#regex), |err| ::std::panic!("Failed to compile regex {:?}: {err}", #regex)));
                let regex = &*REGEX;

                // Match on the slice
                let error_loc = ::ast_toolkit2::nibble::Slice::error_loc(&input);
                match Regex::find(regex, ::ast_toolkit2::nibble::Slice::as_slice(&input)) {
                    ::std::option::Option::Some(m) => match ::ast_toolkit2::nibble::Convert::try_parse(Match::as_bytes(&m)) {
                        Ok(value) => {
                            let loc = ::std::convert::From::from(::ast_toolkit2::loc::Loc { source: error_loc.source, range: ::std::convert::From::from(Match::range(&m)) });
                            ::std::result::Result::Ok((#constr, ::ast_toolkit2::nibble::Slice::slice(input, Match::end(&m)..)))
                        },
                        Err(err) => ::std::result::Result::Err(::ast_toolkit2::nibble::error::NibbleError::Error(err)),
                    },
                    ::std::option::Option::None => ::std::result::Result::Err(::ast_toolkit2::nibble::error::NibbleError::Unmatched(#expects, error_loc))
                }
            }
        }
    })
}
