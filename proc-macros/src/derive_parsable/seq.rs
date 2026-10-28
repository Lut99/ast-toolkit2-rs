//  SEQUENCE.rs
//    by Lut99
//
//  Description:
//!   Implements the logic for structs, i.e., sequential parsers.
//

use std::borrow::Cow;

use convert_case::{Case, Casing as _};
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned as _;
use syn::{Error, Fields, Generics, Ident, LitStr, Path, Token, Type, TypeGenerics, Visibility};

use super::{FieldAttr, OuterAttr, inject_parsable_bound, parse_parse_attrs, whitespace_pop_impl};


/***** HELPER FUNCTIONS *****/
pub fn cook_up_error_mod_name(attrs: &[OuterAttr], ident: &Path) -> Ident {
    // Check: did the user tell us?
    if let Some(name) = OuterAttr::has_mod(attrs) {
        return name.clone();
    }

    // Generate an identifier name
    let mut name = String::new();
    for seg in &ident.segments {
        // We do the whole business with parsing to make sure that we don't accidentally create a keyword
        let error_mod_name = seg.ident.to_string().to_case(Case::Snake);
        let error_mod_name = if syn::parse_str::<Ident>(&error_mod_name).is_ok() { error_mod_name } else { format!("{error_mod_name}_tys") };
        if !name.is_empty() {
            name.push('_');
        }
        name.push_str(&error_mod_name);
    }

    // Done, create identifier out of it
    Ident::new(&name, ident.span())
}





/***** HELPERS *****/
struct FormatterImplDef<'i, 'g, 's, 't>(&'i Path, &'g Generics, &'s FormatterImpl<'t>);
impl<'i, 'g, 's, 't> ToTokens for FormatterImplDef<'i, 'g, 's, 't> {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        // Only do something if we're generated
        let FormatterImpl::Generated(fmt) = &self.2 else { return };

        // Compute each fieldname
        let fmt_def: TokenStream2 = fmt
            .into_iter()
            .map(|(name, ty)| quote! { pub #name: <#ty as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::Formatter, })
            .collect();
        let mut iter = fmt.into_iter();
        let fmt_impl = match iter.next() {
            Some((fname, _)) => {
                let mut res = quote! { ::std::fmt::Display::fmt(#fname, f)?; };
                while let Some((name, _)) = iter.next() {
                    res.extend(quote! {
                        ::std::write!(f, ", then ")?;
                        ::std::fmt::Display::fmt(#name, f)?;
                    });
                }
                res
            },
            None => quote! { ::std::write!(f, "<anything>")?; },
        };
        let fmt_inst: Punctuated<TokenStream2, Token![,]> = fmt.into_iter().map(|(name, _)| name.to_token_stream()).collect();

        // Generate the impl
        let ident = self.0;
        let gens = self.1;
        let (impl_gen, ty_gen, where_bounds) = gens.split_for_impl();
        tokens.extend(quote! {
            #[doc = ::std::concat!("Formatting type for genering the [expects](::ast_toolkit2::nibble::Parsable::expects())-implementation of [`", ::std::stringify!(#ident), "`].")]
            #[derive(::std::clone::Clone, ::std::marker::Copy, ::std::fmt::Debug, ::std::cmp::Eq, ::std::cmp::PartialEq)]
            pub struct Formatter #gens {
                #fmt_def
            }

            #[automatically_derived]
            impl #impl_gen ::std::fmt::Display for Formatter #ty_gen #where_bounds {
                #[inline]
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    let Self { #fmt_inst } = self;
                    #fmt_impl
                    ::std::fmt::Result::Ok(())
                }
            }
        });
    }
}

struct FormatterImplTy<'e, 'g, 'a, 's, 't>(&'e Ident, &'g TypeGenerics<'a>, &'s FormatterImpl<'t>);
impl<'e, 'g, 'a, 's, 't> ToTokens for FormatterImplTy<'e, 'g, 'a, 's, 't> {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match &self.2 {
            FormatterImpl::Given(_) => tokens.extend(quote! { &'static ::std::primitive::str }),

            FormatterImpl::Generated(_) => {
                let error_mod = self.0;
                let ty_gen = self.1;
                tokens.extend(quote! { #error_mod::Formatter #ty_gen });
            },
        }
    }
}

struct FormatterImplInst<'e, 'g, 'a, 's, 't>(&'e Ident, &'g TypeGenerics<'a>, &'s FormatterImpl<'t>);
impl<'e, 'g, 'a, 's, 't> ToTokens for FormatterImplInst<'e, 'g, 'a, 's, 't> {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match &self.2 {
            FormatterImpl::Given(what) => what.to_tokens(tokens),

            FormatterImpl::Generated(fmt) => {
                let error_mod = self.0;
                let ty_gen = self.1;
                let fmt_inst: Punctuated<TokenStream2, Token![,]> = fmt
                    .into_iter()
                    .map(|(name, ty)| quote! { #name: <#ty as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::expects() })
                    .collect();
                tokens.extend(quote! { #error_mod::Formatter #ty_gen { #fmt_inst } });
            },
        }
    }
}


/// Everything we need to know about generating a formatter impl.
///
/// Note this is *either* a given string, or a generated formatter.
enum FormatterImpl<'t> {
    Given(&'t LitStr),
    Generated(Vec<(Ident, Cow<'t, Type>)>),
}

// Updating
impl<'t> FormatterImpl<'t> {
    /// Adds a new field to the formatter.
    ///
    /// If this is already a [`FormatterImpl::Given`], this does nothing.
    fn add_field(&mut self, ident: Ident, ty: Cow<'t, Type>) {
        let Self::Generated(fields) = self else { return };
        fields.push((ident, ty));
    }
}

// Formatting
impl<'t> FormatterImpl<'t> {
    /// Returns a generator for the type declaration of this FormatterImpl.
    fn def<'s, 'g>(&self, ident: &'s Path, gens: &'g Generics) -> FormatterImplDef<'s, 'g, '_, 't> { FormatterImplDef(ident, gens, self) }

    /// Returns a generator for the typename for the formatter's type.
    fn ty<'e, 'g, 'a>(&self, error_mod: &'e Ident, ty_gen: &'g TypeGenerics<'a>) -> FormatterImplTy<'e, 'g, 'a, '_, 't> {
        FormatterImplTy(error_mod, ty_gen, self)
    }

    /// Returns a generator for the instantiation of whatever type this FormatterImpl is.
    fn inst<'e, 'g, 'a>(&self, error_mod: &'e Ident, ty_gen: &'g TypeGenerics<'a>) -> FormatterImplInst<'e, 'g, 'a, '_, 't> {
        FormatterImplInst(error_mod, ty_gen, self)
    }
}



struct ErrorImplDef<'i, 'g, 's, 't>(&'i Path, &'g Generics, &'s ErrorImpl<'t>);
impl<'i, 'g, 's, 't> ToTokens for ErrorImplDef<'i, 'g, 's, 't> {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        // Compute each varname
        // NOTE: We inject an additional error type if we had to convert
        let err_def: TokenStream2 = self
            .2
            .vars
            .iter()
            .map(|(name, ty, as_ty)| {
                if let Some(as_ty) = as_ty {
                    let extra_name = Ident::new(&format!("{}TryFrom", name.to_string()), name.span());
                    quote! {
                        #name(<#as_ty as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::Error),
                        #extra_name { from: &'static ::std::primitive::str, to: &'static ::std::primitive::str, err: <#ty as ::std::convert::TryFrom<#as_ty>>::Error },
                    }
                } else {
                    quote! { #name(<#ty as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::Error), }
                }
            })
            .collect();
        let mut err_fmt: TokenStream2 = self
            .2
            .vars
            .iter()
            .map(|(name, _, as_ty)| {
                let mut res = quote! { Self::#name(err) => ::std::fmt::Display::fmt(err, f), };
                if as_ty.is_some() {
                    let extra_name = Ident::new(&format!("{}TryFrom", name.to_string()), name.span());
                    res.extend(quote! { Self::#extra_name { from, to, .. } => ::std::write!(f, "Failed to convert from {from} to {to}"), })
                }
                res
            })
            .collect();
        let mut err_src: TokenStream2 = self
            .2
            .vars
            .iter()
            .map(|(name, _, as_ty)| {
                let mut res = quote! { Self::#name(err) => ::std::error::Error::source(err), };
                if as_ty.is_some() {
                    let extra_name = Ident::new(&format!("{}TryFrom", name.to_string()), name.span());
                    res.extend(quote! { Self::#extra_name { err, .. } => ::std::option::Option::Some(err), });
                }
                res
            })
            .collect();

        // Take into account empty matches
        if !err_fmt.is_empty() {
            err_fmt = quote! { match self { #err_fmt } };
        } else {
            err_fmt = quote! { ::std::result::Result::Ok(()) }
        }
        if !err_src.is_empty() {
            err_src = quote! { match self { #err_src } };
        } else {
            err_src = quote! { ::std::option::Option::None }
        }

        // Generate the impl
        let ident = self.0;
        let gens = self.1;
        let (impl_gen, ty_gen, where_bounds) = gens.split_for_impl();
        tokens.extend(quote! {
            #[doc = ::std::concat!("Error type for parsing [`", ::std::stringify!(#ident), "`].")]
            #[derive(::std::fmt::Debug, ::std::cmp::Eq, ::std::cmp::PartialEq)]
            pub enum Error #gens {
                #err_def
            }

            #[automatically_derived]
            impl #impl_gen ::std::fmt::Display for Error #ty_gen #where_bounds {
                #[inline]
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    #err_fmt
                }
            }
            #[automatically_derived]
            impl #impl_gen ::std::error::Error for Error #ty_gen #where_bounds {
                #[inline]
                fn source(&self) -> Option<&'_ (dyn 'static + ::std::error::Error)> {
                    #err_src
                }
            }
        });
    }
}

struct ErrorImplTy<'e, 'g, 'a>(&'e Ident, &'g TypeGenerics<'a>);
impl<'e, 'g, 'a> ToTokens for ErrorImplTy<'e, 'g, 'a> {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let error_mod = self.0;
        let ty_gen = self.1;
        tokens.extend(quote! { #error_mod::Error #ty_gen });
    }
}


/// Everything we need to know about generating an error type impl.
struct ErrorImpl<'t> {
    vars: Vec<(Ident, &'t Type, Option<Type>)>,
}

// Updating
impl<'t> ErrorImpl<'t> {
    /// Adds a new variant to the formatter.
    fn add_var(&mut self, ident: Ident, ty: &'t Type, as_ty: Option<Type>) { self.vars.push((ident, ty, as_ty)); }
}

// Formatting
impl<'t> ErrorImpl<'t> {
    /// Returns a generator for the type declaration of this ErrorImpl.
    fn def<'s, 'g>(&self, ident: &'s Path, gens: &'g Generics) -> ErrorImplDef<'s, 'g, '_, 't> { ErrorImplDef(ident, gens, self) }

    /// Returns a generator for the typename for the formatter's type.
    fn ty<'e, 'g, 'a>(error_mod: &'e Ident, ty_gen: &'g TypeGenerics<'a>) -> ErrorImplTy<'e, 'g, 'a> { ErrorImplTy(error_mod, ty_gen) }
}





/***** LIBRARY *****/
/// Handles the implementation.
pub fn handle_impl(outer_attrs: &[OuterAttr], vis: Visibility, ident: Path, mut gens: Generics, fields: Fields) -> Result<TokenStream2, Error> {
    // Check if there already exists an expects-string
    let mut fmt = OuterAttr::has_expects(outer_attrs).map(FormatterImpl::Given).unwrap_or_else(|| FormatterImpl::Generated(Vec::new()));
    let (whitespace_init, whitespace_pop) = whitespace_pop_impl("rem", outer_attrs);

    // Cook up the error module name
    let error_mod = cook_up_error_mod_name(outer_attrs, &ident);

    // Prepare the fields
    let mut inst = Punctuated::<Ident, Token![,]>::new();
    let mut err = ErrorImpl { vars: Vec::new() };
    let mut parse = TokenStream2::new();
    for (i, field) in fields.iter().enumerate() {
        let name: Ident = field.ident.clone().unwrap_or_else(|| Ident::new(&format!("f{i}"), field.span()));
        let camel_name: Ident = Ident::new(&name.to_string().to_case(Case::Pascal), name.span());
        let ty = &field.ty;

        // Parse the field attributes
        let field_attrs: Vec<FieldAttr> = parse_parse_attrs(&field.attrs)?;
        let as_ty: Option<Type> = FieldAttr::has_as(&field_attrs).cloned();
        let target_ty: &Type = as_ty.as_ref().unwrap_or(&ty);
        if FieldAttr::has_skip(&field_attrs) {
            parse.extend(quote! { let #name: #target_ty = ::std::default::Default::default(); });
            if let Some(as_ty) = &as_ty {
                let extra_name = Ident::new(&format!("{camel_name}TryFrom"), camel_name.span());
                parse.extend(quote! {
                    let #name: #ty = ::std::convert::TryFrom::try_from(#name).map_err(|err| #error_mod::Error::#extra_name { from: ::std::any::type_name::<#as_ty>(), to: ::std::any::type_name::<#ty>(), err })?;
                });
            }
            inst.push(name);
            continue;
        }

        // Generate the impl
        parse.extend(quote! {
            let (#name, rem): (#target_ty, ::ast_toolkit2::nibble::Slice<::std::primitive::u8>) = ::ast_toolkit2::nibble::error::ResultExt::map_nerr(
                ::ast_toolkit2::nibble::error::ResultExt::map_fmt(
                    ::ast_toolkit2::nibble::Parsable::<::std::primitive::u8>::parse(rem),
                    |_| <Self as ::ast_toolkit2::nibble::Parsable::<::std::primitive::u8>>::expects()
                ),
                #error_mod::Error::#camel_name
            )?;
        });
        if let Some(as_ty) = &as_ty {
            let extra_name = Ident::new(&format!("{camel_name}TryFrom"), camel_name.span());
            parse.extend(quote! {
                let #name: #ty = ::std::convert::TryFrom::try_from(#name).map_err(|err| #error_mod::Error::#extra_name { from: ::std::any::type_name::<#as_ty>(), to: ::std::any::type_name::<#ty>(), err })?;
            });
        }

        // Save some stuff for later
        inst.push(name.clone());
        fmt.add_field(name, if let Some(as_ty) = &as_ty { Cow::Owned(as_ty.clone()) } else { Cow::Borrowed(ty) });
        err.add_var(camel_name, ty, as_ty);
    }
    let inst: TokenStream2 = match fields {
        Fields::Named(_) => quote! { Self { #inst } },
        Fields::Unnamed(_) => quote! { Self(#inst) },
        Fields::Unit => quote! { Self },
    };

    // Prepare generics by inserting the bound
    inject_parsable_bound(&mut gens);
    let (impl_gen, ty_gen, where_bounds) = gens.split_for_impl();

    // Generate the impl
    let fmt_def = fmt.def(&ident, &gens);
    let fmt_ty = fmt.ty(&error_mod, &ty_gen);
    let fmt_inst = fmt.inst(&error_mod, &ty_gen);
    let err_def = err.def(&ident, &gens);
    let err_ty = ErrorImpl::ty(&error_mod, &ty_gen);
    Ok(quote! {
        #[doc = ::std::concat!("Helper module for [`", ::std::stringify!(#ident), "`] containing auxillary types.")]
        #vis mod #error_mod {
            use super::*;

            #fmt_def

            #err_def
        }

        #[automatically_derived]
        impl #impl_gen ::ast_toolkit2::nibble::Parsable<::std::primitive::u8> for #ident #ty_gen #where_bounds {
            type Formatter = #fmt_ty;

            type Error = #err_ty;

            fn expects() -> Self::Formatter { #fmt_inst }

            fn parse(rem: ::ast_toolkit2::nibble::Slice<::std::primitive::u8>) -> ::std::result::Result<(Self, ::ast_toolkit2::nibble::Slice<::std::primitive::u8>), ::ast_toolkit2::nibble::error::NibbleError<Self::Formatter, Self::Error>> {
                // Pop whitespace first
                #[allow(unused)]
                type Match<'a> = ::ast_toolkit2::nibble::__macros::Match<'a>;
                #[allow(unused)]
                type Regex = ::ast_toolkit2::nibble::__macros::Regex;
                #whitespace_init
                #whitespace_pop

                // Then pop normally
                #parse
                ::std::result::Result::Ok((#inst, rem))
            }
        }
    })
}
