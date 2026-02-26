//  DERIVE TAG.rs
//    by Lut99
//
//  Description:
//!   Implements the derive macro for `Tag`.
//

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned as _;
use syn::{Attribute, Data, DataEnum, DataUnion, DeriveInput, Error, Expr, Meta, Token, Type};

use crate::common::inject_trait_bound;


/***** HELPERS *****/
/// Defines how to parse the contents of a single `#[tag(...)]`-attribute.
struct Attr(Type, Expr);
impl Parse for Attr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ty: Type = input.parse()?;
        input.parse::<Token![,]>()?;
        let expr: Expr = input.parse()?;
        //  Must be all
        if !input.is_empty() {
            return Err(input.error("Unexpected tokens after type and expression"));
        }
        Ok(Self(ty, expr))
    }
}





/***** HELPER FUNCTIONS *****/
/// Defines how to parse the given toplevel attributes of a struct.
fn parse_attrs(attrs: &[Attribute], span: Span) -> Result<(Type, Expr), Error> {
    let mut res: Option<(Span, (Type, Expr))> = None;
    for attr in attrs {
        match &attr.meta {
            // First, find the `#[tag(...)]`-part to select on the tags beloning to us
            Meta::List(l) if l.path.is_ident("tag") => {
                // Parse the inner of the list as our attribute pair
                let Attr(ty, expr) = syn::parse2(l.tokens.clone())?;
                match res {
                    Some((span, _)) => return Err(Error::new(span, "Cannot define `#[tag(...)]` twice")),
                    None => res = Some((l.path.span(), (ty, expr))),
                }
            },

            // Rest is ignored
            _ => continue,
        }
    }
    res.map(|(_, res)| res).ok_or_else(|| Error::new(span, "Missing `#[tag(...)]` macro to define element type and tag string"))
}





/***** LIBRARY *****/
/// Main handler for the macro.
pub fn handle(item: TokenStream2) -> Result<TokenStream2, Error> {
    // We'll need to dive into some attributes
    let DeriveInput { attrs, ident, data, mut generics, .. } = syn::parse2(item)?;
    let (elem, tag): (Type, Expr) = parse_attrs(&attrs, ident.span())?;

    // Now build the impl
    match data {
        Data::Struct(s) => {
            inject_trait_bound(["ast_toolkit2", "loc", "Located"], &mut generics);
            let (impl_gen, ty_gen, where_clauses) = generics.split_for_impl();

            // Find the fields that are loc'd
            // This guarantees to us there is exactly one field*
            let locs: Vec<usize> = crate::derive_located::find_loc_fields(&attrs, &s.fields)?;
            if locs.is_empty() {
                // * Exactly? No!! The user can give us no loc for whatever reason. This means they just discard it.
                // NOTE: We give this a span for better error handling
                let def = quote_spanned! { ident.span() => ::std::default::Default::default() };
                return Ok(quote! {
                    impl #impl_gen ::ast_toolkit2::tree::Tag<#elem> for #ident #ty_gen #where_clauses {
                        type TAG: &'static [#elem] = #tag;

                        #[inline]
                        fn new() -> Self { #def }
                        #[inline]
                        fn with_loc(_: ::ast_toolkit2::loc::Loc) -> Self { #def }
                    }
                });
            }

            // First, figure out how to build a thing of this type
            if s.fields.is_empty() {
                return Err(Error::new(ident.span(), "Expected at least one field to put the tag's `Loc` in"));
            } else if s.fields.len() == 1 {
                // There is precisely one field; we assume that that's the one where we put the
                // `loc`.
                let field = s.fields.into_iter().next().unwrap();
                if let Some(field) = field.ident {
                    Ok(quote! {
                        impl #impl_gen ::ast_toolkit2::tree::Tag<#elem> for #ident #ty_gen #where_clauses {
                            type TAG: &'static [#elem] = #tag;

                            #[inline]
                            fn new() -> Self { Self { #field: ::std::convert::Into::into(::ast_toolkit2::loc::Loc::new()) } }
                            #[inline]
                            fn with_loc(loc: ::ast_toolkit2::loc::Loc) -> Self { Self { #field: ::std::convert::Into::into(loc) } }
                        }
                    })
                } else {
                    Ok(quote! {
                        impl #impl_gen ::ast_toolkit2::tree::Tag<#elem> for #ident #ty_gen #where_clauses {
                            type TAG: &'static [#elem] = #tag;

                            #[inline]
                            fn new() -> Self { Self(::std::convert::Into::into(::ast_toolkit2::loc::Loc::new())) }
                            #[inline]
                            fn with_loc(loc: ::ast_toolkit2::loc::Loc) -> Self { Self(::std::convert::Into::into(loc)) }
                        }
                    })
                }
            } else {
                // More than one field; we'll have to use the normal field resolution rules
                todo!();

                // Now we can generate the impl
                Ok(quote! {
                    impl #impl_gen ::ast_toolkit2::tree::Tag<#elem> for #ident #ty_gen #where_clauses {
                        type TAG: &'static [#elem] = #tag;

                        #[inline]
                        fn new() -> Self { TODO }
                        #[inline]
                        fn with_loc(loc: ::ast_toolkit2::loc::Loc) -> Self { TODO }
                    }
                })
            }
        },

        Data::Union(DataUnion { union_token, .. }) => Err(Error::new(union_token.span, "Can only derive `Tag` on structs")),
        Data::Enum(DataEnum { enum_token, .. }) => Err(Error::new(enum_token.span, "Can only derive `Tag` on structs")),
    }
}
