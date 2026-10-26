//  COMMON.rs
//    by Lut99
//
//  Description:
//!   Defines common algorithms and utilities used across derive macros.
//

use proc_macro2::{Punct, Spacing, Span, TokenStream as TokenStream2, TokenTree as TokenTree2};
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::{GenericParam, Generics, Ident, Path, PathArguments, PathSegment, TraitBound, TraitBoundModifier, TypeParamBound};


/***** HELPER *****/
pub struct ImplGen<'e, 'g>(&'e [&'e str], &'g Generics);
impl<'e, 'g> ToTokens for ImplGen<'e, 'g> {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.extend([TokenTree2::Punct(Punct::new('<', Spacing::Alone))]);

        // Lifetimes first...
        for param in &self.1.params {
            let GenericParam::Lifetime(lt) = param else { continue };
            lt.lifetime.to_tokens(tokens);
            tokens.extend([TokenTree2::Punct(Punct::new(',', Spacing::Alone))]);
        }

        // Then constants
        for param in &self.1.params {
            let GenericParam::Const(c) = param else { continue };
            c.const_token.to_tokens(tokens);
            c.ident.to_tokens(tokens);
            c.colon_token.to_tokens(tokens);
            c.ty.to_tokens(tokens);
            tokens.extend([TokenTree2::Punct(Punct::new(',', Spacing::Alone))]);
        }

        // Finally type generics
        for param in &self.1.params {
            let GenericParam::Type(ty) = param else { continue };
            ty.ident.to_tokens(tokens);
            tokens.extend([TokenTree2::Punct(Punct::new(',', Spacing::Alone))]);
        }
        // With extra!
        for extra in self.0 {
            tokens.extend([TokenTree2::Ident(Ident::new(extra, Span::mixed_site()))]);
            tokens.extend([TokenTree2::Punct(Punct::new(',', Spacing::Alone))]);
        }

        tokens.extend([TokenTree2::Punct(Punct::new('>', Spacing::Alone))]);
    }
}

pub struct TyGen<'g>(&'g Generics);
impl<'g> ToTokens for TyGen<'g> {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.extend([TokenTree2::Punct(Punct::new('<', Spacing::Alone))]);

        // Lifetimes first...
        for param in &self.0.params {
            let GenericParam::Lifetime(lt) = param else { continue };
            lt.lifetime.to_tokens(tokens);
            tokens.extend([TokenTree2::Punct(Punct::new(',', Spacing::Alone))]);
        }

        // Then constants
        for param in &self.0.params {
            let GenericParam::Const(c) = param else { continue };
            c.ident.to_tokens(tokens);
            tokens.extend([TokenTree2::Punct(Punct::new(',', Spacing::Alone))]);
        }

        // Finally type generics
        for param in &self.0.params {
            let GenericParam::Type(ty) = param else { continue };
            ty.ident.to_tokens(tokens);
            tokens.extend([TokenTree2::Punct(Punct::new(',', Spacing::Alone))]);
        }

        tokens.extend([TokenTree2::Punct(Punct::new('>', Spacing::Alone))]);
    }
}

pub struct WhereGen<'g>(&'g Generics);
impl<'g> ToTokens for WhereGen<'g> {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let mut any: bool = false;

        // Lifetimes first...
        for param in &self.0.params {
            let GenericParam::Lifetime(lt) = param else { continue };
            let Some(colon_token) = &lt.colon_token else {
                continue;
            };

            if !any {
                tokens.extend([TokenTree2::Ident(Ident::new("Where", Span::mixed_site()))]);
                any = true;
            }
            lt.lifetime.to_tokens(tokens);
            colon_token.to_tokens(tokens);
            let mut first: bool = true;
            for bound in &lt.bounds {
                if first {
                    tokens.extend([TokenTree2::Punct(Punct::new('+', Spacing::Alone))]);
                    first = false;
                }
                bound.to_tokens(tokens);
            }
            tokens.extend([TokenTree2::Punct(Punct::new(',', Spacing::Alone))]);
        }

        // Finally type generics
        for param in &self.0.params {
            let GenericParam::Type(ty) = param else { continue };
            let Some(colon_token) = &ty.colon_token else { continue };

            ty.ident.to_tokens(tokens);
            colon_token.to_tokens(tokens);
            let mut first: bool = true;
            for bound in &ty.bounds {
                if first {
                    tokens.extend([TokenTree2::Punct(Punct::new('+', Spacing::Alone))]);
                    first = false;
                }
                bound.to_tokens(tokens);
            }
            tokens.extend([TokenTree2::Punct(Punct::new(',', Spacing::Alone))]);
        }
    }
}





/***** LIBRARY *****/
/// Given a set of [`Generics`], assigns each of them the `Located`-trait.
///
/// # Arguments
/// - `path`: An iterable of strings that defines the path to add that refers to the
///   to-be-implemented trait.
/// - `gens`: The [`Generics`] to inject the additional trait bounds in.
pub fn inject_trait_bound<const LEN: usize>(path: [&'static str; LEN], gens: &mut Generics) {
    for param in gens.type_params_mut() {
        param.bounds.push(TypeParamBound::Trait(TraitBound {
            paren_token: None,
            modifier: TraitBoundModifier::None,
            lifetimes: None,
            path: Path {
                leading_colon: Some(Default::default()),
                segments:      {
                    let mut segs = Punctuated::new();
                    for path in path {
                        segs.push(PathSegment { ident: Ident::new(path, Span::call_site()), arguments: PathArguments::None });
                    }
                    segs
                },
            },
        }))
    }
}



/// Given a set of [`Generics`], splits it into something that generates `impl` generics, type
/// generics and where-clauses.
///
/// In contrast to the function provided by [`syn`], this will inject an additional set of generics
/// ONLY added to the impl-generics and (potentially) where-clause.
///
/// # Arguments
/// - `extra`: Any extra generics to add.
/// - `gens`: The [`Generics`] to inject the additional generic parameters in.
///
/// # Returns
/// A triplet of [`ImplGen`], [`TyGen`] and [`WhereGen`] that each render the three parts of the
/// generics.
pub const fn split_for_impl_with_extra<'e, 'g>(extra: &'e [&'e str], gens: &'g Generics) -> (ImplGen<'e, 'g>, TyGen<'g>, WhereGen<'g>) {
    (ImplGen(extra, gens), TyGen(gens), WhereGen(gens))
}
