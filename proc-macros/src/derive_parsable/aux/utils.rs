//  UTILITIES.rs
//    by Lut99
//
//  Description:
//!   A collection of useful, global helper functions.
//

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{
    AngleBracketedGenericArguments, GenericArgument, Generics, Ident, Path, PathArguments, PathSegment, TraitBound, Type, TypeParamBound, TypePath,
};

use super::super::OuterAttribute;


/***** LIBRARY *****/
/// Turns an [`Ident`]ifier into a [`Path`].
#[inline]
pub fn identity_path(ident: Ident) -> Path {
    Path {
        leading_colon: None,
        segments:      {
            let mut segs = Punctuated::new();
            segs.push(PathSegment { ident, arguments: PathArguments::None });
            segs
        },
    }
}



/// Given a set of [`Generics`], assigns each of them the `Parsable`-trait.
///
/// # Arguments
/// - `gens`: The [`Generics`] to inject the additional trait bounds in.
pub fn inject_parsable_bound(gens: &mut Generics) {
    for param in gens.type_params_mut() {
        param.bounds.push(TypeParamBound::Trait(TraitBound {
            paren_token: None,
            lifetimes: None,
            modifiers: Default::default(),
            maybe: None,
            path: Path {
                leading_colon: Some(Default::default()),
                segments:      {
                    let mut segs = Punctuated::new();
                    segs.push(PathSegment { ident: Ident::new("ast_toolkit2", Span::call_site()), arguments: PathArguments::None });
                    segs.push(PathSegment { ident: Ident::new("nibble", Span::call_site()), arguments: PathArguments::None });
                    segs.push(PathSegment {
                        ident:     Ident::new("Parsable", Span::call_site()),
                        arguments: PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                            colon2_token: None,
                            lt_token: Default::default(),
                            args: {
                                let mut args = Punctuated::new();
                                args.push(GenericArgument::Type(Type::Path(TypePath {
                                    attrs: Vec::new(),
                                    qself: None,
                                    path:  Path {
                                        leading_colon: Some(Default::default()),
                                        segments:      {
                                            let mut segs = Punctuated::new();
                                            segs.push(PathSegment {
                                                ident:     Ident::new("std", Span::call_site()),
                                                arguments: PathArguments::None,
                                            });
                                            segs.push(PathSegment {
                                                ident:     Ident::new("primitive", Span::call_site()),
                                                arguments: PathArguments::None,
                                            });
                                            segs.push(PathSegment { ident: Ident::new("u8", Span::call_site()), arguments: PathArguments::None });
                                            segs
                                        },
                                    },
                                })));
                                args
                            },
                            gt_token: Default::default(),
                        }),
                    });
                    segs
                },
            },
        }))
    }
}



/// Given a set of [`OuterAttr`]s, generates the correct whitespace-pop implementations.
///
/// # Returns
/// The "init" implementation, compiling and setting up the regex; and the implementation doing a
/// pop on a variable called `input`.
pub fn whitespace_pop_impl<A: OuterAttribute>(input: &'static str, attrs: &[A]) -> (TokenStream2, TokenStream2) {
    let input = Ident::new(input, Span::call_site());
    let whitespace_init = OuterAttribute::has_whitespace(attrs)
        .map(|r| match r {
            // Use the given regular expression
            Some(regex) => quote! {
                static WHITESPACE_REGEX: ::std::sync::LazyLock<Regex> = ::std::sync::LazyLock::new(|| ::std::result::Result::unwrap_or_else(Regex::new(#regex), |err| ::std::panic!("Failed to compile regex {:?}: {err}", #regex)));
                let whitespace_regex = &*WHITESPACE_REGEX;
            },
            // Default to the global whitespace one
            None => quote! {
                let whitespace_regex = &*::ast_toolkit2::nibble::convert::WHITESPACE_REGEX;
            },
        }).unwrap_or_else(TokenStream2::new);
    let whitespace_pop: TokenStream2 = if !whitespace_init.is_empty() {
        quote! {
            let #input = if let Some(m) = Regex::find(whitespace_regex, ::ast_toolkit2::nibble::Slice::as_slice(&#input)) {
                ::ast_toolkit2::nibble::Slice::slice(#input, Match::end(&m)..)
            } else {
                #input
            };
        }
    } else {
        TokenStream2::new()
    };
    (whitespace_init, whitespace_pop)
}
