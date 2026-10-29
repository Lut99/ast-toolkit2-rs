//  BRANCH.rs
//    by Lut99
//
//  Description:
//!   Implements the logic for enums, i.e., branching parsers.
//

// Modules
mod newty;

// Imports
use newty::{Newty, NewtyInfo};
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens as _, quote};
use syn::{DataEnum, Error, Generics, Ident, Visibility};

use super::OuterAttr;
use super::aux::{Err, ErrInfo, Fmt, FmtInfo, identity_path, inject_parsable_bound, whitespace_pop_impl};
use super::seq::cook_up_error_mod_name;


/***** LIBRARY *****/
/// Handles the implementation.
pub fn handle_impl(outer_attrs: &[OuterAttr], vis: Visibility, ident: Ident, gens: Generics, data: DataEnum) -> Result<TokenStream2, Error> {
    // Assert there are enough things to parse.
    if data.variants.is_empty() {
        return Err(Error::new(ident.span(), "Cannot derive `Parsable` on empty enums"));
    }

    // Cook up the error module name
    let error_mod = cook_up_error_mod_name(outer_attrs, &ident);
    let error_mod_path = identity_path(error_mod.clone());

    // Start by collecting how to generate the newtypes for every variant
    let newtys: Vec<Newty> = data
        .variants
        .iter()
        .map(|v| {
            // Build the info and the newtype around it
            Ok(Newty::new(NewtyInfo::new(&gens, &ident, v)?))
        })
        .collect::<Result<Vec<Newty>, Error>>()?;

    // Then collect how to generate the formatter & error types for the whole enum
    let pident = identity_path(ident.clone());
    let fmt = Fmt::new("or", outer_attrs, &pident, &gens, newtys.iter().map(|nt| FmtInfo { parsed_ty: nt.parsed_ty(None, false).into_owned(), parsed_ty_inst: Some(nt.parsed_ty(Some(&error_mod), false).into_owned()), skip: nt.skip() }));
    let err = Err::new(&pident, &gens, newtys.iter().map(|nt| ErrInfo::new(nt.name(), nt.parsed_ty(None, false).into_owned(), nt.target_ty(), nt.skip())));

    // Get some whitespace info
    let (whitespace_init, whitespace_pop) = whitespace_pop_impl("rem", outer_attrs);

    // Generate the module with the newtypes, error- and formatter types
    let newty_def: TokenStream2 = newtys.iter().map(|nt| nt.def().to_token_stream()).collect();
    let fmt_def = fmt.def();
    let err_def = err.def();
    let mut res = TokenStream2::new();
    res.extend(quote! {
        #[doc = ::std::concat!("Helper module for [`", ::std::stringify!(#ident), "`] containing auxillary types.")]
        #vis mod #error_mod {
            use super::*;

            #newty_def

            #fmt_def

            #err_def
        }
    });

    // Generate the implementation of every variant
    let parse_impl: TokenStream2 = newtys.iter().filter(|nt| !nt.skip()).map(|nt| {
        // Generate the impl
        let ty = nt.parsed_ty(Some(&error_mod), false);
        let var_inst = nt.var_inst();
        let err_fn = err.from_fn(&error_mod_path, nt.var_name());
        quote! {
            match <#ty as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::parse(::std::clone::Clone::clone(&input)) {
                ::std::result::Result::Ok((res, rem)) => return ::std::result::Result::Ok((#var_inst, rem)),
                ::std::result::Result::Err(::ast_toolkit2::nibble::error::NibbleError::Unmatched(_, _)) => {},
                ::std::result::Result::Err(::ast_toolkit2::nibble::error::NibbleError::NotEnough(_, needed, loc)) => {
                    return ::std::result::Result::Err(::ast_toolkit2::nibble::error::NibbleError::NotEnough(
                        <Self as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::expects(),
                        needed,
                        loc,
                    ))
                },
                ::std::result::Result::Err(::ast_toolkit2::nibble::error::NibbleError::Error(err)) => {
                    return ::std::result::Result::Err(::ast_toolkit2::nibble::error::NibbleError::Error(#err_fn(err)));
                },
            }
        }
    }).collect();

    // Prepare generics by inserting the bound
    let mut gens = gens.clone();
    inject_parsable_bound(&mut gens);
    let (impl_gen, ty_gen, where_bounds) = gens.split_for_impl();

    // Then generate the parser impl for the enum itself
    let fmt_ty = fmt.ty(&error_mod_path);
    let fmt_inst = fmt.inst(&error_mod_path);
    let err_ty = err.ty(&error_mod_path);
    res.extend(quote! {
        impl #impl_gen ::ast_toolkit2::nibble::Parsable<::std::primitive::u8> for #ident #ty_gen #where_bounds {
            type Formatter = #fmt_ty;
            type Error = #err_ty;

            #[inline]
            fn expects() -> Self::Formatter { #fmt_inst }

            #[inline]
            fn parse(input: ::ast_toolkit2::nibble::Slice<::std::primitive::u8>) -> ::std::result::Result<(Self, ::ast_toolkit2::nibble::Slice<::std::primitive::u8>), ::ast_toolkit2::nibble::error::NibbleError<Self::Formatter, Self::Error>> {
                // Pop whitespace first
                #[allow(unused)]
                type Match<'a> = ::ast_toolkit2::nibble::__macros::Match<'a>;
                #[allow(unused)]
                type Regex = ::ast_toolkit2::nibble::__macros::Regex;
                #whitespace_init
                #whitespace_pop
        
                // Then pop normally
                #parse_impl
                ::std::result::Result::Err(::ast_toolkit2::nibble::error::NibbleError::Unmatched(<Self as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::expects(), ::ast_toolkit2::nibble::Slice::error_loc(&input)))
            }
        }
    });

    // Now run the generation ~~
    Ok(res)
}
