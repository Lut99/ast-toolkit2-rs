//  SEQUENCE.rs
//    by Lut99
//
//  Description:
//!   Implements the logic for structs, i.e., sequential parsers.
//

use convert_case::{Case, Casing as _};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, quote};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned as _;
use syn::{
    AngleBracketedGenericArguments, DataStruct, Error, Fields, GenericArgument, Generics, Ident, LitStr, Path, PathArguments, PathSegment, Token,
    TraitBound, TraitBoundModifier, Type, TypeGenerics, TypeParamBound, TypePath, Visibility,
};

use super::{OuterAttr, whitespace_pop_impl};


/***** HELPER FUNCTIONS *****/
/// Given a set of [`Generics`], assigns each of them the `Parsable`-trait.
///
/// # Arguments
/// - `gens`: The [`Generics`] to inject the additional trait bounds in.
fn inject_parsable_bound(gens: &mut Generics) {
    for param in gens.type_params_mut() {
        param.bounds.push(TypeParamBound::Trait(TraitBound {
            paren_token: None,
            modifier: TraitBoundModifier::None,
            lifetimes: None,
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
struct FormatterImplDef<'i, 'g, 's, 't>(&'i Ident, &'g Generics, &'s FormatterImpl<'t>);
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
            None => quote! { ::std::write!(f, "<anything>") },
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
    Generated(Vec<(Ident, &'t Type)>),
}

// Updating
impl<'t> FormatterImpl<'t> {
    /// Adds a new field to the formatter.
    ///
    /// If this is already a [`FormatterImpl::Given`], this does nothing.
    fn add_field(&mut self, ident: Ident, ty: &'t Type) {
        let Self::Generated(fields) = self else { return };
        fields.push((ident, ty));
    }
}

// Formatting
impl<'t> FormatterImpl<'t> {
    /// Returns a generator for the type declaration of this FormatterImpl.
    fn def<'s, 'g>(&self, ident: &'s Ident, gens: &'g Generics) -> FormatterImplDef<'s, 'g, '_, 't> { FormatterImplDef(ident, gens, self) }

    /// Returns a generator for the typename for the formatter's type.
    fn ty<'e, 'g, 'a>(&self, error_mod: &'e Ident, ty_gen: &'g TypeGenerics<'a>) -> FormatterImplTy<'e, 'g, 'a, '_, 't> {
        FormatterImplTy(error_mod, ty_gen, self)
    }

    /// Returns a generator for the instantiation of whatever type this FormatterImpl is.
    fn inst<'e, 'g, 'a>(&self, error_mod: &'e Ident, ty_gen: &'g TypeGenerics<'a>) -> FormatterImplInst<'e, 'g, 'a, '_, 't> {
        FormatterImplInst(error_mod, ty_gen, self)
    }
}



struct ErrorImplDef<'i, 'g, 's, 't>(&'i Ident, &'g Generics, &'s ErrorImpl<'t>);
impl<'i, 'g, 's, 't> ToTokens for ErrorImplDef<'i, 'g, 's, 't> {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        // Compute each varname
        let err_def: TokenStream2 =
            self.2.vars.iter().map(|(name, ty)| quote! { #name(<#ty as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::Error), }).collect();
        let err_fmt: TokenStream2 = self.2.vars.iter().map(|(name, _)| quote! { Self::#name(err) => ::std::fmt::Display::fmt(err, f), }).collect();
        let err_src: TokenStream2 = self.2.vars.iter().map(|(name, _)| quote! { Self::#name(err) => ::std::error::Error::source(err), }).collect();

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

            impl #impl_gen ::std::fmt::Display for Error #ty_gen #where_bounds {
                #[inline]
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    match self {
                        #err_fmt
                    }
                }
            }
            impl #impl_gen ::std::error::Error for Error #ty_gen #where_bounds {
                #[inline]
                fn source(&self) -> Option<&'_ (dyn 'static + ::std::error::Error)> {
                    match self {
                        #err_src
                    }
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
    vars: Vec<(Ident, &'t Type)>,
}

// Updating
impl<'t> ErrorImpl<'t> {
    /// Adds a new variant to the formatter.
    fn add_var(&mut self, ident: Ident, ty: &'t Type) { self.vars.push((ident, ty)); }
}

// Formatting
impl<'t> ErrorImpl<'t> {
    /// Returns a generator for the type declaration of this ErrorImpl.
    fn def<'s, 'g>(&self, ident: &'s Ident, gens: &'g Generics) -> ErrorImplDef<'s, 'g, '_, 't> { ErrorImplDef(ident, gens, self) }

    /// Returns a generator for the typename for the formatter's type.
    fn ty<'e, 'g, 'a>(error_mod: &'e Ident, ty_gen: &'g TypeGenerics<'a>) -> ErrorImplTy<'e, 'g, 'a> { ErrorImplTy(error_mod, ty_gen) }
}





/***** LIBRARY *****/
/// Handles the implementation.
pub fn handle_impl(outer_attrs: &[OuterAttr], vis: Visibility, ident: Ident, mut gens: Generics, data: DataStruct) -> Result<TokenStream2, Error> {
    // Check if there already exists an expects-string
    let mut fmt = OuterAttr::has_expects(outer_attrs).map(FormatterImpl::Given).unwrap_or_else(|| FormatterImpl::Generated(Vec::new()));
    let (whitespace_init, whitespace_pop) = whitespace_pop_impl("rem", outer_attrs);

    // Cook up the error module name
    let error_mod = Ident::new(&ident.to_string().to_case(Case::Snake), ident.span());

    // Prepare the fields
    let mut inst = Punctuated::<Ident, Token![,]>::new();
    let mut err = ErrorImpl { vars: Vec::new() };
    let mut parse = TokenStream2::new();
    for (i, field) in data.fields.iter().enumerate() {
        // Get a name
        let name: Ident = field.ident.clone().unwrap_or_else(|| Ident::new(&format!("f{i}"), field.span()));
        let camel_name: Ident = Ident::new(&name.to_string().to_case(Case::Pascal), name.span());
        let ty = &field.ty;

        // Generate the impl
        parse.extend(quote! {
            let (#name, rem): (#ty, ::ast_toolkit2::nibble::Slice<::std::primitive::u8>) = ::ast_toolkit2::nibble::error::ResultExt::map_nerr(
                ::ast_toolkit2::nibble::error::ResultExt::map_fmt(
                    ::ast_toolkit2::nibble::Parsable::<::std::primitive::u8>::parse(rem),
                    |_| <Self as ::ast_toolkit2::nibble::Parsable::<::std::primitive::u8>>::expects()
                ),
                #error_mod::Error::#camel_name
            )?;
        });

        // Save some stuff for later
        inst.push(name.clone());
        fmt.add_field(name, ty);
        err.add_var(camel_name, ty);
    }
    let inst: TokenStream2 = match &data.fields {
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
