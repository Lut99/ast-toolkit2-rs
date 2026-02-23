//  DERIVE LOCATED.rs
//    by Lut99
//
//  Description:
//!   Implements the derive macro for `Located`.
//

use proc_macro2::{Literal as Literal2, Span, TokenStream as TokenStream2, TokenTree as TokenTree2};
use quote::quote;
use syn::parse::{Error, Parser as _};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned as _;
use syn::{
    Attribute, Data, DataEnum, DataStruct, DataUnion, DeriveInput, Fields, Generics, Ident, Meta, Path, PathArguments, PathSegment, Token,
    TraitBound, TraitBoundModifier, TypeParamBound, Variant,
};


/***** HELPER FUNCTIONS *****/
/// Scans a list of attributes for the `#[loc]`-attribute.
///
/// # Arguments
/// - `attrs`: Some list of attributes.
///
/// # Returns
/// Whether the attribute was found or not.
fn has_loc_attr<'a>(attrs: impl IntoIterator<Item = &'a Attribute>) -> Result<bool, Error> {
    for attr in attrs {
        match &attr.meta {
            // We only filter on our own attributes
            Meta::Path(p) if p.is_ident("loc") => return Ok(true),
            Meta::List(l) if l.path.is_ident("loc") => {
                return Err(Error::new(l.path.span(), "Unsure what to do with attribute; at a field, use `#[loc]` or none."));
            },
            Meta::NameValue(nv) if nv.path.is_ident("loc") => {
                return Err(Error::new(nv.span(), "Unsure what to do with attribute; at a field, use `#[loc]` or none."));
            },

            // The rest we ignore, part of other crates (or macros)
            _ => continue,
        }
    }
    Ok(false)
}

/// Given a set of [`Fields`], attempts to find the `loc`-field.
///
/// It scans for either:
/// - A toplevel `#[loc(all)]` to include all fields;
/// - Any fields marked with `#[loc]`;
/// - If none of those are given but it's in struct syntax, a single field called `loc`; or
/// - If none of those are given but it's in tuple syntax, the only field.
///
/// # Arguments
/// - `attrs`: A list of toplevel attributes to scan through.
/// - `fields`: A [`Fields`] that represents the fields we scan through.
///
/// # Returns
/// The list of found fields.
fn find_loc_fields<'a>(attrs: impl IntoIterator<Item = &'a Attribute>, fields: &Fields) -> Result<Vec<usize>, Error> {
    // First, check if we have a toplevel attribute
    let mut do_all: Option<Span> = None;
    let mut do_new: Option<Span> = None;
    for attr in attrs {
        match &attr.meta {
            // We only filter on our own attributes
            Meta::List(l) if l.path.is_ident("loc") => {
                let inner: Punctuated<Meta, Token![,]> = Punctuated::parse_terminated.parse2(l.tokens.clone())?;
                for meta in inner {
                    match meta {
                        // Now we control all attributes, so only do sensible ones
                        Meta::Path(p) if p.is_ident("all") => do_all = Some(p.span()),
                        Meta::Path(p) if p.is_ident("new") => do_new = Some(p.span()),
                        meta => {
                            return Err(Error::new(
                                meta.span(),
                                &format!(
                                    "Unknown attribute{}",
                                    if let Some(ident) = meta.path().get_ident() { format!(" {ident}") } else { String::new() }
                                ),
                            ));
                        },
                    }
                }
            },
            Meta::Path(p) if p.is_ident("loc") => {
                return Err(Error::new(p.span(), "Unsure what to do with attribute; at the toplevel, use `#[loc(all)]` or none."));
            },
            Meta::NameValue(nv) if nv.path.is_ident("loc") => {
                return Err(Error::new(nv.span(), "Unsure what to do with attribute; at the toplevel, use `#[loc(all)]` or none."));
            },

            // The rest we ignore, part of other crates (or macros)
            _ => continue,
        }
    }

    // Decide if we have global info
    if do_all.is_some() && do_new.is_none() {
        return Ok((0..fields.len()).collect());
    } else if do_all.is_none() && do_new.is_some() {
        return Ok(Vec::new());
    } else if let (Some(_), Some(do_new)) = (do_all, do_new) {
        return Err(Error::new(do_new, "Cannot declare both `#[loc(all)]` and `#[loc(new)]` on the same type or variant"));
    }

    // If we didn't find it, then try to find the fields.
    match fields {
        Fields::Named(n) => {
            // Attempt to find any fields with
            let mut only_candidate: Option<usize> = None;
            let mut too_many_candidates: Vec<usize> = Vec::new();
            let mut res: Vec<usize> = Vec::new();
            for (i, field) in n.named.iter().enumerate() {
                if has_loc_attr(&field.attrs)? {
                    res.push(i);
                } else if field.ident.as_ref().map(|i| i == "loc").unwrap_or(false) {
                    if only_candidate.is_none() {
                        only_candidate = Some(i);
                    } else {
                        too_many_candidates.push(i);
                    }
                }
            }

            // We now...
            if !res.is_empty() {
                // ...have a list of marked attributes
                Ok(res)
            } else if let Some(only_candidate) = only_candidate
                && too_many_candidates.is_empty()
            {
                // ...precisely one field called `loc`
                Ok(vec![only_candidate])
            } else if !too_many_candidates.is_empty() {
                Err(Error::new(fields.span(), "Failed to find any `#[loc]` field but found more than one `loc` fields; cannot derive `Located`"))
            } else {
                // ...or absolutely nothing
                Err(Error::new(fields.span(), "Failed to find any `#[loc]` field or a field named `loc`; cannot derive `Located`"))
            }
        },

        Fields::Unnamed(u) => {
            // Use the only field if it's the only field
            if u.unnamed.len() == 1 {
                return Ok(vec![0]);
            }

            // Else, collect a list of explicitly marked ones
            let mut res: Vec<usize> = Vec::new();
            for (i, field) in u.unnamed.iter().enumerate() {
                if has_loc_attr(&field.attrs)? {
                    res.push(i);
                }
            }

            // We either have a list of marked fields; or nothing
            if !res.is_empty() { Ok(res) } else { Err(Error::new(fields.span(), "Failed to find any `#[loc]` field; cannot derive `Located`")) }
        },

        Fields::Unit => Err(Error::new(fields.span(), "No fields present; cannot derive `Located`")),
    }
}

/// Given a set of [`Generics`], assigns each of them the `Located`-trait.
///
/// # Arguments
/// - `gens`: The [`Generics`] to inject the additional trait bounds in.
fn inject_located(gens: &mut Generics) {
    for param in gens.type_params_mut() {
        param.bounds.push(TypeParamBound::Trait(TraitBound {
            paren_token: None,
            modifier: TraitBoundModifier::None,
            lifetimes: None,
            path: Path {
                leading_colon: Some(Default::default()),
                segments:      {
                    let mut segs = Punctuated::new();
                    segs.push(PathSegment { ident: Ident::new("ast_toolkit2", Span::call_site()), arguments: PathArguments::None });
                    segs.push(PathSegment { ident: Ident::new("loc", Span::call_site()), arguments: PathArguments::None });
                    segs.push(PathSegment { ident: Ident::new("Located", Span::call_site()), arguments: PathArguments::None });
                    segs
                },
            },
        }))
    }
}





/***** LIBRARY *****/
/// Handler for structs.
fn handle_struct(attrs: Vec<Attribute>, ident: Ident, mut generics: Generics, DataStruct { fields, .. }: DataStruct) -> Result<TokenStream2, Error> {
    // Search the fields for our darling fields
    let mut loc_fields = find_loc_fields(&attrs, &fields)?;
    if loc_fields.is_empty() {
        // Special case: the user gave us `#[loc(new)]`
        let (impl_gen, ty_gen, where_bounds) = generics.split_for_impl();
        return Ok(quote! { impl #impl_gen ::ast_toolkit2::loc::Located for #ident #ty_gen #where_bounds {
            #[inline(always)]
            fn loc(&self) -> ::ast_toolkit2::loc::Loc { ::ast_toolkit2::loc::Loc::new() }
        } });
    }

    // Injects the generics
    inject_located(&mut generics);

    // Use that to build an impl
    if loc_fields.len() == 1 {
        let field = loc_fields.pop().unwrap();
        let name: TokenTree2 = match &fields.iter().nth(field).unwrap().ident {
            Some(name) => TokenTree2::Ident(name.clone()),
            None => TokenTree2::Literal(Literal2::usize_unsuffixed(field)),
        };
        let (impl_gen, ty_gen, where_bounds) = generics.split_for_impl();
        Ok(quote! { impl #impl_gen ::ast_toolkit2::loc::Located for #ident #ty_gen #where_bounds {
            #[inline]
            fn loc(&self) -> ::ast_toolkit2::loc::Loc {
                ::ast_toolkit2::loc::Located::loc(&self.#name)
            }
        } })
    } else {
        // Collect how to refer to each field
        let mut names: Vec<TokenTree2> = Vec::with_capacity(loc_fields.len());
        loc_fields.sort(); // Just to be safe
        let mut iter = fields.iter();
        let mut last_i: usize = 0;
        for i in loc_fields {
            // Find the pointer-to field
            while last_i < i {
                iter.next();
                last_i += 1;
            }
            let field = iter.next().unwrap();
            last_i += 1;

            // Build the resulting identifier
            names.push(match &field.ident {
                Some(name) => TokenTree2::Ident(name.clone()),
                None => TokenTree2::Literal(Literal2::usize_unsuffixed(i)),
            });
        }

        // Now build the full impl
        let (impl_gen, ty_gen, where_bounds) = generics.split_for_impl();
        let first: &TokenTree2 = names.first().unwrap();
        let rest: &[TokenTree2] = &names[1..];
        Ok(quote! { impl #impl_gen ::ast_toolkit2::loc::Located for #ident #ty_gen #where_bounds {
            #[inline]
            fn loc(&self) -> ::ast_toolkit2::loc::Loc {
                let mut res: ::ast_toolkit2::loc::Loc = ::ast_toolkit2::loc::Located::loc(&self.#first);
                #(res.extend(::ast_toolkit2::loc::Located::loc(&self.#rest));)*
                res
            }
        } })
    }
}



/// Handler for enums.
fn handle_enum(attrs: Vec<Attribute>, ident: Ident, mut generics: Generics, data: DataEnum) -> Result<TokenStream2, Error> {
    // For every variant...
    let mut variants: Vec<(Ident, bool, usize, Vec<(usize, Ident)>)> = Vec::with_capacity(data.variants.len());
    for Variant { attrs: vattrs, ident, fields, .. } in data.variants {
        // Search the fields for our darling fields
        let loc_fields = find_loc_fields(attrs.iter().chain(vattrs.iter()), &fields)?;
        if loc_fields.is_empty() {
            // Special case: the user gave us `#[loc(new)]` on this type or variant
            variants.push((ident, matches!(fields, Fields::Named(_)), fields.len(), Vec::new()));
            continue;
        }

        // Store the fields we have selected
        let is_named: bool = matches!(fields, Fields::Named(_));
        let total_fields: usize = fields.len();
        let mut res = Vec::with_capacity(loc_fields.len());
        let mut iter = fields.into_iter();
        let mut last_i: usize = 0;
        for i in loc_fields {
            // Find the pointer-to field
            while last_i < i {
                iter.next();
                last_i += 1;
            }
            let field = iter.next().unwrap();
            last_i += 1;

            // Build the resulting identifier
            res.push((i, match field.ident {
                Some(name) => name,
                None => Ident::new(&format!("field{i}"), field.span()),
            }));
        }
        variants.push((ident, is_named, total_fields, res));
    }

    // Early-escape: if there are no variants, we don't generate the normal impl
    if variants.is_empty() {
        let (impl_gen, ty_gen, where_bounds) = generics.split_for_impl();
        return Ok(quote! { impl #impl_gen ::ast_toolkit2::loc::Located for #ident #ty_gen #where_bounds {
            /// NOTE: Unreachable because there are no variants
            #[inline]
            fn loc(&self) -> ::ast_toolkit2::loc::Loc { ::std::unreachable!() }
        } });
    }
    // Check if all fields are empty
    if variants.iter().all(|(_, _, _, res)| res.is_empty()) {
        // Special case: the user gave us `#[loc(new)]` on _all_ variants
        let (impl_gen, ty_gen, where_bounds) = generics.split_for_impl();
        return Ok(quote! { impl #impl_gen ::ast_toolkit2::loc::Located for #ident #ty_gen #where_bounds {
            #[inline(always)]
            fn loc(&self) -> ::ast_toolkit2::loc::Loc { ::ast_toolkit2::loc::Loc::new() }
        } });
    }
    inject_located(&mut generics);

    // With that done, build the impl for each variant
    let mut inner: Vec<TokenStream2> = Vec::with_capacity(variants.len());
    for (variant, is_named, total_fields, fields) in variants {
        // Special case: the user gave `#[loc(new)]` _only_ for this variant
        if fields.is_empty() {
            inner.push(quote! { Self::#variant { .. } => ::ast_toolkit2::loc::Loc::new(), });
            continue;
        }

        // Else, we do the complex case
        if is_named {
            let nfields: Vec<&Ident> = fields.iter().map(|(_, n)| n).collect();
            let name: &Ident = nfields.first().unwrap();
            let rest: &[&Ident] = &nfields[1..];
            inner.push(quote! { Self::#variant{ #(#nfields,)* .. } => { let mut res = ::ast_toolkit2::loc::Located::loc(#name); #(res.extend(::ast_toolkit2::loc::Located::loc(#rest));)* res }, });
        } else {
            let ufields: Vec<&Ident> = fields.iter().map(|(_, n)| n).collect();
            let mut all_ufields = Vec::with_capacity(total_fields);
            let mut last_i: usize = 0;
            for (i, n) in &fields {
                while last_i < *i {
                    all_ufields.push(Ident::new("_", Span::call_site()));
                    last_i += 1;
                }
                all_ufields.push(n.clone());
                last_i += 1;
            }
            while last_i < total_fields {
                all_ufields.push(Ident::new("_", Span::call_site()));
                last_i += 1;
            }
            let name: &Ident = ufields.first().unwrap();
            let rest: &[&Ident] = &ufields[1..];
            inner.push(quote! { Self::#variant(#(#all_ufields),*) => { let mut res = ::ast_toolkit2::loc::Located::loc(#name); #(res.extend(::ast_toolkit2::loc::Located::loc(#rest));)* res } });
        }
    }

    // Now build the full impl
    let (impl_gen, ty_gen, where_bounds) = generics.split_for_impl();
    Ok(quote! { impl #impl_gen ::ast_toolkit2::loc::Located for #ident #ty_gen #where_bounds {
        #[inline]
        fn loc(&self) -> ::ast_toolkit2::loc::Loc {
            match self {
                #(#inner)*
            }
        }
    } })
}



/// Main handler for the macro.
pub fn handle(item: TokenStream2) -> Result<TokenStream2, Error> {
    let DeriveInput { attrs, ident, generics, data, .. } = syn::parse2(item)?;
    match data {
        Data::Struct(data_struct) => handle_struct(attrs, ident, generics, data_struct),
        Data::Enum(data_enum) => handle_enum(attrs, ident, generics, data_enum),
        Data::Union(DataUnion { union_token, .. }) => Err(Error::new(union_token.span, "Can only derive `Located` on structs or enums")),
    }
}
