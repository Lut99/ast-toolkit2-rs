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
use syn::{Error, Field, Fields, Generics, Ident, LitStr, Path, Token, Type, TypeGenerics, Visibility};

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
enum StructTy {
    Named,
    Unnamed,
    Unit,
}
impl<'a> From<&'a Fields> for StructTy {
    #[inline]
    fn from(value: &'a Fields) -> Self {
        match value {
            Fields::Named(_) => Self::Named,
            Fields::Unnamed(_) => Self::Unnamed,
            Fields::Unit => Self::Unit,
        }
    }
}



struct FieldInfo {
    name: Ident,
    camel_name: Ident,
    ty: Type,
    as_ty: Option<Type>,
    skip: bool,
}
impl FieldInfo {
    /// Returns the type we're actually parsing.
    #[inline]
    fn parsed_ty(&self) -> &Type { self.as_ty.as_ref().unwrap_or(&self.ty) }

    /// Returns the as-type name of the extra error variant, if applicable.
    #[inline]
    fn as_error_name(&self, span: Span) -> Ident { Ident::new(&format!("{}TryFrom", self.camel_name), span) }
}
impl TryFrom<(usize, Field)> for FieldInfo {
    type Error = Error;

    #[inline]
    fn try_from((i, field): (usize, Field)) -> Result<Self, Self::Error> {
        let span = field.span();
        let Field { attrs, vis: _, modifiers: _, ident, colon_token: _, ty, default: _ } = field;

        // Read attributes for this field
        let field_attrs: Vec<FieldAttr> = parse_parse_attrs(&attrs)?;
        let as_ty: Option<Type> = FieldAttr::has_as(&field_attrs).cloned();
        let skip: bool = FieldAttr::has_skip(&field_attrs);

        // Compute some names for this field
        let name: Ident = ident.unwrap_or_else(|| Ident::new(&format!("field{i}"), span));
        let camel_name: Ident = Ident::new(&name.to_string().to_case(Case::Pascal), name.span());

        // Done!
        Ok(Self { name, camel_name, ty, as_ty, skip })
    }
}



/// Everything we need to know about generating a formatter impl.
///
/// Note this is *either* a given string, or a generated formatter.
enum FormatterImpl<'a> {
    Given(&'a LitStr),
    Generated(&'a [FieldInfo]),
    Single(&'a FieldInfo),
    None,
}
impl<'a> FormatterImpl<'a> {
    /// Builds a new Self.
    fn new<'o, 'f>(outer_attrs: &'o [OuterAttr], fields: &'f [FieldInfo]) -> Self
    where
        'o: 'a,
        'f: 'a,
    {
        // Check if there is an overrulling expects
        if let Some(what) = OuterAttr::has_expects(outer_attrs) {
            return Self::Given(what);
        }

        // Check if we need to generate any formatter.
        //
        // This is NOT the case if:
        // - There are no fields; or
        // - There is exactly one unskipped field.
        //
        // Note that this is slightly simpler than errors, because we don't need it for
        // conversions.
        let actual_field_count: usize = fields.into_iter().filter(|f| !f.skip).count();
        if actual_field_count > 1 {
            Self::Generated(fields)
        } else if actual_field_count == 1 {
            Self::Single(fields.into_iter().filter(|f| !f.skip).next().unwrap())
        } else {
            Self::None
        }
    }


    /// Returns a generator for the type declaration of this FormatterImpl.
    fn def<'s, 'g>(&self, ident: &'s Path, gens: &'g Generics) -> FormatterImplDef<'s, 'g, '_, 'a> { FormatterImplDef(ident, gens, self) }

    /// Returns a generator for the typename for the formatter's type.
    fn ty<'e, 't, 'g>(&self, error_mod: &'e Ident, ty_gen: &'t TypeGenerics<'g>) -> FormatterImplTy<'e, 't, 'g, '_, 'a> {
        FormatterImplTy(error_mod, ty_gen, self)
    }

    /// Returns a generator for the instantiation of whatever type this FormatterImpl is.
    fn inst<'e>(&self, error_mod: &'e Ident) -> FormatterImplInst<'e, '_, 'a> { FormatterImplInst(error_mod, self) }
}

struct FormatterImplDef<'i, 'g, 's, 'a>(&'i Path, &'g Generics, &'s FormatterImpl<'a>);
impl<'i, 'g, 's, 'a> ToTokens for FormatterImplDef<'i, 'g, 's, 'a> {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        // Only do something if we're generated
        let FormatterImpl::Generated(fmt) = &self.2 else { return };

        // Compute the definition of every field in the formatter...
        let fields_def: TokenStream2 = fmt
            .into_iter()
            .map(|field| {
                let name = &field.name;
                let ty = field.parsed_ty();
                quote! { pub #name: <#ty as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::Formatter, }
            })
            .collect();

        // Compute the `Display` implementation for every field...
        let mut iter = fmt.into_iter();
        // SAFETY: This is OK because we only get here if there's something to generate in the first place
        let first = iter.next().unwrap();
        let fname = &first.name;
        let mut fmt_impl: TokenStream2 = quote! { ::std::fmt::Display::fmt(&self.#fname, f)?; };
        while let Some(field) = iter.next() {
            let name = &field.name;
            fmt_impl.extend(quote! {
                ::std::write!(f, ", then ")?;
                ::std::fmt::Display::fmt(&self.#name, f)?;
            });
        }

        // Generate the final impl
        let ident = self.0;
        let gens = self.1;
        let (impl_gen, ty_gen, where_bounds) = gens.split_for_impl();
        tokens.extend(quote! {
            #[doc = ::std::concat!("Formatting type for genering the [expects](::ast_toolkit2::nibble::Parsable::expects())-implementation of [`", ::std::stringify!(#ident), "`].")]
            #[derive(::std::clone::Clone, ::std::marker::Copy, ::std::fmt::Debug, ::std::cmp::Eq, ::std::cmp::PartialEq)]
            pub struct Formatter #gens {
                #fields_def
            }

            #[automatically_derived]
            impl #impl_gen ::std::fmt::Display for Formatter #ty_gen #where_bounds {
                #[inline]
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    #fmt_impl
                    ::std::fmt::Result::Ok(())
                }
            }
        });
    }
}

struct FormatterImplTy<'e, 't, 'g, 's, 'a>(&'e Ident, &'t TypeGenerics<'g>, &'s FormatterImpl<'a>);
impl<'e, 't, 'g, 's, 'a> ToTokens for FormatterImplTy<'e, 't, 'g, 's, 'a> {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match &self.2 {
            FormatterImpl::Given(_) | FormatterImpl::None => tokens.extend(quote! { &'static ::std::primitive::str }),

            FormatterImpl::Single(field) => {
                let ty = field.parsed_ty();
                tokens.extend(quote! { <#ty as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::Formatter })
            },

            FormatterImpl::Generated(_) => {
                let error_mod = self.0;
                let ty_gen = self.1;
                tokens.extend(quote! { #error_mod::Formatter #ty_gen });
            },
        }
    }
}

struct FormatterImplInst<'e, 's, 't>(&'e Ident, &'s FormatterImpl<'t>);
impl<'e, 's, 't> ToTokens for FormatterImplInst<'e, 's, 't> {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match &self.1 {
            FormatterImpl::Given(what) => what.to_tokens(tokens),
            FormatterImpl::None => LitStr::new("<anything>", Span::call_site()).to_tokens(tokens),

            FormatterImpl::Single(field) => {
                let ty = field.parsed_ty();
                tokens.extend(quote! { <#ty as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::expects() });
            },

            FormatterImpl::Generated(fmt) => {
                let error_mod = self.0;
                let fmt_inst: TokenStream2 = fmt
                    .into_iter()
                    .map(|field| {
                        let name = &field.name;
                        let ty = field.parsed_ty();
                        quote! { #name: <#ty as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::expects(), }
                    })
                    .collect();
                tokens.extend(quote! { #error_mod::Formatter { #fmt_inst } });
            },
        }
    }
}



/// Everything we need to know about generating an error type impl.
enum ErrorImpl<'a> {
    Generated(&'a [FieldInfo]),
    Single(&'a FieldInfo),
    None,
}
impl<'a> ErrorImpl<'a> {
    /// Builds a new Self.
    fn new(fields: &'a [FieldInfo]) -> Self {
        // Check if we need to generate any formatter.
        //
        // This is NOT the case if:
        // - There are no fields; or
        // - There is exactly one unskipped field that does not `as`.
        //
        // Note that this is slightly more complicated than the formatter bc of as-conversion
        // errors.
        let nonskipped_field_count: usize = fields.into_iter().filter(|f| !f.skip).count();
        if nonskipped_field_count > 1 || fields.into_iter().any(|f| f.as_ty.is_some()) {
            Self::Generated(fields)
        } else if nonskipped_field_count == 1 {
            Self::Single(fields.into_iter().filter(|f| !f.skip).next().unwrap())
        } else {
            Self::None
        }
    }


    /// Returns a generator for the type declaration of this ErrorImpl.
    fn def<'i, 'g>(&self, ident: &'i Path, gens: &'g Generics) -> ErrorImplDef<'i, 'g, '_, 'a> { ErrorImplDef(ident, gens, self) }

    /// Returns a generator for the typename for the formatter's type.
    fn ty<'e>(&self, error_mod: &'e Ident) -> ErrorImplTy<'e, '_, 'a> { ErrorImplTy(error_mod, self) }
}

struct ErrorImplDef<'i, 'g, 's, 'a>(&'i Path, &'g Generics, &'s ErrorImpl<'a>);
impl<'i, 'g, 's, 'a> ToTokens for ErrorImplDef<'i, 'g, 's, 'a> {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        // NOTE: We only generate something if there is something to generate (:O)
        let ErrorImpl::Generated(fields) = self.2 else { return };

        // Generate the definition of each variant
        let vars_def: TokenStream2 = fields
            .into_iter()
            .map(|field| {
                let name = &field.camel_name;
                let ty = field.parsed_ty();
                let mut res = quote! { #name(<#ty as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::Error), };
                if let Some(as_ty) = &field.as_ty {
                    let og_ty = &field.ty;
                    let as_error_name = field.as_error_name(field.name.span());
                    res.extend(quote! { #as_error_name { from: &'static ::std::primitive::str, to: &'static ::std::primitive::str, err: <#og_ty as ::std::convert::TryFrom<#as_ty>>::Error }, });
                }
                res
            })
            .collect();

        // Generate the `Display` implementation for every variant
        let fmt_impl: TokenStream2 = fields
            .into_iter()
            .map(|field| {
                let name = &field.camel_name;
                let mut res = quote! { Self::#name(err) => ::std::fmt::Display::fmt(err, f), };
                if field.as_ty.is_some() {
                    let as_error_name = field.as_error_name(field.name.span());
                    res.extend(quote! { Self::#as_error_name { from, to, .. } => ::std::write!(f, "Failed to convert from {from} to {to}"), });
                }
                res
            })
            .collect();

        // Generate the `Error` implementation for every variant
        let source_impl: TokenStream2 = fields
            .into_iter()
            .map(|field| {
                let name = &field.camel_name;
                let mut res = quote! { Self::#name(err) => ::std::error::Error::source(err), };
                if field.as_ty.is_some() {
                    let as_error_name = field.as_error_name(field.name.span());
                    res.extend(quote! { Self::#as_error_name { err, .. } => ::std::option::Option::Some(err), });
                }
                res
            })
            .collect();

        // Generate the full impl
        let ident = self.0;
        let gens = self.1;
        let (impl_gen, ty_gen, where_bounds) = gens.split_for_impl();
        tokens.extend(quote! {
            #[doc = ::std::concat!("Error type for parsing [`", ::std::stringify!(#ident), "`].")]
            #[derive(::std::fmt::Debug, ::std::cmp::Eq, ::std::cmp::PartialEq)]
            pub enum Error #gens {
                #vars_def
            }

            #[automatically_derived]
            impl #impl_gen ::std::fmt::Display for Error #ty_gen #where_bounds {
                #[inline]
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    match self {
                        #fmt_impl
                    }
                }
            }
            #[automatically_derived]
            impl #impl_gen ::std::error::Error for Error #ty_gen #where_bounds {
                #[inline]
                fn source(&self) -> Option<&'_ (dyn 'static + ::std::error::Error)> {
                    match self {
                        #source_impl
                    }
                }
            }
        });
    }
}

struct ErrorImplTy<'e, 's, 'a>(&'e Ident, &'s ErrorImpl<'a>);
impl<'e, 's, 'a> ToTokens for ErrorImplTy<'e, 's, 'a> {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self.1 {
            ErrorImpl::Generated(_) => {
                let error_mod = self.0;
                tokens.extend(quote! { #error_mod::Error })
            },
            ErrorImpl::Single(field) => {
                let ty = field.parsed_ty();
                tokens.extend(quote! { <#ty as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::Error })
            },
            ErrorImpl::None => tokens.extend(quote! { ::std::convert::Infallible }),
        }
    }
}





/***** LIBRARY *****/
/// Handles the implementation.
pub fn handle_impl(outer_attrs: &[OuterAttr], vis: Visibility, ident: Path, mut gens: Generics, fields: Fields) -> Result<TokenStream2, Error> {
    // Do some outer attribute processing
    let (whitespace_init, whitespace_pop) = whitespace_pop_impl("rem", outer_attrs);
    let error_mod = cook_up_error_mod_name(outer_attrs, &ident);

    // First, analyze each field and remember what we need to keep
    let struct_ty = StructTy::from(&fields);
    let fields = fields.into_iter().enumerate().map(FieldInfo::try_from).collect::<Result<Vec<FieldInfo>, Error>>()?;

    // Decide on the formatter & error impls
    let fmt = FormatterImpl::new(outer_attrs, &fields);
    let err = ErrorImpl::new(&fields);

    // Generate the main implementation based on the field infos
    let mut parse_impl = TokenStream2::new();
    for field in &fields {
        // Check whether to generate a default implementation, or a field-based implementation
        let name = &field.name;
        let ty = field.parsed_ty();
        if !field.skip {
            // Generate the basic parse
            let inner_impl = quote! {
                ::ast_toolkit2::nibble::error::ResultExt::map_fmt(
                    ::ast_toolkit2::nibble::Parsable::<::std::primitive::u8>::parse(rem),
                    |_| <Self as ::ast_toolkit2::nibble::Parsable::<::std::primitive::u8>>::expects()
                )
            };

            // Decide if we need to map the error, then insert the inner_impl with map or not
            if matches!(err, ErrorImpl::Generated(_)) {
                let error_name = &field.camel_name;
                parse_impl.extend(quote! {
                    let (#name, rem): (#ty, ::ast_toolkit2::nibble::Slice<::std::primitive::u8>) = ::ast_toolkit2::nibble::error::ResultExt::map_nerr(
                        #inner_impl,
                        #error_mod::Error::#error_name
                    )?;
                });
            } else {
                parse_impl.extend(quote! {
                    let (#name, rem): (#ty, ::ast_toolkit2::nibble::Slice<::std::primitive::u8>) = #inner_impl?;
                });
            }
        } else {
            // This one's easy
            parse_impl.extend(quote! { let #name: #ty = ::std::default::Default::default(); });
        }

        // Don't forget to convert types if necessary
        if let Some(as_ty) = &field.as_ty {
            let as_error_name = field.as_error_name(field.name.span());
            let og_ty = &field.ty;
            parse_impl.extend(quote! {
                let #name: #og_ty = ::std::result::Result::map_err(
                    ::std::convert::TryFrom::try_from(#name),
                    |err| #error_mod::Error::#as_error_name {
                        from: ::std::any::type_name::<#as_ty>(),
                        to: ::std::any::type_name::<#og_ty>(),
                        err
                    }
                )?;
            });
        }
    }

    // Generate the instantiation at the end of the parse
    let inst_impl: Punctuated<TokenStream2, Token![,]> = fields.iter().map(|field| field.name.to_token_stream()).collect();
    let inst_impl = match struct_ty {
        StructTy::Named => quote! { Self { #inst_impl } },
        StructTy::Unnamed => quote! { Self(#inst_impl) },
        StructTy::Unit => quote! { Self },
    };

    // Prepare generics by inserting the bound
    inject_parsable_bound(&mut gens);
    let (impl_gen, ty_gen, where_bounds) = gens.split_for_impl();

    // Generate the module impl
    let mut res = TokenStream2::new();
    if matches!(fmt, FormatterImpl::Generated(_)) || matches!(err, ErrorImpl::Generated(_)) {
        let fmt_def = fmt.def(&ident, &gens);
        let err_def = err.def(&ident, &gens);
        res.extend(quote! {
            #[doc = ::std::concat!("Helper module for [`", ::std::stringify!(#ident), "`] containing auxillary types.")]
            #vis mod #error_mod {
                use super::*;

                #fmt_def

                #err_def
            }
        });
    }

    // Generate the parse impl
    let fmt_ty = fmt.ty(&error_mod, &ty_gen);
    let fmt_inst = fmt.inst(&error_mod);
    let err_ty = err.ty(&error_mod);
    res.extend(quote! {
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
                #parse_impl
                ::std::result::Result::Ok((#inst_impl, rem))
            }
        }
    });

    // Done
    Ok(res)
}
