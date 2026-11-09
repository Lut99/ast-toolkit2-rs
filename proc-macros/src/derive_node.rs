//  DERIVE NODE.rs
//    by Lut99
//
//  Description:
//!   Implements the derive macro for `Node`.
//

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::ext::IdentExt as _;
use syn::parse::{Parse, ParseStream, Parser as _};
use syn::punctuated::Punctuated;
use syn::{Data, DeriveInput, Error, GenericParam, Generics, Ident, Lifetime, LitStr, Meta, Token, TypeParamBound};

use crate::common::inject_trait_bound;


/***** HELPER FUNCTIONS *****/
fn inject_lifetime(lt: &'static str, gens: &mut Generics) {
    let lifetime = Lifetime::new(lt, Span::call_site());
    for param in &mut gens.params {
        let GenericParam::Type(param) = param else { continue };
        param.bounds.insert(0, TypeParamBound::Lifetime(lifetime.clone()))
    }
}





/***** ITERATORS *****/
trait IterFlattenRes<A, E>
where
    Self: Sized + Iterator<Item = Result<A, E>>,
    A: Iterator,
{
    fn flatten_res(self) -> FlattenRes<Self, A>;
}
impl<T: Iterator<Item = Result<A, E>>, A: Iterator, E> IterFlattenRes<A, E> for T {
    #[inline(always)]
    fn flatten_res(self) -> FlattenRes<Self, A> { FlattenRes(self, None) }
}

struct FlattenRes<I, A>(I, Option<A>);
impl<I: Iterator<Item = Result<A, E>>, A: Iterator, E> Iterator for FlattenRes<I, A> {
    type Item = Result<A::Item, E>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Drain the current iterator
        if let Some(mut iter) = self.1.take() {
            if let Some(item) = iter.next() {
                return Some(Ok(item));
            }
        }

        // Attempt to get the next one and drain it again
        self.1 = Some(match self.0.next()? {
            Ok(iter) => iter,
            // We failed. Return the error and then potentially re-try next run.
            Err(err) => return Some(Err(err)),
        });
        self.next()
    }
}





/***** ATTRIBUTES *****/
/// Defines struct/enum-level attributes for `#[node(...)]`.
pub enum OuterAttr {
    Lit(LitStr),
    NoMut,
    NonTerm,
    Term(LitStr),
}
impl OuterAttr {
    #[inline]
    fn has_lit(attrs: &[Self]) -> Option<&LitStr> {
        attrs.into_iter().find_map(|a| match a {
            Self::Lit(l) => Some(l),
            _ => None,
        })
    }
    #[inline]
    fn has_no_mut(attrs: &[Self]) -> bool { attrs.into_iter().any(|a| matches!(a, Self::NoMut)) }
    #[inline]
    fn has_nonterm(attrs: &[Self]) -> bool { attrs.into_iter().any(|a| matches!(a, Self::NonTerm)) }
    #[inline]
    fn has_term(attrs: &[Self]) -> Option<&LitStr> {
        attrs.into_iter().find_map(|a| match a {
            Self::Term(t) => Some(t),
            _ => None,
        })
    }

    fn check_consistency(attrs: &[Self]) -> Result<(), Error> {
        if let Some(lit) = Self::has_lit(attrs)
            && Self::has_nonterm(attrs)
        {
            return Err(Error::new(lit.span(), "Attributes `lit` and `nonterm` are mutually exclusive"));
        }
        if let Some(lit) = Self::has_lit(attrs)
            && Self::has_term(attrs).is_some()
        {
            return Err(Error::new(lit.span(), "Attributes `lit` and `term` are mutually exclusive"));
        }
        if let Some(term) = Self::has_term(attrs)
            && Self::has_nonterm(attrs)
        {
            return Err(Error::new(term.span(), "Attributes `nonterm` and `term` are mutually exclusive"));
        }
        Ok(())
    }
}
impl Parse for OuterAttr {
    #[inline]
    fn parse(input: ParseStream) -> Result<Self, Error> {
        // The stream must begin with an identifier
        let ident: Ident = Ident::parse_any(input)?;
        if ident == "lit" || ident == "literal" {
            input.parse::<Token![=]>()?;
            Ok(Self::Lit(input.parse()?))
        } else if ident == "nomut" || ident == "no_mut" {
            Ok(Self::NoMut)
        } else if ident == "nonterm" || ident == "nonterminal" {
            Ok(Self::NonTerm)
        } else if ident == "term" || ident == "terminal" {
            input.parse::<Token![=]>()?;
            Ok(Self::Term(input.parse()?))
        } else {
            Err(Error::new(ident.span(), &format!("Unknown `node` outer attribute {:?}", ident.to_string())))
        }
    }
}





/***** LIBRARY *****/
/// Main handler for the macro.
pub fn handle(item: TokenStream2) -> Result<TokenStream2, Error> {
    let DeriveInput { attrs, ident, data, mut generics, .. } = syn::parse2(item)?;
    if let Data::Union(u) = data {
        return Err(Error::new(u.union_token.span, "Can only derive `Node` on structs or enums"));
    }

    // Parse the attributes
    let attrs: Vec<OuterAttr> = attrs
        .into_iter()
        .map(|a| match a.meta {
            Meta::List(l) if l.path.is_ident("node") => {
                let inner: Punctuated<OuterAttr, Token![,]> = Punctuated::parse_terminated.parse2(l.tokens.clone())?;
                Ok(inner.into_iter())
            },

            // Ignore
            Meta::List(_) => Ok(Punctuated::<OuterAttr, Token![,]>::new().into_iter()),
            Meta::NameValue(_) => Ok(Punctuated::<OuterAttr, Token![,]>::new().into_iter()),
            Meta::Path(_) => Ok(Punctuated::<OuterAttr, Token![,]>::new().into_iter()),
        })
        .flatten_res()
        .collect::<Result<Vec<OuterAttr>, Error>>()?;
    OuterAttr::check_consistency(&attrs)?;

    // Generate the node mut impl
    inject_lifetime("'static", &mut generics);
    inject_trait_bound(["ast_toolkit2", "loc", "Located"], &mut generics);
    let (impl_gen, ty_gen, where_clauses) = generics.split_for_impl();
    let node_mut_impl = if !OuterAttr::has_no_mut(&attrs) {
        quote! {
            impl #impl_gen ::ast_toolkit2::tree:NodeMut for #ident #ty_gen #where_clauses {
                #[inline(always)]
                fn as_any_mut(&mut self) -> &mut dyn ::std::any::Any { self as &mut dyn ::std::any::Any }
            }
        }
    } else {
        TokenStream2::new()
    };

    // Generate the base node impl
    // NOTE: Inherited this because we need to implement `Located`.
    let mut res = quote! {
        impl #impl_gen ::ast_toolkit2::tree::Node for #ident #ty_gen #where_clauses {
            #[inline(always)]
            fn as_any(&self) -> &dyn ::std::any::Any { self as &dyn ::std::any::Any }
        }
        #node_mut_impl
    };

    // Then generate the appropriate subtrait
    if OuterAttr::has_nonterm(&attrs) {
        res.extend(quote! {
            impl #impl_gen ::ast_toolkit2::tree::NonTerm for #ident #ty_gen #where_clauses {}
        });
    } else {
        res.extend(quote! {
            impl #impl_gen ::ast_toolkit2::tree::Term for #ident #ty_gen #where_clauses {}
        });
    }

    // Done
    Ok(res)
}
