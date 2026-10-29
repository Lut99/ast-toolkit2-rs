//  SEQUENCE.rs
//    by Lut99
//
//  Description:
//!   Implements the logic for structs, i.e., sequential parsers.
//

use convert_case::{Case, Casing as _};
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned as _;
use syn::{Error, Field, Fields, Generics, Ident, Token, Type, Visibility};

use super::aux::{Fmt, FmtInfo, identity_path, inject_parsable_bound, whitespace_pop_impl};
use super::{FieldAttr, OuterAttr, OuterAttribute, parse_parse_attrs};
use crate::derive_parsable::aux::{Err, ErrInfo};


/***** HELPER FUNCTIONS *****/
pub fn cook_up_error_mod_name<A: OuterAttribute>(attrs: &[A], ident: &Ident) -> Ident {
    // Check: did the user tell us?
    if let Some(name) = OuterAttribute::has_mod(attrs) {
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
    pub name:   Ident,
    pub ty:     Type,
    pub as_ty:  Option<Type>,
    pub try_as: bool,
    pub skip:   bool,
}
impl FieldInfo {
    /// Returns the type we're actually parsing.
    #[inline]
    pub fn parsed_ty(&self) -> &Type { self.as_ty.as_ref().unwrap_or(&self.ty) }
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

        // Done!
        Ok(Self { name, ty, as_ty, try_as, skip })
    }
}
impl<'a> From<&'a FieldInfo> for FmtInfo {
    #[inline]
    fn from(value: &'a FieldInfo) -> Self { Self { parsed_ty: value.parsed_ty().clone(), parsed_ty_inst: None, skip: value.skip } }
}
impl<'a> From<&'a FieldInfo> for ErrInfo<'a> {
    #[inline]
    fn from(value: &'a FieldInfo) -> Self { Self::new(&value.name, value.parsed_ty().clone(), value.as_ty.as_ref(), value.skip) }
}





/***** LIBRARY *****/
/// Generates an implementation for the module, i.e., auxillary types.
pub fn generate_mod_impl(error_mod: &Ident, vis: &Visibility, ident: &Ident, fmt: &Fmt, err: &Err) -> TokenStream2 {
    // Generate the module impl
    if fmt.generates() || err.generates() {
        let fmt_def = fmt.def();
        let err_def = err.def();
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
pub fn generate_parsable_impl_logic<A: OuterAttribute>(error_mod: &Ident, outer_attrs: &[A], fields: &[FieldInfo], err: &Err) -> TokenStream2 {
    let error_mod_path = identity_path(error_mod.clone());
    let (whitespace_init, whitespace_pop) = whitespace_pop_impl("rem", outer_attrs);

    // Generate the main implementation based on the field infos
    let mut parse_impl = TokenStream2::new();
    for field in fields {
        // Check whether to generate a default implementation, or a field-based implementation
        let name = &field.name;
        let ty = field.parsed_ty();
        if !field.skip {
            // Generate the parse
            // NOTE: `from_fn` now generates the identity function if no error map is necessary
            let error_fn = err.from_fn(&error_mod_path, &field.name);
            parse_impl.extend(quote! {
                let (#name, rem): (#ty, ::ast_toolkit2::nibble::Slice<::std::primitive::u8>) = ::ast_toolkit2::nibble::error::ResultExt::map_nerr(
                    ::ast_toolkit2::nibble::error::ResultExt::map_fmt(
                        ::ast_toolkit2::nibble::Parsable::<::std::primitive::u8>::parse(rem),
                        |_| <Self as ::ast_toolkit2::nibble::Parsable::<::std::primitive::u8>>::expects()
                    ),
                    #error_fn
                )?;
            });
        } else {
            // This one's easy
            parse_impl.extend(quote! { let #name: #ty = ::std::default::Default::default(); });
        }

        // Don't forget to convert types if necessary
        if field.as_ty.is_some() {
            let og_ty = &field.ty;
            if !field.try_as {
                parse_impl.extend(quote! {
                    let #name: #og_ty = ::std::convert::From::from(#name);
                });
            } else {
                let error_fn = err.try_as_from_fn(&error_mod_path, &field.name);
                parse_impl.extend(quote! {
                    let #name: #og_ty = ::std::result::Result::map_err(
                        ::std::convert::TryFrom::try_from(#name),
                        #error_fn
                    )?;
                });
            }
        }
    }

    quote! {
        // Pop whitespace first
        #[allow(unused)]
        type Match<'a> = ::ast_toolkit2::nibble::__macros::Match<'a>;
        #[allow(unused)]
        type Regex = ::ast_toolkit2::nibble::__macros::Regex;
        #whitespace_init
        #whitespace_pop

        // Then pop normally
        #parse_impl
    }
}



/// Handles the implementation.
pub fn handle_impl(outer_attrs: &[OuterAttr], vis: Visibility, ident: Ident, gens: Generics, fields: Fields) -> Result<TokenStream2, Error> {
    // Do some outer attribute processing
    let error_mod = cook_up_error_mod_name(outer_attrs, &ident);
    let error_mod_path = identity_path(error_mod.clone());

    // First, analyze each field and remember what we need to keep
    let struct_ty = StructTy::from(&fields);
    let fields = fields.into_iter().enumerate().map(FieldInfo::try_from).collect::<Result<Vec<FieldInfo>, Error>>()?;
    if OuterAttr::has_as(outer_attrs).is_none() && OuterAttr::has_try_as(outer_attrs).is_none() && fields.iter().filter(|f| !f.skip).count() == 0 {
        return Err(Error::new(
            ident.span(),
            "Cannot derive `Parsable` on struct without any (unskipped) fields.\n\nGive it fields, or use another parser via `#[parse(as = ...)]` \
             or `#[parse(try_as = ...)]`.",
        ));
    }

    // Decide on the formatter & error impls
    let pident = identity_path(ident.clone());
    let fmt = Fmt::new("then", outer_attrs, &pident, &gens, fields.iter().map(FmtInfo::from));
    let err = Err::new(&pident, &gens, fields.iter().map(ErrInfo::from));

    // Generate the mod implementation & main implementation logic
    let mod_impl = generate_mod_impl(&error_mod, &vis, &ident, &fmt, &err);
    let main_logic = generate_parsable_impl_logic(&error_mod, outer_attrs, &fields, &err);

    // Generate the instantiation at the end of the parse
    let inst_impl: Punctuated<TokenStream2, Token![,]> = fields.iter().map(|field| field.name.to_token_stream()).collect();
    let inst_impl = match struct_ty {
        StructTy::Named => quote! { #ident { #inst_impl } },
        StructTy::Unnamed => quote! { #ident(#inst_impl) },
        StructTy::Unit => quote! { #ident },
    };

    // Prepare generics by inserting the bound
    let mut gens = gens.clone();
    inject_parsable_bound(&mut gens);
    let (impl_gen, ty_gen, where_bounds) = gens.split_for_impl();

    // Done, generate the mod & impl for real
    let fmt_ty = fmt.ty(&error_mod_path);
    let fmt_inst = fmt.inst(&error_mod_path);
    let err_ty = err.ty(&error_mod_path);
    Ok(quote! {
        #mod_impl

        #[automatically_derived]
        impl #impl_gen ::ast_toolkit2::nibble::Parsable<::std::primitive::u8> for #ident #ty_gen #where_bounds {
            type Formatter = #fmt_ty;

            type Error = #err_ty;

            fn expects() -> Self::Formatter { #fmt_inst }

            fn parse(rem: ::ast_toolkit2::nibble::Slice<::std::primitive::u8>) -> ::std::result::Result<(Self, ::ast_toolkit2::nibble::Slice<::std::primitive::u8>), ::ast_toolkit2::nibble::error::NibbleError<Self::Formatter, Self::Error>> {
                #main_logic
                ::std::result::Result::Ok((#inst_impl, rem))
            }
        }
    })
}
