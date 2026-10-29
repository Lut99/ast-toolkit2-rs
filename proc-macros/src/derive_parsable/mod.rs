//  DERIVE PARSABLE.rs
//    by Lut99
//
//  Description:
//!   Implements the derive logic of the `nibble` parser.
//

// Submodules
mod aux;
mod branch;
mod lit;
mod seq;
mod term;

// Imports
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream, Parser as _};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned as _;
use syn::{Data, DataUnion, DeriveInput, Error, Generics, Ident, LitStr, Meta, Token, Type};


/***** HELPER FUNCTIONS *****/
/// Parses the first `#[parse(...)]`-attribute layer away.
fn parse_parse_attrs<A: Attribute>(attrs: &[syn::Attribute]) -> Result<Vec<A>, Error> {
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



/// Master function for generating the "as"-implementation of a type; completely parse it as some
/// other type, then convert it.
fn generate_as_impl(ident: &Ident, gens: &Generics, other: &Type) -> TokenStream2 {
    let (impl_gen, ty_gen, where_clause) = gens.split_for_impl();
    quote! {
        impl #impl_gen ::ast_toolkit2::nibble::Parsable<::std::primitive::u8> #ty_gen for #ident #where_clause {
            type Formatter = <#other as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::Formatter;
            type Error = <#other as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::Error;

            #[inline]
            fn expects() -> Self::Formatter {
                <#other as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::expects()
            }

            #[inline]
            fn parse(input: ::ast_toolkit2::nibble::Slice<::std::primitive::u8>) -> ::std::result::Result<(Self, ::ast_toolkit2::nibble::Slice<::std::primitive::u8>), ::ast_toolkit2::nibble::error::NibbleError<Self::Formatter, Self::Error>> {
                ::std::result::Result::map(<#other as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::parse(input), |(other, rem)| (::std::convert::From::from(other), rem))
            }
        }
    }
}

/// Master function for generating the "as"-implementation of a type; completely parse it as some
/// other type, then convert it.
fn generate_try_as_impl(ident: &Ident, gens: &Generics, other: &Type) -> TokenStream2 {
    let (impl_gen, ty_gen, where_clause) = gens.split_for_impl();
    quote! {
        impl #impl_gen ::ast_toolkit2::nibble::Parsable<::std::primitive::u8> #ty_gen for #ident #where_clause {
            type Formatter = <#other as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::Formatter;
            type Error = ::ast_toolkit2::error::contrib::TryAsError<
                <#other as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::Error,
                <#ident as ::std::convert::TryFrom>::Error,
            >;

            #[inline]
            fn expects() -> Self::Formatter {
                <#other as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::expects()
            }

            #[inline]
            fn parse(input: ::ast_toolkit2::nibble::Slice<::std::primitive::u8>) -> ::std::result::Result<(Self, ::ast_toolkit2::nibble::Slice<::std::primitive::u8>), ::ast_toolkit2::nibble::error::NibbleError<Self::Formatter, Self::Error>> {
                ::std::result::Result::and_then(
                    ::std::result::Result::map_err(
                        <#other as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::parse(input),
                        ::ast_toolkit2::error::contrib::TryAsError::Parse,
                    ),
                    |(other, rem)| ::std::result::Result::Ok((
                        ::std::convert::TryFrom::try_from(other).map_err(::ast_toolkit2::error::contrib::TryAsError::TryFrom)?,
                        rem,
                    ))
                )
            }
        }
    }
}





/***** HELPERS *****/
trait Attribute: Parse {
    fn check_consistency(attrs: &[Self]) -> Result<(), Error>;
}
trait OuterAttribute: Attribute {
    fn has_as(attrs: &[Self]) -> Option<&Type>;
    fn has_expects(attrs: &[Self]) -> Option<&LitStr>;
    fn has_lit(attrs: &[Self]) -> Option<&LitStr>;
    fn has_mod(attrs: &[Self]) -> Option<&Ident>;
    fn has_sep_check(attrs: &[Self]) -> Option<&LitStr>;
    fn has_term(attrs: &[Self]) -> Option<&LitStr>;
    fn has_try_as(attrs: &[Self]) -> Option<&Type>;
    fn has_whitespace(attrs: &[Self]) -> Option<&Option<LitStr>>;
}



/// Defines- and parses outer attributes.
enum OuterAttr {
    /// Parses another type, then uses from to turn A into B.
    As { other: Type },
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
    /// Parses another type, then uses try_from to turn A into B.
    TryAs { other: Type },
    /// Adds additional parsers accepting any whitespace.
    Whitespace { regex: Option<LitStr> },
}

// Checking
impl OuterAttribute for OuterAttr {
    #[inline]
    fn has_as(attrs: &[Self]) -> Option<&Type> {
        attrs.iter().find_map(|a| match a {
            Self::As { other } => Some(other),
            _ => None,
        })
    }
    #[inline]
    fn has_expects(attrs: &[Self]) -> Option<&LitStr> {
        attrs.iter().find_map(|a| match a {
            Self::Expects { what } => Some(what),
            _ => None,
        })
    }
    #[inline]
    fn has_lit(attrs: &[Self]) -> Option<&LitStr> {
        attrs.iter().find_map(|a| match a {
            Self::Lit { regex } => Some(regex),
            _ => None,
        })
    }
    #[inline]
    fn has_mod(attrs: &[Self]) -> Option<&Ident> {
        attrs.iter().find_map(|a| match a {
            Self::Mod { name } => Some(name),
            _ => None,
        })
    }
    #[inline]
    fn has_sep_check(attrs: &[Self]) -> Option<&LitStr> {
        attrs.iter().find_map(|a| match a {
            Self::SepCheck { regex } => Some(regex),
            _ => None,
        })
    }
    #[inline]
    fn has_term(attrs: &[Self]) -> Option<&LitStr> {
        attrs.iter().find_map(|a| match a {
            Self::Term { tag } => Some(tag),
            _ => None,
        })
    }
    #[inline]
    fn has_try_as(attrs: &[Self]) -> Option<&Type> {
        attrs.iter().find_map(|a| match a {
            Self::TryAs { other } => Some(other),
            _ => None,
        })
    }
    #[inline]
    fn has_whitespace(attrs: &[Self]) -> Option<&Option<LitStr>> {
        attrs.iter().find_map(|a| match a {
            Self::Whitespace { regex } => Some(regex),
            _ => None,
        })
    }
}
impl Attribute for OuterAttr {
    /// Check if the list of attributes makes sense.
    fn check_consistency(attrs: &[Self]) -> Result<(), Error> {
        if let Some(a) = Self::has_as(attrs)
            && (Self::has_term(attrs).is_some() || Self::has_lit(attrs).is_some())
        {
            return Err(Error::new(a.span(), "`as` can only be used without `term` or `sep`"));
        }
        if let Some(t) = Self::has_try_as(attrs)
            && (Self::has_term(attrs).is_some() || Self::has_lit(attrs).is_some())
        {
            return Err(Error::new(t.span(), "`try_as` can only be used without `term` or `sep`"));
        }
        if let Some(a) = Self::has_as(attrs)
            && Self::has_try_as(attrs).is_some()
        {
            return Err(Error::new(a.span(), "`as` and `try_as` are mutually exclusive"));
        }
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
        if ident == "as" {
            input.parse::<Token![=]>()?;
            Ok(Self::As { other: input.parse()? })
        } else if ident == "expects" || ident == "expected" {
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
        } else if ident == "try_as" {
            input.parse::<Token![=]>()?;
            Ok(Self::TryAs { other: input.parse()? })
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
enum VariantAttr {
    /// Represents a custom expects string.
    Expects { what: LitStr },
    /// Represents that the struct is a literal.
    Lit { regex: LitStr },
    /// Defines the name to use for the resulting module.
    Mod { name: Ident },
    /// Represents that we check that a token is not continued by...
    SepCheck { regex: LitStr },
    /// It's to skip.
    Skip,
    /// Represents that the struct is a terminal.
    Term { tag: LitStr },
    /// Adds additional parsers accepting any whitespace.
    Whitespace { regex: Option<LitStr> },
}

// Checking
impl VariantAttr {
    #[inline]
    fn has_skip(attrs: &[Self]) -> bool { attrs.iter().any(|a| matches!(a, Self::Skip)) }
}
impl OuterAttribute for VariantAttr {
    #[inline]
    fn has_as(_attrs: &[Self]) -> Option<&Type> { None }
    #[inline]
    fn has_expects(attrs: &[Self]) -> Option<&LitStr> {
        attrs.iter().find_map(|a| match a {
            Self::Expects { what } => Some(what),
            _ => None,
        })
    }
    #[inline]
    fn has_lit(attrs: &[Self]) -> Option<&LitStr> {
        attrs.iter().find_map(|a| match a {
            Self::Lit { regex } => Some(regex),
            _ => None,
        })
    }
    #[inline]
    fn has_mod(attrs: &[Self]) -> Option<&Ident> {
        attrs.iter().find_map(|a| match a {
            Self::Mod { name } => Some(name),
            _ => None,
        })
    }
    #[inline]
    fn has_sep_check(attrs: &[Self]) -> Option<&LitStr> {
        attrs.iter().find_map(|a| match a {
            Self::SepCheck { regex } => Some(regex),
            _ => None,
        })
    }
    #[inline]
    fn has_term(attrs: &[Self]) -> Option<&LitStr> {
        attrs.iter().find_map(|a| match a {
            Self::Term { tag } => Some(tag),
            _ => None,
        })
    }
    #[inline]
    fn has_try_as(_attrs: &[Self]) -> Option<&Type> { None }
    #[inline]
    fn has_whitespace(attrs: &[Self]) -> Option<&Option<LitStr>> {
        attrs.iter().find_map(|a| match a {
            Self::Whitespace { regex } => Some(regex),
            _ => None,
        })
    }
}
impl Attribute for VariantAttr {
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
impl Parse for VariantAttr {
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
        } else if ident == "skip" {
            Ok(Self::Skip)
        } else if ident == "whitespace" {
            let regex = if input.parse::<Token![=]>().is_ok() { Some(input.parse()?) } else { None };
            Ok(Self::Whitespace { regex })
        } else {
            Err(Error::new(ident.span(), &format!("Unknown `parse` variant attribute {:?}", ident.to_string())))
        }
    }
}



/// Defines- and parses field attributes.
enum FieldAttr {
    /// Parses another type, then uses from to turn A into B.
    As { other: Type },
    /// Skips this field, using its default implementation instead.
    Skip,
    /// Parses another type, then uses try_from to turn A into B.
    TryAs { other: Type },
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
    fn has_try_as(attrs: &[Self]) -> Option<&Type> {
        attrs.iter().find_map(|a| match a {
            Self::TryAs { other } => Some(other),
            _ => None,
        })
    }

    #[inline]
    fn has_skip(attrs: &[Self]) -> bool { attrs.iter().find(|a| matches!(a, Self::Skip)).is_some() }
}
impl Attribute for FieldAttr {
    /// Check if the list of attributes makes sense.
    fn check_consistency(attrs: &[Self]) -> Result<(), Error> {
        if let Some(other) = Self::has_as(attrs)
            && Self::has_try_as(attrs).is_some()
        {
            return Err(Error::new(other.span(), "`as` and `try_as` are mutually exclusive"));
        }
        Ok(())
    }
}

// Parsing
impl Parse for FieldAttr {
    fn parse(input: ParseStream) -> Result<Self, Error> {
        // The stream must begin with an identifier
        let ident: Ident = Ident::parse_any(input)?;
        if ident == "as" {
            input.parse::<Token![=]>()?;
            Ok(Self::As { other: input.parse()? })
        } else if ident == "try_as" {
            input.parse::<Token![=]>()?;
            Ok(Self::TryAs { other: input.parse()? })
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

    // In all cases, simply defer to `TryFrom` if the user asks us to
    if let Some(other) = OuterAttr::has_as(&attrs) {
        return Ok(generate_as_impl(&ident, &generics, other));
    }
    if let Some(other) = OuterAttr::has_try_as(&attrs) {
        return Ok(generate_try_as_impl(&ident, &generics, other));
    }

    // Handle each one individually
    match data {
        Data::Struct(s) => {
            if let Some(tag) = OuterAttr::has_term(&attrs) {
                // Handle as a keyword token
                term::handle_impl(&attrs, tag, &ident, &generics, &s.fields)
            } else if let Some(regex) = OuterAttr::has_lit(&attrs) {
                // Handle as a literal token
                lit::handle_impl(&attrs, regex, &ident, &generics, &s.fields)
            } else {
                // Handle as a sequence
                seq::handle_impl(&attrs, vis, ident, generics, s.fields)
            }
        },
        Data::Enum(e) => {
            // Handle as a branch
            branch::handle_impl(&attrs, vis, ident, generics, e)
        },
        Data::Union(DataUnion { union_token, .. }) => Err(Error::new(union_token.span, "Can only derive `Parsable` on structs or enums")),
    }
}
