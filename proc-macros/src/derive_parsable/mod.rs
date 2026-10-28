//  DERIVE PARSABLE.rs
//    by Lut99
//
//  Description:
//!   Implements the derive logic of the `nibble` parser.
//

// Submodules
mod branch;
mod lit;
mod seq;
mod term;

// Imports
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream, Parser as _};
use syn::punctuated::Punctuated;
use syn::{
    AngleBracketedGenericArguments, Attribute, Data, DataUnion, DeriveInput, Error, GenericArgument, Generics, Ident, LitStr, Meta, Path,
    PathArguments, PathSegment, Token, TraitBound, Type, TypeParamBound, TypePath,
};


/***** HELPER FUNCTIONS *****/
/// Parses the first `#[parse(...)]`-attribute layer away.
fn parse_parse_attrs<A: Attr>(attrs: &[Attribute]) -> Result<Vec<A>, Error> {
    let mut res = Vec::new();
    for attr in attrs {
        match &attr.meta {
            // Lists
            Meta::List(l) if l.path.is_ident("parse") => {
                // Parse the meta as further metas
                let inner: Punctuated<A, Token![,]> = Punctuated::parse_terminated.parse2(l.tokens.clone())?;
                res.extend(inner.into_iter());
            },

            // Ignore
            Meta::List(_) => continue,
            Meta::NameValue(_) => continue,
            Meta::Path(_) => continue,
        }
    }
    A::check_consistency(&res)?;
    Ok(res)
}



/// Given a set of [`OuterAttr`]s, generates the correct whitespace-pop implementations.
///
/// # Returns
/// The "init" implementation, compiling and setting up the regex; and the implementation doing a
/// pop on a variable called `input`.
fn whitespace_pop_impl(input: &'static str, attrs: &[OuterAttr]) -> (TokenStream2, TokenStream2) {
    let input = Ident::new(input, Span::call_site());
    let whitespace_init = OuterAttr::has_whitespace(attrs)
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



/// Given a set of [`Generics`], assigns each of them the `Parsable`-trait.
///
/// # Arguments
/// - `gens`: The [`Generics`] to inject the additional trait bounds in.
fn inject_parsable_bound(gens: &mut Generics) {
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





/***** HELPERS *****/
trait Attr: Parse {
    fn check_consistency(attrs: &[Self]) -> Result<(), Error>;
}



/// Defines- and parses outer attributes.
enum OuterAttr {
    /// Represents a custom expects string.
    Expects { what: LitStr },
    /// Represents that the struct is a literal.
    Lit { regex: LitStr },
    /// Defines the name to use for the resulting module.
    Mod { name: Ident },
    /// Represents that we check that a token is not continued by...
    SepCheck { regex: LitStr },
    /// Represents that the struct is a terminal.
    Term { tag: LitStr },
    /// Adds additional parsers accepting any whitespace.
    Whitespace { regex: Option<LitStr> },
}

// Checking
impl OuterAttr {
    #[inline]
    fn has_expects(attrs: &[Self]) -> Option<&LitStr> {
        attrs.iter().find_map(|a| match a {
            OuterAttr::Expects { what } => Some(what),
            _ => None,
        })
    }
    #[inline]
    fn has_lit(attrs: &[Self]) -> Option<&LitStr> {
        attrs.iter().find_map(|a| match a {
            OuterAttr::Lit { regex } => Some(regex),
            _ => None,
        })
    }
    #[inline]
    fn has_mod(attrs: &[Self]) -> Option<&Ident> {
        attrs.iter().find_map(|a| match a {
            OuterAttr::Mod { name } => Some(name),
            _ => None,
        })
    }
    #[inline]
    fn has_sep_check(attrs: &[Self]) -> Option<&LitStr> {
        attrs.iter().find_map(|a| match a {
            OuterAttr::SepCheck { regex } => Some(regex),
            _ => None,
        })
    }
    #[inline]
    fn has_term(attrs: &[Self]) -> Option<&LitStr> {
        attrs.iter().find_map(|a| match a {
            OuterAttr::Term { tag } => Some(tag),
            _ => None,
        })
    }
    #[inline]
    fn has_whitespace(attrs: &[Self]) -> Option<&Option<LitStr>> {
        attrs.iter().find_map(|a| match a {
            OuterAttr::Whitespace { regex } => Some(regex),
            _ => None,
        })
    }
}
impl Attr for OuterAttr {
    /// Check if the list of attributes makes sense.
    fn check_consistency(attrs: &[Self]) -> Result<(), Error> {
        if let Some(lit) = Self::has_lit(attrs)
            && Self::has_term(attrs).is_some()
        {
            return Err(Error::new(lit.span(), "`lit` and `term` are mutually exclusive"));
        }
        if let Some(lit) = Self::has_sep_check(attrs)
            && Self::has_term(attrs).is_none()
        {
            return Err(Error::new(lit.span(), "`sep` can only be used with `term`"));
        }
        if let Some(md) = Self::has_mod(attrs)
            && (Self::has_term(attrs).is_some() || Self::has_lit(attrs).is_some())
        {
            return Err(Error::new(md.span(), "`mod` can only be used without `term` or `sep`"));
        }
        Ok(())
    }
}

// Parsing
impl Parse for OuterAttr {
    fn parse(input: ParseStream) -> Result<Self, Error> {
        // The stream must begin with an identifier
        let ident: Ident = Ident::parse_any(input)?;
        if ident == "expects" || ident == "expected" {
            input.parse::<Token![=]>()?;
            Ok(Self::Expects { what: input.parse()? })
        } else if ident == "lit" || ident == "literal" {
            input.parse::<Token![=]>()?;
            Ok(Self::Lit { regex: input.parse()? })
        } else if ident == "mod" || ident == "module" {
            input.parse::<Token![=]>()?;
            Ok(Self::Mod { name: input.parse()? })
        } else if ident == "term" || ident == "terminal" {
            input.parse::<Token![=]>()?;
            Ok(Self::Term { tag: input.parse()? })
        } else if ident == "sep" || ident == "sep_check" {
            input.parse::<Token![=]>()?;
            Ok(Self::SepCheck { regex: input.parse()? })
        } else if ident == "whitespace" {
            let regex = if input.parse::<Token![=]>().is_ok() { Some(input.parse()?) } else { None };
            Ok(Self::Whitespace { regex })
        } else {
            Err(Error::new(ident.span(), &format!("Unknown `parse` outer attribute {:?}", ident.to_string())))
        }
    }
}



/// Defines- and parses variant attributes.
enum VariantAttr {}

// Checking
impl VariantAttr {
    // #[inline]
    // fn has_expects(attrs: &[Self]) -> Option<&LitStr> {
    //     attrs.iter().find_map(|a| match a {
    //         VariantAttr::Expects { what } => Some(what),
    //         _ => None,
    //     })
    // }
}
impl Attr for VariantAttr {
    /// Check if the list of attributes makes sense.
    fn check_consistency(attrs: &[Self]) -> Result<(), Error> { Ok(()) }
}

// Parsing
impl Parse for VariantAttr {
    fn parse(input: ParseStream) -> Result<Self, Error> {
        // The stream must begin with an identifier
        let ident: Ident = Ident::parse_any(input)?;
        Err(Error::new(ident.span(), &format!("Unknown `parse` variant attribute {:?}", ident.to_string())))
    }
}



/// Defines- and parses field attributes.
enum FieldAttr {
    /// Parses another type, then uses try_from to turn A into B.
    As { other: Type },
    /// Skips this field, using its default implementation instead.
    Skip,
}

// Checking
impl FieldAttr {
    #[inline]
    fn has_as(attrs: &[Self]) -> Option<&Type> {
        attrs.iter().find_map(|a| match a {
            Self::As { other } => Some(other),
            _ => None,
        })
    }

    #[inline]
    fn has_skip(attrs: &[Self]) -> bool { attrs.iter().find(|a| matches!(a, Self::Skip)).is_some() }
}
impl Attr for FieldAttr {
    /// Check if the list of attributes makes sense.
    fn check_consistency(attrs: &[Self]) -> Result<(), Error> { Ok(()) }
}

// Parsing
impl Parse for FieldAttr {
    fn parse(input: ParseStream) -> Result<Self, Error> {
        // The stream must begin with an identifier
        let ident: Ident = Ident::parse_any(input)?;
        if ident == "as" {
            input.parse::<Token![=]>()?;
            Ok(Self::As { other: input.parse()? })
        } else if ident == "skip" {
            Ok(Self::Skip)
        } else {
            Err(Error::new(ident.span(), &format!("Unknown `parse` field attribute {:?}", ident.to_string())))
        }
    }
}





/***** LIBRARY *****/
/// Main handler for the macro.
pub fn handle(item: TokenStream2) -> Result<TokenStream2, Error> {
    // Collect some general information
    let DeriveInput { attrs, vis, ident, generics, data } = syn::parse2(item)?;
    let attrs: Vec<OuterAttr> = parse_parse_attrs(&attrs)?;

    // Handle each one individually
    match data {
        Data::Struct(s) => {
            if let Some(tag) = attrs.iter().find_map(|a| match a {
                OuterAttr::Term { tag } => Some(tag),
                _ => None,
            }) {
                // Handle as a keyword token
                term::handle_impl(&attrs, tag, ident, generics, s)
            } else if let Some(regex) = attrs.iter().find_map(|a| match a {
                OuterAttr::Lit { regex } => Some(regex),
                _ => None,
            }) {
                // Handle as a keyword token
                lit::handle_impl(&attrs, regex, ident, generics, s)
            } else {
                // Handle as a sequence
                seq::handle_impl(
                    &attrs,
                    vis,
                    Path {
                        leading_colon: None,
                        segments:      {
                            let mut segs = Punctuated::new();
                            segs.push(PathSegment { ident, arguments: PathArguments::None });
                            segs
                        },
                    },
                    generics,
                    s.fields,
                )
            }
        },
        Data::Enum(e) => {
            // Handle as a branch
            branch::handle_impl(&attrs, vis, ident, generics, e)
        },
        Data::Union(DataUnion { union_token, .. }) => Err(Error::new(union_token.span, "Can only derive `Parsable` on structs or enums")),
    }
}
