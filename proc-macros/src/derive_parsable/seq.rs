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
use syn::{Error, Field, Fields, Generics, Ident, LitStr, Token, Type, TypeGenerics, Visibility};

use super::{FieldAttr, OuterAttr, inject_parsable_bound, parse_parse_attrs, whitespace_pop_impl};


/***** HELPER FUNCTIONS *****/
pub fn cook_up_error_mod_name(attrs: &[OuterAttr], ident: &Ident) -> Ident {
    // Check: did the user tell us?
    if let Some(name) = OuterAttr::has_mod(attrs) {
        return name.clone();
    }

    // Generate an identifier name
    let error_mod_name = ident.to_string().to_case(Case::Snake);
    let error_mod_name = if syn::parse_str::<Ident>(&error_mod_name).is_ok() { error_mod_name } else { format!("{error_mod_name}_tys") };

    // Done, create identifier out of it
    Ident::new(&error_mod_name, ident.span())
}





/***** HELPERS *****/
pub enum StructTy {
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



pub struct FieldInfo {
    pub name: Ident,
    pub camel_name: Ident,
    pub ty: Type,
    pub as_ty: Option<Type>,
    pub try_as: bool,
    pub skip: bool,
}
impl FieldInfo {
    /// Returns the type we're actually parsing.
    #[inline]
    pub fn parsed_ty(&self) -> &Type { self.as_ty.as_ref().unwrap_or(&self.ty) }

    /// Returns the as-type name of the extra error variant, if applicable.
    #[inline]
    pub fn as_error_name(&self, span: Span) -> Ident { Ident::new(&format!("{}TryFrom", self.camel_name), span) }
}
impl TryFrom<(usize, Field)> for FieldInfo {
    type Error = Error;

    #[inline]
    fn try_from((i, field): (usize, Field)) -> Result<Self, Self::Error> {
        let span = field.span();
        let Field { attrs, vis: _, modifiers: _, ident, colon_token: _, ty, default: _ } = field;

        // Read attributes for this field
        let field_attrs: Vec<FieldAttr> = parse_parse_attrs(&attrs)?;
        let try_as_ty: Option<&Type> = FieldAttr::has_try_as(&field_attrs);
        let as_ty: Option<Type> = FieldAttr::has_as(&field_attrs).or(try_as_ty).cloned();
        let try_as: bool = try_as_ty.is_some();
        let skip: bool = FieldAttr::has_skip(&field_attrs);

        // Compute some names for this field
        let name: Ident = ident.unwrap_or_else(|| Ident::new(&format!("field{i}"), span));
        let camel_name: Ident = Ident::new(&name.to_string().to_case(Case::Pascal), name.span());

        // Done!
        Ok(Self { name, camel_name, ty, as_ty, try_as, skip })
    }
}



/// Everything we need to know about generating a formatter impl.
///
/// Note this is *either* a given string, or a generated formatter.
pub enum FormatterImpl<'a> {
    Given(&'a LitStr),
    Generated(&'a [FieldInfo]),
    Single(&'a FieldInfo),
    None,
}
impl<'a> FormatterImpl<'a> {
    /// Builds a new Self.
    pub fn new<'o, 'f>(outer_attrs: &'o [OuterAttr], fields: &'f [FieldInfo]) -> Self
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
    fn def<'s, 'g>(&self, ident: &'s Ident, gens: &'g Generics) -> FormatterImplDef<'s, 'g, '_, 'a> { FormatterImplDef(ident, gens, self) }

    /// Returns a generator for the typename for the formatter's type.
    pub fn ty<'e, 't, 'g>(&self, error_mod: &'e Ident, ty_gen: &'t TypeGenerics<'g>) -> FormatterImplTy<'e, 't, 'g, '_, 'a> {
        FormatterImplTy(error_mod, ty_gen, self)
    }

    /// Returns a generator for the instantiation of whatever type this FormatterImpl is.
    fn inst<'e>(&self, error_mod: &'e Ident) -> FormatterImplInst<'e, '_, 'a> { FormatterImplInst(error_mod, self) }
}

struct FormatterImplDef<'i, 'g, 's, 'a>(&'i Ident, &'g Generics, &'s FormatterImpl<'a>);
impl<'i, 'g, 's, 'a> ToTokens for FormatterImplDef<'i, 'g, 's, 'a> {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        // Only do something if we're generated
        let FormatterImpl::Generated(fields) = &self.2 else { return };

        // Compute the definition of every field in the formatter...
        let fields_def: TokenStream2 = fields
            .into_iter()
            .filter(|field| !field.skip)
            .map(|field| {
                let name = &field.name;
                let ty = field.parsed_ty();
                quote! { pub #name: <#ty as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::Formatter, }
            })
            .collect();

        // Compute the `Display` implementation for every field...
        let mut iter = fields.into_iter().filter(|field| !field.skip);
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

pub struct FormatterImplTy<'e, 't, 'g, 's, 'a>(&'e Ident, &'t TypeGenerics<'g>, &'s FormatterImpl<'a>);
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

            FormatterImpl::Generated(fields) => {
                let error_mod = self.0;
                let fmt_inst: TokenStream2 = fields
                    .into_iter()
                    .filter(|f| !f.skip)
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
pub enum ErrorImpl<'a> {
    Generated(&'a [FieldInfo]),
    Single(&'a FieldInfo),
    None,
}
impl<'a> ErrorImpl<'a> {
    /// Builds a new Self.
    pub fn new(fields: &'a [FieldInfo]) -> Self {
        // Check if we need to generate any formatter.
        //
        // This is NOT the case if:
        // - There are no fields; or
        // - There is exactly one unskipped field that does not `as`.
        //
        // Note that this is slightly more complicated than the formatter bc of as-conversion
        // errors.
        let nonskipped_field_count: usize = fields.into_iter().filter(|f| !f.skip).count();
        if nonskipped_field_count > 1 || fields.into_iter().any(|f| f.try_as) {
            Self::Generated(fields)
        } else if nonskipped_field_count == 1 {
            Self::Single(fields.into_iter().filter(|f| !f.skip).next().unwrap())
        } else {
            Self::None
        }
    }


    /// Returns a generator for the type declaration of this ErrorImpl.
    fn def<'i, 'g>(&self, ident: &'i Ident, gens: &'g Generics) -> ErrorImplDef<'i, 'g, '_, 'a> { ErrorImplDef(ident, gens, self) }

    /// Returns a generator for the typename for the formatter's type.
    pub fn ty<'e, 't, 'g>(&self, error_mod: &'e Ident, ty_gen: &'t TypeGenerics<'g>) -> ErrorImplTy<'e, 't, 'g, '_, 'a> {
        ErrorImplTy(error_mod, ty_gen, self)
    }
}

struct ErrorImplDef<'i, 'g, 's, 'a>(&'i Ident, &'g Generics, &'s ErrorImpl<'a>);
impl<'i, 'g, 's, 'a> ToTokens for ErrorImplDef<'i, 'g, 's, 'a> {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        // NOTE: We only generate something if there is something to generate (:O)
        let ErrorImpl::Generated(fields) = self.2 else { return };

        // Generate the definition of each variant
        let vars_def: TokenStream2 = fields
            .into_iter()
            .filter(|field| !field.skip)
            .map(|field| {
                let name = &field.camel_name;
                let ty = field.parsed_ty();
                let mut res = quote! { #name(<#ty as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::Error), };
                if let Some(as_ty) = &field.as_ty && field.try_as {
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
            .filter(|field| !field.skip)
            .map(|field| {
                let name = &field.camel_name;
                let mut res = quote! { Self::#name(err) => ::std::fmt::Display::fmt(err, f), };
                if field.as_ty.is_some() && field.try_as {
                    let as_error_name = field.as_error_name(field.name.span());
                    res.extend(quote! { Self::#as_error_name { from, to, .. } => ::std::write!(f, "Failed to convert from {from} to {to}"), });
                }
                res
            })
            .collect();

        // Generate the `Error` implementation for every variant
        let source_impl: TokenStream2 = fields
            .into_iter()
            .filter(|field| !field.skip)
            .map(|field| {
                let name = &field.camel_name;
                let mut res = quote! { Self::#name(err) => ::std::error::Error::source(err), };
                if field.as_ty.is_some() && field.try_as {
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

pub struct ErrorImplTy<'e, 't, 'g, 's, 'a>(&'e Ident, &'t TypeGenerics<'g>, &'s ErrorImpl<'a>);
impl<'e, 't, 'g, 's, 'a> ToTokens for ErrorImplTy<'e, 't, 'g, 's, 'a> {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self.2 {
            ErrorImpl::Generated(_) => {
                let error_mod = self.0;
                let ty_gen = self.1;
                tokens.extend(quote! { #error_mod::Error #ty_gen })
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
/// Generates an implementation for the module, i.e., auxillary types.
pub fn generate_mod_impl(error_mod: &Ident, vis: &Visibility, ident: &Ident, gens: &Generics, fmt: &FormatterImpl, err: &ErrorImpl) -> TokenStream2 {
    // Generate the module impl
    if matches!(fmt, FormatterImpl::Generated(_)) || matches!(err, ErrorImpl::Generated(_)) {
        let fmt_def = fmt.def(&ident, &gens);
        let err_def = err.def(&ident, &gens);
        quote! {
            #[doc = ::std::concat!("Helper module for [`", ::std::stringify!(#ident), "`] containing auxillary types.")]
            #vis mod #error_mod {
                use super::*;

                #fmt_def

                #err_def
            }
        }
    } else {
        TokenStream2::new()
    }
}

/// Generates an implementation for the module, i.e., auxillary types.
pub fn generate_parsable_impl(
    error_mod: &Ident,
    outer_attrs: &[OuterAttr],
    ident: &Ident,
    mut gens: Generics,
    struct_ty: StructTy,
    fields: &[FieldInfo],
    fmt: &FormatterImpl,
    err: &ErrorImpl,
) -> TokenStream2 {
    let (whitespace_init, whitespace_pop) = whitespace_pop_impl("rem", outer_attrs);

    // Generate the main implementation based on the field infos
    let mut parse_impl = TokenStream2::new();
    for field in fields {
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
            let og_ty = &field.ty;
            if !field.try_as {
                parse_impl.extend(quote! {
                    let #name: #og_ty = ::std::convert::From::from(#name);
                });
            } else {
                let as_error_name = field.as_error_name(field.name.span());
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
    }

    // Generate the instantiation at the end of the parse
    let inst_impl: Punctuated<TokenStream2, Token![,]> = fields.iter().map(|field| field.name.to_token_stream()).collect();
    let inst_impl = match struct_ty {
        StructTy::Named => quote! { #ident { #inst_impl } },
        StructTy::Unnamed => quote! { #ident(#inst_impl) },
        StructTy::Unit => quote! { #ident },
    };

    // Prepare generics by inserting the bound
    inject_parsable_bound(&mut gens);
    let (impl_gen, ty_gen, where_bounds) = gens.split_for_impl();

    // Generate the parse impl
    let fmt_ty = fmt.ty(&error_mod, &ty_gen);
    let fmt_inst = fmt.inst(&error_mod);
    let err_ty = err.ty(&error_mod, &ty_gen);
    quote! {
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
    }
}



/// Handles the implementation.
pub fn handle_impl(outer_attrs: &[OuterAttr], vis: Visibility, ident: Ident, gens: Generics, fields: Fields) -> Result<TokenStream2, Error> {
    // Do some outer attribute processing
    let error_mod = cook_up_error_mod_name(outer_attrs, &ident);

    // First, analyze each field and remember what we need to keep
    let struct_ty = StructTy::from(&fields);
    let fields = fields.into_iter().enumerate().map(FieldInfo::try_from).collect::<Result<Vec<FieldInfo>, Error>>()?;

    // Decide on the formatter & error impls
    let fmt = FormatterImpl::new(outer_attrs, &fields);
    let err = ErrorImpl::new(&fields);

    // Generate the mod implementation & main implementation
    let mod_impl = generate_mod_impl(&error_mod, &vis, &ident, &gens, &fmt, &err);
    let main_impl = generate_parsable_impl(&error_mod, outer_attrs, &ident, gens, struct_ty, &fields, &fmt, &err);

    // Done
    Ok(quote! { #mod_impl #main_impl })
}
