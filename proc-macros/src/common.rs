//  COMMON.rs
//    by Lut99
//
//  Description:
//!   Defines common algorithms and utilities used across derive macros.
//

use proc_macro2::Span;
use syn::punctuated::Punctuated;
use syn::{Generics, Ident, Path, PathArguments, PathSegment, TraitBound, TraitBoundModifier, TypeParamBound};


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
