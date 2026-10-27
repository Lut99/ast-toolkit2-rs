//  TERMINAL.rs
//    by Lut99
//
//  Description:
//!   Implements the logic for terminals, i.e., tag parsers.
//

use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::{DataStruct, Error, Fields, Generics, Ident, LitStr};

use super::{OuterAttr, whitespace_pop_impl};


/***** LIBRARY *****/
/// Handles the implementation.
pub fn handle_impl(outer_attrs: &[OuterAttr], tag: &LitStr, ident: Ident, gens: Generics, data: DataStruct) -> Result<TokenStream2, Error> {
    // Prepare stuff
    let expects: TokenStream2 = OuterAttr::has_expects(outer_attrs)
        .map(ToTokens::to_token_stream)
        .unwrap_or_else(|| LitStr::new(&format!("{}", tag.value()), ident.span()).to_token_stream());
    let (whitespace_init, whitespace_pop) = whitespace_pop_impl("rem", outer_attrs);
    let (impl_gen, ty_gen, where_bounds) = gens.split_for_impl();

    // Figure out how to build the terminal
    let constr: TokenStream2 = match &data.fields {
        Fields::Named(n) => {
            if n.named.len() == 1 {
                let ident = n.named.iter().next().unwrap().ident.as_ref().unwrap();
                quote!(Self { #ident: <_ as ::std::convert::From<::ast_toolkit2::loc::Loc>>::from(loc) })
            } else {
                return Err(Error::new(ident.span(), "Expected one `Loc`-field to put the parsed range in"));
            }
        },
        Fields::Unnamed(u) => {
            if u.unnamed.len() == 1 {
                quote!(Self(<_ as ::std::convert::From<::ast_toolkit2::loc::Loc>>::from(loc)))
            } else {
                return Err(Error::new(ident.span(), "Expected exactly one `Loc`-field to put the parsed range in"));
            }
        },
        Fields::Unit => return Err(Error::new(ident.span(), "Expected exactly one `Loc`-field to put the parsed range in")),
    };

    // Generate the checking for a separator at the end
    let sepcheck: TokenStream2 = if let Some(regex) = outer_attrs.into_iter().find_map(|a| match a {
        OuterAttr::SepCheck { regex } => Some(regex),
        _ => None,
    }) {
        quote! {
            if !<[u8]>::is_empty(&rem) {
                // Check that it does *not* continue being an identifier
                static REGEX: ::std::sync::LazyLock<Regex> = ::std::sync::LazyLock::new(|| ::std::result::Result::unwrap_or_else(Regex::new(#regex), |err| ::std::panic!("Failed to compile regex {:?}: {err}", #regex)));
                let regex = &*REGEX;
                let slice = &rem[..1];
                if Regex::is_match(regex, slice) {
                    return ::std::result::Result::Err(::ast_toolkit2::nibble::error::NibbleError::Unmatched(
                        <#ident as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::expects(),
                        loc
                    ));
                }
            }
        }
    } else {
        TokenStream2::new()
    };

    // Generate the impl
    Ok(quote! {
        impl #impl_gen ::ast_toolkit2::nibble::Parsable<::std::primitive::u8> for #ident #ty_gen #where_bounds {
            type Formatter = &'static ::std::primitive::str;

            type Error = ::std::convert::Infallible;

            fn expects() -> Self::Formatter { #expects }

            fn parse(rem: ::ast_toolkit2::nibble::Slice<::std::primitive::u8>) -> ::std::result::Result<(Self, ::ast_toolkit2::nibble::Slice<::std::primitive::u8>), ::ast_toolkit2::nibble::error::NibbleError<Self::Formatter, Self::Error>> {
                #[allow(unused)]
                type Match<'a> = ::ast_toolkit2::nibble::__macros::Match<'a>;
                #[allow(unused)]
                type Regex = ::ast_toolkit2::nibble::__macros::Regex;
                const TAG: &[::std::primitive::u8] = ::std::primitive::str::as_bytes(#tag);

                #whitespace_init
                #whitespace_pop

                // Get the head slice
                let tag_len: usize = <[::std::primitive::u8]>::len(TAG);
                let ((head, loc), rem) = ::ast_toolkit2::nibble::Slice::head_slice_loc(rem, tag_len);

                // Check the overlapping part of the tag and the slice for equality
                let head_len: usize = <[::std::primitive::u8]>::len(head);
                if &TAG[..head_len] != head {
                    return ::std::result::Result::Err(::ast_toolkit2::nibble::error::NibbleError::Unmatched(
                        <#ident as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::expects(),
                        loc
                    ));
                }

                // Ensure we matched the whole tag
                if head_len < tag_len {
                    let needed: usize = tag_len - head_len;
                    return ::std::result::Result::Err(::ast_toolkit2::nibble::error::NibbleError::NotEnough(
                        <#ident as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::expects(),
                        ::ast_toolkit2::nibble::error::Needed::Bounded(needed, needed),
                        loc
                    ));
                }

                #sepcheck

                // Done, build self
                Ok((#constr, rem))
            }
        }
    })
}
