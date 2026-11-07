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
use syn::{Data, DeriveInput, Error, Fields, GenericParam, Generics, Ident, Lifetime, LifetimeParam, LitInt, LitStr, Meta, Token, Variant};

use crate::common::inject_trait_bound;


/***** HELPER FUNCTIONS *****/
fn find_unused_lifetime(gens: &Generics) -> Lifetime {
    'attempts: for i in 0.. {
        let attempt = format!("'a{i}");
        for lt in gens.lifetimes() {
            if lt.lifetime.ident == &attempt {
                continue 'attempts;
            }
        }

        // We found it
        return Lifetime::new(&attempt, Span::call_site());
    }
    // SAFETY: This loop never terminates, unless it returns
    unsafe { std::hint::unreachable_unchecked() }
}



fn inject_lifetime(lt: &Lifetime, gens: &mut Generics) {
    // Look for the last lifetime
    let mut i: usize = 0;
    while i < gens.params.len() {
        if !matches!(gens.params[i], GenericParam::Lifetime(_)) {
            break;
        }
        i += 1;
    }

    // Insert the new one just after that
    gens.params
        .insert(i, GenericParam::Lifetime(LifetimeParam { attrs: Vec::new(), lifetime: lt.clone(), colon_token: None, bounds: Punctuated::new() }));
}



fn generate_struct_iter_impl(gens: &Generics, ident: &Ident, fields: &Fields) -> TokenStream2 {
    // Prepare the state enum
    let fieldnames: Vec<Ident> =
        fields.into_iter().enumerate().map(|(i, f)| f.ident.clone().unwrap_or_else(|| Ident::new(&format!("field{i}"), Span::call_site()))).collect();
    let len: LitInt = LitInt::new(&fieldnames.len().to_string(), Span::call_site());
    let variants: TokenStream2 = fieldnames.iter().map(|f| quote! { #[allow(non_camel_case_types)] #f, }).collect();

    // Prepare the struct unwrappor
    let unwrap = match fields {
        Fields::Named(_) => {
            let fieldnames = fieldnames.iter().map(|i| quote! {#i,}).collect::<TokenStream2>();
            quote! { let #ident { #fieldnames } = self.0; }
        },
        Fields::Unnamed(_) => {
            let fieldnames = fieldnames.iter().map(|i| quote! {#i,}).collect::<TokenStream2>();
            quote! { let #ident(#fieldnames) = self.0; }
        },
        Fields::Unit => TokenStream2::new(),
    };

    // Prepare the iterator steps
    let first =
        fieldnames.first().map(|f| quote! { ::std::option::Option::Some(State::#f) }).unwrap_or_else(|| quote! { ::std::option::Option::None });
    let mut steps: TokenStream2 = fieldnames
        .array_windows()
        .map(|[this, next]| {
            quote! {
                State::#this => {
                    self.1 = ::std::option::Option::Some(State::#next);
                    ::std::option::Option::Some(#this)
                },
            }
        })
        .collect();
    if let Some(last) = fieldnames.last() {
        steps.extend(quote! {
            State::#last => {
                self.1 = ::std::option::Option::None;
                ::std::option::Option::Some(#last)
            },
        });
    }

    // Prepare the generics
    let lt: Lifetime = find_unused_lifetime(&gens);
    let mut iter_gens = gens.clone();
    inject_lifetime(&lt, &mut iter_gens);
    let (_, ty_gen, _) = gens.split_for_impl();
    let (iter_impl_gen, iter_ty_gen, iter_where_clauses) = iter_gens.split_for_impl();
    let mut iterator_gens = iter_gens.clone();
    inject_trait_bound(["ast_toolkit2", "tree", "Node"], &mut iterator_gens);
    let (iterator_impl_gen, iterator_ty_gen, iterator_where_clauses) = iterator_gens.split_for_impl();

    // Build it!
    quote! {
        enum State {
            #variants
        }

        struct Iter #iter_gens(&#lt #ident #ty_gen, ::std::option::Option<State>);
        impl #iter_impl_gen Iter #iter_ty_gen #iter_where_clauses {
            #[inline(always)]
            const fn new(obj: &#lt #ident #ty_gen) -> Self { Self(obj, #first) }
        }
        impl #iterator_impl_gen ::std::iter::Iterator for Iter #iterator_ty_gen #iterator_where_clauses {
            type Item = &#lt dyn ::ast_toolkit2::tree::Node;

            #[inline]
            fn next(&mut self) -> ::std::option::Option<Self::Item> {
                #unwrap
                match ::std::option::Option::as_ref(&self.1)? {
                    #steps
                }
            }

            #[inline]
            fn size_hint(&self) -> (::std::primitive::usize, ::std::option::Option<::std::primitive::usize>) { (#len, ::std::option::Option::Some(#len)) }
        }
    }
}

fn generate_enum_iter_impl(gens: &Generics, ident: &Ident, vars: &Punctuated<Variant, Token![,]>) -> TokenStream2 { TokenStream2::new() }





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

    // Generate the iterator impl
    let iter_impl: TokenStream2 = match &data {
        Data::Struct(s) => generate_struct_iter_impl(&generics, &ident, &s.fields),
        Data::Enum(e) => generate_enum_iter_impl(&generics, &ident, &e.variants),
        Data::Union(_) => unreachable!(),
    };

    // Generate the node impl
    inject_trait_bound(["ast_toolkit2", "tree", "Node"], &mut generics);
    let (impl_gen, ty_gen, where_clauses) = generics.split_for_impl();
    let tfish = ty_gen.as_turbofish();
    let mut res = quote! {
        impl #impl_gen ::ast_toolkit2::tree::Node for #ident #ty_gen #where_clauses {
            fn children(&self) -> ::std::boxed::Box<dyn '_ + ::std::iter::Iterator<Item = &dyn ::ast_toolkit2::tree::Node>> {
                #iter_impl

                ::std::boxed::Box::new(Iter #tfish ::new(self))
            }
        }
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
