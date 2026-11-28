//  DERIVE VISIT.rs
//    by Lut99
//
//  Description:
//!   Implements the derive macro for `Visit`.
//

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, quote};
use syn::ext::IdentExt as _;
use syn::parse::{Parse, ParseStream, Parser as _};
use syn::punctuated::Punctuated;
use syn::{Data, DeriveInput, Error, Fields, Ident, Meta, Token};

use crate::common::inject_trait_bound;


/***** ATTRIBUTES *****/
/// Defines variant-level attributes for `#[visit(...)]`.
pub enum VarAttr {
    Skip,
}
impl VarAttr {
    #[inline]
    fn has_skip(attrs: &[Self]) -> bool { attrs.into_iter().any(|a| matches!(a, Self::Skip)) }

    fn check_consistency(_attrs: &[Self]) -> Result<(), Error> { Ok(()) }
}
impl Parse for VarAttr {
    #[inline]
    fn parse(input: ParseStream) -> Result<Self, Error> {
        // The stream must begin with an identifier
        let ident: Ident = Ident::parse_any(input)?;
        if ident == "skip" {
            Ok(Self::Skip)
        } else {
            Err(Error::new(ident.span(), &format!("Unknown `skip` variant attribute {:?}", ident.to_string())))
        }
    }
}



/// Defines field-level attributes for `#[visit(...)]`.
pub enum FieldAttr {
    Skip,
}
impl FieldAttr {
    #[inline]
    fn has_skip(attrs: &[Self]) -> bool { attrs.into_iter().any(|a| matches!(a, Self::Skip)) }

    fn check_consistency(_attrs: &[Self]) -> Result<(), Error> { Ok(()) }
}
impl Parse for FieldAttr {
    #[inline]
    fn parse(input: ParseStream) -> Result<Self, Error> {
        // The stream must begin with an identifier
        let ident: Ident = Ident::parse_any(input)?;
        if ident == "skip" {
            Ok(Self::Skip)
        } else {
            Err(Error::new(ident.span(), &format!("Unknown `skip` field attribute {:?}", ident.to_string())))
        }
    }
}





/***** FUNCTIONS *****/
/// Builds a struct's implementation.
fn impl_struct(trait_fn_name_ident: &Ident, trait_visitor_ident: &Ident, fields: Fields) -> Result<TokenStream2, Error> {
    // Compute the visit impl for all fields
    let visit_impl = fields
        .into_iter()
        .enumerate()
        .filter_map(|(i, f)| {
            // Parse the field's arguments
            let mut attrs: Vec<FieldAttr> = Vec::new();
            for attr in f.attrs {
                let Meta::List(l) = attr.meta else { continue };
                if !l.path.is_ident("visit") {
                    continue;
                }
                let inner: Punctuated<FieldAttr, Token![,]> = match Punctuated::parse_terminated.parse2(l.tokens.clone()) {
                    Ok(inner) => inner,
                    Err(err) => return Some(Err(err)),
                };
                attrs.extend(inner.into_iter());
            }
            if let Err(err) = FieldAttr::check_consistency(&attrs) {
                return Some(Err(err));
            }

            // Skip the field if requested
            if FieldAttr::has_skip(&attrs) {
                return None;
            }

            // Compute its name
            let name = f.ident.unwrap_or_else(|| Ident::new(&format!("f{i}"), Span::call_site()));

            // Build the impl
            Some(Ok(quote! { ::ast_toolkit2::visit::Composable::compose_with(&mut res, ::ast_toolkit2::visit::#trait_visitor_ident::#trait_fn_name_ident(visitor, #name)); }))
        })
        .collect::<Result<TokenStream2, Error>>()?;

    // Build the impl
    Ok(quote! {
        let mut res: V::Result = ::ast_toolkit2::visit::Initializable::init();
        #visit_impl
        res
    })
}





/***** LIBRARY *****/
/// Main handler for the macro.
pub fn handle(impl_mut: bool, item: TokenStream2) -> Result<TokenStream2, Error> {
    let trait_name: &str = if impl_mut { "VisitMut" } else { "Visit" };
    let trait_ident = Ident::new(trait_name, Span::call_site());
    let trait_fn_name_ident = if impl_mut { Ident::new("visit_mut", Span::call_site()) } else { Ident::new("visit", Span::call_site()) };
    let trait_visitor_ident = if impl_mut { Ident::new("VisitorMut", Span::call_site()) } else { Ident::new("Visitor", Span::call_site()) };
    let trait_mut = if impl_mut { Ident::new("mut", Span::call_site()).into_token_stream() } else { TokenStream2::new() };

    let DeriveInput { ident, data, mut generics, .. } = syn::parse2(item)?;
    if let Data::Union(u) = data {
        return Err(Error::new(u.union_token.span, &format!("Can only derive `{trait_name}` on structs or enums")));
    }

    // Do the generics already
    inject_trait_bound(["ast_toolkit2", "visit", trait_name], &mut generics);
    let (impl_gen, ty_gen, where_bounds) = generics.split_for_impl();

    // Match on the type of data
    match data {
        Data::Struct(s) => {
            // Build the impl with that
            let fields = match &s.fields {
                Fields::Named(n) => {
                    let fields = n.named.iter().map(|f| f.ident.as_ref().unwrap()).collect::<Punctuated<&Ident, Token![,]>>();
                    quote! {
                        #[allow(unused_variables)]
                        let Self { #fields } = self;
                    }
                },
                Fields::Unnamed(u) => {
                    let fields =
                        (0..u.unnamed.len()).map(|i| Ident::new(&format!("f{i}"), Span::call_site())).collect::<Punctuated<Ident, Token![,]>>();
                    quote! {
                        #[allow(unused_variables)]
                        let Self( #fields ) = self;
                    }
                },
                Fields::Unit => TokenStream2::new(),
            };
            let visit_impl = impl_struct(&trait_fn_name_ident, &trait_visitor_ident, s.fields)?;
            Ok(quote! {
                impl #impl_gen ::ast_toolkit2::visit::#trait_ident for #ident #ty_gen #where_bounds {
                    fn #trait_fn_name_ident<V: ::ast_toolkit2::visit::#trait_visitor_ident>(&#trait_mut self, visitor: &mut V) -> V::Result {
                        #fields
                        #visit_impl
                    }
                }
            })
        },

        Data::Enum(e) => {
            // Go thru each variant to build match arms
            let mut skipped_any: bool = false;
            let var_impls = e
                .variants
                .into_iter()
                .filter_map(|v| {
                    // Parse the variants's arguments
                    let mut attrs: Vec<VarAttr> = Vec::new();
                    for attr in v.attrs {
                        let Meta::List(l) = attr.meta else { continue };
                        if !l.path.is_ident("visit") {
                            continue;
                        }
                        let inner: Punctuated<VarAttr, Token![,]> = match Punctuated::parse_terminated.parse2(l.tokens.clone()) {
                            Ok(inner) => inner,
                            Err(err) => return Some(Err(err)),
                        };
                        attrs.extend(inner.into_iter());
                    }
                    if let Err(err) = VarAttr::check_consistency(&attrs) {
                        return Some(Err(err));
                    }

                    // Check if we should skip this variant
                    if VarAttr::has_skip(&attrs) {
                        skipped_any = true;
                        return None;
                    }

                    // If we don't, generate the match arm
                    let var = v.ident;
                    let fields = match &v.fields {
                        Fields::Named(n) => {
                            let fields = n.named.iter().map(|f| f.ident.as_ref().unwrap()).collect::<Punctuated<&Ident, Token![,]>>();
                            quote! { { #fields } }
                        },
                        Fields::Unnamed(u) => {
                            let fields = (0..u.unnamed.len())
                                .map(|i| Ident::new(&format!("f{i}"), Span::call_site()))
                                .collect::<Punctuated<Ident, Token![,]>>();
                            quote! { ( #fields ) }
                        },
                        Fields::Unit => TokenStream2::new(),
                    };
                    let var_impl = match impl_struct(&trait_fn_name_ident, &trait_visitor_ident, v.fields) {
                        Ok(var_impl) => var_impl,
                        Err(err) => return Some(Err(err)),
                    };
                    Some(Ok(quote! {
                        #[allow(unused_variables)]
                        Self::#var #fields => { #var_impl },
                    }))
                })
                .collect::<Result<TokenStream2, Error>>()?;

            // Build the impl
            let rem = if skipped_any {
                quote! { _ => ::ast_toolkit2::visit::Initializable::init(), }
            } else {
                TokenStream2::new()
            };
            Ok(quote! {
                impl #impl_gen ::ast_toolkit2::visit::#trait_ident for #ident #ty_gen #where_bounds {
                    fn #trait_fn_name_ident<V: ::ast_toolkit2::visit::#trait_visitor_ident>(&#trait_mut self, visitor: &mut V) -> V::Result {
                        match self {
                            #var_impls
                            #rem
                        }
                    }
                }
            })
        },

        // Caught above
        Data::Union(_) => unreachable!(),
    }
}
