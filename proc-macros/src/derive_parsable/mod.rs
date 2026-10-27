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
use syn::parse::Parser as _;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned as _;
use syn::{Attribute, Data, DataUnion, DeriveInput, Error, Expr, ExprLit, Ident, Lit, LitStr, Meta, Token};


/***** HELPER FUNCTIONS *****/
/// Parses the first `#[parse(...)]`-attribute layer away.
fn parse_parse_attrs(attrs: &[Attribute]) -> Result<Vec<Meta>, Error> {
    let mut res = Vec::new();
    for attr in attrs {
        match &attr.meta {
            // Lists
            Meta::List(l) if l.path.is_ident("parse") => {
                // Parse the meta as further metas
                let inner: Punctuated<Meta, Token![,]> = Punctuated::parse_terminated.parse2(l.tokens.clone())?;
                res.extend(inner.into_iter());
            },

            // Ignore
            Meta::List(_) => continue,
            Meta::NameValue(_) => continue,
            Meta::Path(_) => continue,
        }
    }
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





/***** HELPERS *****/
/// Defines- and parses outer attributes.
enum OuterAttr {
    /// Represents a custom expects string.
    Expects { what: LitStr },
    /// Represents that the struct is a literal.
    Lit { regex: LitStr },
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

    /// Check if the list of attributes makes sense.
    fn check_consistency(attrs: &[Self]) -> Result<(), Error> {
        if let Some(lit) = Self::has_lit(attrs)
            && Self::has_term(attrs).is_some()
        {
            return Err(Error::new(lit.span(), "`lit` and `term` are mutually exclusive"));
        }
        if let Some(lit) = Self::has_sep_check(attrs)
            && !Self::has_term(attrs).is_some()
        {
            return Err(Error::new(lit.span(), "`sep` can only be used with `term`"));
        }
        Ok(())
    }
}

// Parsing
impl OuterAttr {
    /// Parses it from a list of [`Meta`]s.
    fn parse(metas: Vec<Meta>) -> Result<Vec<Self>, Error> {
        let mut res = Vec::with_capacity(metas.len());
        for meta in metas {
            match meta {
                Meta::NameValue(nv) if nv.path.is_ident("expects") || nv.path.is_ident("expected") => {
                    let Expr::Lit(ExprLit { attrs: _, lit: Lit::Str(what) }) = nv.value else {
                        return Err(Error::new(nv.value.span(), "Expected a string literal"));
                    };
                    res.push(Self::Expects { what });
                },
                Meta::NameValue(nv) if nv.path.is_ident("lit") || nv.path.is_ident("literal") => {
                    let Expr::Lit(ExprLit { attrs: _, lit: Lit::Str(regex) }) = nv.value else {
                        return Err(Error::new(nv.value.span(), "Expected a string literal"));
                    };
                    res.push(Self::Lit { regex });
                },
                Meta::NameValue(nv) if nv.path.is_ident("term") || nv.path.is_ident("terminal") => {
                    let Expr::Lit(ExprLit { attrs: _, lit: Lit::Str(tag) }) = nv.value else {
                        return Err(Error::new(nv.value.span(), "Expected a string literal"));
                    };
                    res.push(Self::Term { tag });
                },
                Meta::NameValue(nv) if nv.path.is_ident("sep") || nv.path.is_ident("sep_check") => {
                    let Expr::Lit(ExprLit { attrs: _, lit: Lit::Str(regex) }) = nv.value else {
                        return Err(Error::new(nv.value.span(), "Expected a string literal"));
                    };
                    res.push(Self::SepCheck { regex });
                },
                Meta::NameValue(nv) if nv.path.is_ident("whitespace") => {
                    let Expr::Lit(ExprLit { attrs: _, lit: Lit::Str(regex) }) = nv.value else {
                        return Err(Error::new(nv.value.span(), "Expected a string literal"));
                    };
                    res.push(Self::Whitespace { regex: Some(regex) });
                },

                Meta::Path(p) if p.is_ident("whitespace") => res.push(Self::Whitespace { regex: None }),

                Meta::List(l) => {
                    return Err(Error::new(
                        l.path.span(),
                        &format!("Unknown attribute {}", l.path.get_ident().map(|i| i.to_string()).unwrap_or("<no ident>".into())),
                    ));
                },
                Meta::NameValue(nv) => {
                    return Err(Error::new(
                        nv.path.span(),
                        &format!("Unknown attribute {}", nv.path.get_ident().map(|i| i.to_string()).unwrap_or("<no ident>".into())),
                    ));
                },
                Meta::Path(p) => {
                    return Err(Error::new(
                        p.span(),
                        &format!("Unknown attribute {}", p.get_ident().map(|i| i.to_string()).unwrap_or("<no ident>".into())),
                    ));
                },
            }
        }
        Ok(res)
    }
}





/***** LIBRARY *****/
/// Main handler for the macro.
pub fn handle(item: TokenStream2) -> Result<TokenStream2, Error> {
    // Collect some general information
    let DeriveInput { attrs, vis, ident, generics, data } = syn::parse2(item)?;
    let attrs: Vec<Meta> = parse_parse_attrs(&attrs)?;
    let attrs: Vec<OuterAttr> = OuterAttr::parse(attrs)?;
    OuterAttr::check_consistency(&attrs)?;

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
                seq::handle_impl(&attrs, vis, ident, generics, s)
            }
        },
        Data::Enum(e) => {
            // Handle as a branch
            todo!()
        },
        Data::Union(DataUnion { union_token, .. }) => Err(Error::new(union_token.span, "Can only derive `Parsable` on structs or enums")),
    }
}
