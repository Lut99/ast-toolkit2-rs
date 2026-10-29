//  FORMATTER IMPL.rs
//    by Lut99
//
//  Description:
//!   Defines a codegen helper for generating everything around `Parsable`
//!   formatters.
//

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, quote};
use syn::{Generics, LitInt, LitStr, Path, Type};

use super::super::OuterAttr;


/***** GENERATORS *****/
/// Generates the definition of a Formatter.
pub struct FmtDef<'f, 'a> {
    fmt: &'f Fmt<'a>,
}
impl<'f, 'a> ToTokens for FmtDef<'f, 'a> {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        // Only do something if we're generating something
        let Fmt::Generate(conn, gens, path, infos) = self.fmt else { return };

        // Compute the definition of every field in the formatter...
        let fields_def: TokenStream2 = infos
            .into_iter()
            .map(|info| {
                let ty = info.parsed_ty;
                if !info.skip {
                    // Normal implementation; refer to the parser's Formatter type.
                    quote! { pub <#ty as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::Formatter, }
                } else {
                    // Special implementation; empty placeholder for correct generic typing.
                    quote! { pub ::std::marker::PhantomData<#ty>, }
                }
            })
            .collect();

        // Compute the `Display` implementation for every field...
        let mut fmt_impl = TokenStream2::new();
        let mut first: bool = true;
        for (i, info) in infos.into_iter().enumerate() {
            // See if we should skip this one
            if info.skip {
                continue;
            }

            // Add the couple word if _not_ the first
            if first {
                first = false;
            } else {
                fmt_impl.extend(quote! { ::std::write!(f, ::std::concat!(", ", #conn, " "))?; });
            }

            // Write the main impl
            let i = LitInt::new(&i.to_string(), Span::call_site());
            fmt_impl.extend(quote! { ::std::fmt::Display::fmt(&self.#i, f)?; });
        }

        // Generate the final impl
        let (impl_gen, ty_gen, where_bounds) = gens.split_for_impl();
        tokens.extend(quote! {
            #[doc = ::std::concat!("Formatting type for genering the [expects](::ast_toolkit2::nibble::Parsable::expects())-implementation of [`", ::std::stringify!(#path), "`].")]
            #[derive(::std::clone::Clone, ::std::marker::Copy, ::std::fmt::Debug, ::std::cmp::Eq, ::std::cmp::PartialEq)]
            pub struct Formatter #gens(
                #fields_def
            );

            #[automatically_derived]
            impl #impl_gen ::std::fmt::Display for Formatter #ty_gen #where_bounds {
                #[inline]
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    #fmt_impl
                    ::std::fmt::Result::Ok(())
                }
            }
        });
    }
}



/// Generates the type reference for a Formatter.
pub struct FmtTy<'p, 'f, 'a> {
    mod_path: &'p Path,
    fmt:      &'f Fmt<'a>,
}
impl<'p, 'f, 'a> ToTokens for FmtTy<'p, 'f, 'a> {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self.fmt {
            Fmt::Generate(_, gens, _, _) => {
                let mod_path = self.mod_path;
                let (_, ty_gen, _) = gens.split_for_impl();
                tokens.extend(quote! { #mod_path::Formatter #ty_gen })
            },

            Fmt::Identity(info) => {
                let ty = &info.parsed_ty;
                tokens.extend(quote! { <#ty as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::Formatter })
            },

            Fmt::Manual(_) => tokens.extend(quote! { &'static ::std::primitive::str }),
        }
    }
}



/// Generates the type reference for a Formatter.
pub struct FmtInst<'p, 'f, 'a> {
    mod_path: &'p Path,
    fmt:      &'f Fmt<'a>,
}
impl<'p, 'f, 'a> ToTokens for FmtInst<'p, 'f, 'a> {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self.fmt {
            Fmt::Generate(_, _, _, infos) => {
                let mod_path = self.mod_path;
                let inst_impl: TokenStream2 = infos
                    .into_iter()
                    .map(|i| {
                        if !i.skip {
                            let ty = &i.parsed_ty;
                            quote! { <#ty as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::expects(), }
                        } else {
                            quote! { ::std::marker::PhantomData, }
                        }
                    })
                    .collect();
                tokens.extend(quote! { #mod_path::Formatter(#inst_impl) })
            },

            Fmt::Identity(info) => {
                let ty = &info.parsed_ty;
                tokens.extend(quote! { <#ty as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::expects() })
            },

            Fmt::Manual(lit) => lit.to_tokens(tokens),
        }
    }
}





/***** AUXILLARY *****/
/// Defines everything the [`Fmt`] needs to know about nested parsers.
pub struct FmtInfo<'a> {
    /// The type of the nested parser.
    ///
    /// Note that this is the parsed type; we're looking after `as` and such.
    pub parsed_ty: &'a Type,
    /// Whether this parser is skipped or not.
    ///
    /// Skipped fields still need to be present in the Formatter impl to ensure there are no
    /// unused generics.
    pub skip:      bool,
}





/***** LIBRARY *****/
/// A codegen helper for generating everything around `Parsable` formatters.
pub enum Fmt<'a> {
    // YES generation
    /// The type we're generating `Parsable` for has multiple formatters, time to pull up our
    /// sleeves.
    Generate(&'static str, &'a Generics, &'a Path, Vec<FmtInfo<'a>>),

    // NO generation
    /// The user specified a manual `expects`-string.
    Manual(&'a LitStr),
    /// A type we're generating `Parsable` for has only one nested parser. No need to generate
    /// anything extra.
    Identity(FmtInfo<'a>),
}

// Constructors
impl<'a> Fmt<'a> {
    /// Creates a new Fmt.
    ///
    /// Upon running this function, it will decide whether formatter generation is necessary and,
    /// if not, what then.
    ///
    /// # Arguments
    /// - `conn`: A small word indicating the connective in the list of formatters. E.g., `then` or `or`.
    /// - `attrs`: A list of [`OuterAttr`]s that may have a relevant `#[parse(expected = ...)]`.
    /// - `gens`: A list of [`Generics`] that is equal to the type we're generating for.
    /// - `ident`: The identifier of the parent type we're generating for. Used only for debugging purposes.
    /// - `infos`: An iterator over [`FmtInfo`]s that describe each nested parser.
    ///
    /// # Returns
    /// A new Fmt that can be used to generate a Formatter implementation and every reference to
    /// it.
    pub fn new<'o: 'a, 'p: 'a, 'g: 'a, 'i: 'a>(
        conn: &'static str,
        attrs: &'o [OuterAttr],
        path: &'p Path,
        gens: &'g Generics,
        infos: impl IntoIterator<Item = FmtInfo<'i>>,
    ) -> Self {
        // See if the user overrides whatever we thought of
        if let Some(expected) = OuterAttr::has_expects(attrs) {
            return Self::Manual(expected);
        }

        // Decide what to do based on the number of infos
        let infos = infos.into_iter().collect::<Vec<FmtInfo<'i>>>();
        let nonskipped_count = infos.iter().filter(|i| !i.skip).count();
        if nonskipped_count == 1 {
            Self::Identity(infos.into_iter().filter(|i| !i.skip).next().unwrap())
        } else {
            #[cfg(debug_assertions)]
            if nonskipped_count == 0 {
                panic!("Got an empty info list in `Fmt`, this should not happen")
            }
            Self::Generate(conn, gens, path, infos)
        }
    }
}

// Information
impl<'a> Fmt<'a> {
    /// Returns whether this `Fmt` would generate a Formatter impl.
    #[inline]
    pub const fn generates(&self) -> bool { matches!(self, Self::Generate(_, _, _, _)) }
}

// Generation
impl<'a> Fmt<'a> {
    /// Returns a [`ToTokens`]-type for generating the Formatter implementation.
    ///
    /// Does nothing iff [`Fmt::will_generate()`] is false.
    pub const fn def(&self) -> FmtDef<'_, 'a> { FmtDef { fmt: self } }

    /// Returns a [`ToTokens`]-type for referring to the Formatter implementation's type.
    ///
    /// # Arguments
    /// - `mod_path`: The path to the module where the generated type lives.
    pub const fn ty<'p>(&self, mod_path: &'p Path) -> FmtTy<'p, '_, 'a> { FmtTy { mod_path, fmt: self } }

    /// Returns a [`ToTokens`]-type for referring to the Formatter implemenation instantiation.
    ///
    /// # Arguments
    /// - `mod_path`: The path to the module where the generated type lives.
    pub const fn inst<'p>(&self, mod_path: &'p Path) -> FmtInst<'p, '_, 'a> { FmtInst { mod_path, fmt: self } }
}
