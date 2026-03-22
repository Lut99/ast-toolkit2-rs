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
use syn::{Attribute, Data, DataEnum, DataUnion, DeriveInput, Error, Expr, LitInt, Meta, Token, Type};

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

            // Find the fields that are loc'd and compute a list of "index jumps"; i.e., the number
            // of elements to skip until one arrives at the next index.
            let mut locs: Vec<usize> = crate::derive_located::find_loc_fields("Tag", &attrs, &s.fields)?;
            for i in 1..locs.len() {
                locs[i] -= locs[i - 1];
            }

            // Then generate an implementation overwriting each of those fields
            let mut fields = s.fields.into_iter().enumerate();
            let locs: Vec<TokenStream2> = locs
                .into_iter()
                .map(|di| {
                    // SAFETY: We expect both all found loc fields indices to be in range, and our
                    // difference algorithm above to work.
                    let (i, field) = fields.nth(di.saturating_sub(1)).unwrap();
                    if let Some(name) = field.ident {
                        quote_spanned! { name.span() => res.#name = ::std::convert::Into::into(loc); }
                    } else {
                        // Avoid quote adding a `usize` suffix to the identifier by explicitly
                        // turning it into a literal integer
                        let i = LitInt::new(&i.to_string(), field.span());
                        quote_spanned! { field.span() => res.#i = ::std::convert::Into::into(loc); }
                    }
                })
                .collect();

            // Then build an impl that generates the default and replaces loc fields if they are
            // there
            let default_impl = quote_spanned! { ident.span() => ::std::default::Default::default() };
            Ok(quote! {
                impl #impl_gen ::ast_toolkit2::tree::Tag<#elem> for #ident #ty_gen #where_clauses {
                    const TAG: &'static [#elem] = #tag;

                    #[inline]
                    fn new() -> Self { #default_impl }

                    #[inline]
                    fn with_loc(loc: ::ast_toolkit2::loc::Loc) -> Self {
                        let mut res: Self = #default_impl;
                        #(#locs)*
                        res
                    }
                }
            })
        },

        Data::Union(DataUnion { union_token, .. }) => Err(Error::new(union_token.span, "Can only derive `Tag` on structs")),
        Data::Enum(DataEnum { enum_token, .. }) => Err(Error::new(enum_token.span, "Can only derive `Tag` on structs")),
    }
}
