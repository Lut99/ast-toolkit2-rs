//  BRANCH.rs
//    by Lut99
//
//  Description:
//!   Implements the logic for enums, i.e., branching parsers.
//

use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::punctuated::Punctuated;
use syn::{DataEnum, Error, Generics, Ident, LitStr, Meta, Path, PathArguments, PathSegment, Token, Type, TypeGenerics, Visibility};

use super::seq::cook_up_error_mod_name;
use super::{OuterAttr, parse_parse_attrs, whitespace_pop_impl};


/***** HELPERS *****/
struct FormatterImplDef<'i, 'g, 's, 't>(&'i Ident, &'g Generics, &'s FormatterImpl<'t>);
impl<'i, 'g, 's, 't> ToTokens for FormatterImplDef<'i, 'g, 's, 't> {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        // Only do something if we're generated
        let FormatterImpl::Generated(fmt) = &self.2 else { return };

        // Compute each fieldname
        let fmt_def: TokenStream2 = fmt
            .into_iter()
            .map(|(name, ty)| quote! { pub #name: <#ty as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::Formatter, })
            .collect();
        let mut iter = fmt.into_iter();
        let fmt_impl = match iter.next() {
            Some((fname, _)) => {
                let mut res = quote! { ::std::fmt::Display::fmt(#fname, f)?; };
                while let Some((name, _)) = iter.next() {
                    res.extend(quote! {
                        ::std::write!(f, ", or ")?;
                        ::std::fmt::Display::fmt(#name, f)?;
                    });
                }
                res
            },
            None => quote! { ::std::write!(f, "<anything>") },
        };
        let fmt_inst: Punctuated<TokenStream2, Token![,]> = fmt.into_iter().map(|(name, _)| name.to_token_stream()).collect();

        // Generate the impl
        let ident = self.0;
        let gens = self.1;
        let (impl_gen, ty_gen, where_bounds) = gens.split_for_impl();
        tokens.extend(quote! {
            #[doc = ::std::concat!("Formatting type for genering the [expects](::ast_toolkit2::nibble::Parsable::expects())-implementation of [`", ::std::stringify!(#ident), "`].")]
            #[derive(::std::clone::Clone, ::std::marker::Copy, ::std::fmt::Debug, ::std::cmp::Eq, ::std::cmp::PartialEq)]
            pub struct Formatter #gens {
                #fmt_def
            }

            impl #impl_gen ::std::fmt::Display for Formatter #ty_gen #where_bounds {
                #[inline]
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    let Self { #fmt_inst } = self;
                    #fmt_impl
                    ::std::fmt::Result::Ok(())
                }
            }
        });
    }
}


struct FormatterImplTy<'e, 'g, 'a, 's, 't>(&'e Ident, &'g TypeGenerics<'a>, &'s FormatterImpl<'t>);
impl<'e, 'g, 'a, 's, 't> ToTokens for FormatterImplTy<'e, 'g, 'a, 's, 't> {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match &self.2 {
            FormatterImpl::Given(_) => tokens.extend(quote! { &'static ::std::primitive::str }),

            FormatterImpl::Generated(_) => {
                let error_mod = self.0;
                let ty_gen = self.1;
                tokens.extend(quote! { #error_mod::Formatter #ty_gen });
            },
        }
    }
}


struct FormatterImplInst<'e, 'g, 'a, 's, 't>(&'e Ident, &'g TypeGenerics<'a>, &'s FormatterImpl<'t>);
impl<'e, 'g, 'a, 's, 't> ToTokens for FormatterImplInst<'e, 'g, 'a, 's, 't> {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match &self.2 {
            FormatterImpl::Given(what) => what.to_tokens(tokens),

            FormatterImpl::Generated(fmt) => {
                let error_mod = self.0;
                let ty_gen = self.1;
                let fmt_inst: Punctuated<TokenStream2, Token![,]> = fmt
                    .into_iter()
                    .map(|(name, ty)| quote! { #name: <#ty as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::expects() })
                    .collect();
                tokens.extend(quote! { #error_mod::Formatter #ty_gen { #fmt_inst } });
            },
        }
    }
}


/// Everything we need to know about generating a formatter impl.
///
/// Note this is *either* a given string, or a generated formatter.
enum FormatterImpl<'t> {
    Given(&'t LitStr),
    Generated(Vec<(Ident, &'t Type)>),
}

// Updating
impl<'t> FormatterImpl<'t> {
    /// Adds a new field to the formatter.
    ///
    /// If this is already a [`FormatterImpl::Given`], this does nothing.
    fn add_field(&mut self, ident: Ident, ty: &'t Type) {
        let Self::Generated(fields) = self else { return };
        fields.push((ident, ty));
    }
}

// Formatting
impl<'t> FormatterImpl<'t> {
    /// Returns a generator for the type declaration of this FormatterImpl.
    fn def<'s, 'g>(&self, ident: &'s Ident, gens: &'g Generics) -> FormatterImplDef<'s, 'g, '_, 't> { FormatterImplDef(ident, gens, self) }

    /// Returns a generator for the typename for the formatter's type.
    fn ty<'e, 'g, 'a>(&self, error_mod: &'e Ident, ty_gen: &'g TypeGenerics<'a>) -> FormatterImplTy<'e, 'g, 'a, '_, 't> {
        FormatterImplTy(error_mod, ty_gen, self)
    }

    /// Returns a generator for the instantiation of whatever type this FormatterImpl is.
    fn inst<'e, 'g, 'a>(&self, error_mod: &'e Ident, ty_gen: &'g TypeGenerics<'a>) -> FormatterImplInst<'e, 'g, 'a, '_, 't> {
        FormatterImplInst(error_mod, ty_gen, self)
    }
}





/***** LIBRARY *****/
/// Handles the implementation.
pub fn handle_impl(outer_attrs: &[OuterAttr], vis: Visibility, ident: Ident, mut gens: Generics, data: DataEnum) -> Result<TokenStream2, Error> {
    // Turn that ident into a path for consistency
    let ident = Path {
        leading_colon: None,
        segments:      {
            let mut segs = Punctuated::new();
            segs.push(PathSegment { ident, arguments: PathArguments::None });
            segs
        },
    };

    // First of all, generate the implementations for every enum variant...
    let mut res = TokenStream2::new();
    for variant in &data.variants {
        // Parse its attributes as new outer attributes
        let var_attrs: Vec<OuterAttr> = parse_parse_attrs(&variant.attrs)?;

        // Pass it to the struct generator :TM:
        res.extend(super::seq::handle_impl(
            &var_attrs,
            vis.clone(),
            Path {
                leading_colon: None,
                segments:      {
                    let mut segs = ident.segments.clone();
                    segs.push(PathSegment { ident: variant.ident.clone(), arguments: PathArguments::None });
                    segs
                },
            },
            gens.clone(),
            variant.fields.clone(),
        )?);
    }

    // Check if there already exists an expects-string
    let mut fmt = OuterAttr::has_expects(outer_attrs).map(FormatterImpl::Given).unwrap_or_else(|| FormatterImpl::Generated(Vec::new()));
    let (whitespace_init, whitespace_pop) = whitespace_pop_impl("rem", outer_attrs);

    // Cook up the error module name
    let error_mod = cook_up_error_mod_name(outer_attrs, &ident);

    //     // Prepare the fields
    //     let mut inst = Punctuated::<Ident, Token![,]>::new();
    //     let mut err = ErrorImpl { vars: Vec::new() };
    //     let mut parse = TokenStream2::new();
    //     for (i, field) in data.fields.iter().enumerate() {
    //         // Get a name
    //         let name: Ident = field.ident.clone().unwrap_or_else(|| Ident::new(&format!("f{i}"), field.span()));
    //         let camel_name: Ident = Ident::new(&name.to_string().to_case(Case::Pascal), name.span());
    //         let ty = &field.ty;
    //
    //         // Generate the impl
    //         parse.extend(quote! {
    //             let (#name, rem): (#ty, ::ast_toolkit2::nibble::Slice<::std::primitive::u8>) = ::ast_toolkit2::nibble::error::ResultExt::map_nerr(
    //                 ::ast_toolkit2::nibble::error::ResultExt::map_fmt(
    //                     ::ast_toolkit2::nibble::Parsable::<::std::primitive::u8>::parse(rem),
    //                     |_| <Self as ::ast_toolkit2::nibble::Parsable::<::std::primitive::u8>>::expects()
    //                 ),
    //                 #error_mod::Error::#camel_name
    //             )?;
    //         });
    //
    //         // Save some stuff for later
    //         inst.push(name.clone());
    //         fmt.add_field(name, ty);
    //         err.add_var(camel_name, ty);
    //     }
    //     let inst: TokenStream2 = match &data.fields {
    //         Fields::Named(_) => quote! { Self { #inst } },
    //         Fields::Unnamed(_) => quote! { Self(#inst) },
    //         Fields::Unit => quote! { Self },
    //     };
    //
    //     // Prepare generics by inserting the bound
    //     inject_parsable_bound(&mut gens);
    //     let (impl_gen, ty_gen, where_bounds) = gens.split_for_impl();

    todo!()
}
