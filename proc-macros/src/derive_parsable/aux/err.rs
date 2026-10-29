//  ERROR IMPL.rs
//    by Lut99
//
//  Description:
//!   Defines a codegen helper for generating everything around `Parsable`
//!   errors.
//

use convert_case::{Case, Casing as _};
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::{Generics, Ident, Path, Type};


/***** GENERATORS *****/
/// Generates the definition of a Formatter.
pub struct ErrDef<'f, 'a> {
    err: &'f Err<'a>,
}
impl<'f, 'a> ToTokens for ErrDef<'f, 'a> {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        // Only do something if we're generating something
        let Err::Generate(gens, path, infos) = self.err else { return };

        // Compute the definition of every field in the formatter...
        let mut has_any_skip: bool = false;
        let vars_def: TokenStream2 = infos
            .into_iter()
            .map(|info| {
                let name = &info.var_name;
                let ty = &info.parsed_ty;
                if !info.skip {
                    // Normal implementation; refer to the parser's Formatter type.
                    let mut res = quote! { #name(<#ty as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::Error), };
                    if let Some((try_from_name, target_ty)) = &info.try_as {
                        // Add the TryFrom error variant too
                        res.extend(quote! { #try_from_name(<#target_ty as ::std::convert::TryFrom<#ty>>::Error), });
                    }
                    res
                } else {
                    has_any_skip = true;
                    // Special implementation; empty placeholder for correct generic typing.
                    quote! { #name(::std::marker::PhantomData<#ty>, ::std::convert::Infallible), }
                }
            })
            .collect();

        // Compute the `Display` implementation for every field...
        let mut fmt_impl: TokenStream2 = infos
            .into_iter()
            .filter(|info| !info.skip)
            .map(|info| {
                let name = &info.var_name;
                let mut res = quote! { Self::#name(err) => ::std::fmt::Display::fmt(err, f), };
                if let Some((try_from_name, target_ty)) = &info.try_as {
                    // Add the TryFrom error variant too
                    let sty = info.parsed_ty.to_token_stream().to_string();
                    let starget_ty = target_ty.to_token_stream().to_string();
                    res.extend(quote! { Self::#try_from_name(_) => ::std::write!(f, "Failed to convert from {:?} to {:?}", #sty, #starget_ty), });
                }
                res
            })
            .collect();
        if has_any_skip {
            // Add in the special catch-all match for the skipped variants
            fmt_impl.extend(quote! {
                // SAFETY: This arm is always skipped, as the remaining arms feature `Infallible`
                // and cannot be constructed
                _ => unsafe { ::std::hint::unreachable_unchecked() },
            });
        }

        // Generate the source impl
        let mut source_impl: TokenStream2 = infos
            .into_iter()
            .filter(|info| !info.skip)
            .map(|info| {
                let name = &info.var_name;
                let mut res = quote! { Self::#name(err) => ::std::error::Error::source(err), };
                if let Some((try_from_name, _)) = &info.try_as {
                    // Add the TryFrom error variant too
                    res.extend(quote! { Self::#try_from_name(err) => ::std::option::Option::Some(err), });
                }
                res
            })
            .collect();
        if has_any_skip {
            // Add in the special catch-all match for the skipped variants
            source_impl.extend(quote! {
                // SAFETY: This arm is always skipped, as the remaining arms feature `Infallible`
                // and cannot be constructed
                _ => unsafe { ::std::hint::unreachable_unchecked() },
            });
        }

        // Generate the final impl
        let (impl_gen, ty_gen, where_bounds) = gens.split_for_impl();
        tokens.extend(quote! {
            #[doc = ::std::concat!("Error type for parsing [`", ::std::stringify!(#path), "`].")]
            #[derive(::std::fmt::Debug, ::std::cmp::Eq, ::std::cmp::PartialEq)]
            pub enum Error #gens {
                #vars_def
            }

            #[automatically_derived]
            impl #impl_gen ::std::fmt::Display for Error #ty_gen #where_bounds {
                #[inline]
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    match self {
                        #fmt_impl
                    }
                }
            }

            #[automatically_derived]
            impl #impl_gen ::std::error::Error for Error #ty_gen #where_bounds {
                #[inline]
                fn source(&self) -> Option<&(dyn 'static + ::std::error::Error)> {
                    match self {
                        #source_impl
                    }
                }
            }
        });
    }
}



/// Generates the type reference for a Formatter.
pub struct ErrTy<'p, 'f, 'a> {
    mod_path: &'p Path,
    err:      &'f Err<'a>,
}
impl<'p, 'f, 'a> ToTokens for ErrTy<'p, 'f, 'a> {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self.err {
            Err::Generate(gens, _, _) => {
                let mod_path = self.mod_path;
                let (_, ty_gen, _) = gens.split_for_impl();
                tokens.extend(quote! { #mod_path::Error #ty_gen })
            },

            Err::Identity(info) => {
                let ty = &info.parsed_ty;
                tokens.extend(quote! { <#ty as ::ast_toolkit2::nibble::Parsable<::std::primitive::u8>>::Error })
            },
        }
    }
}



/// Generates an instantiation of a single variant.
pub struct ErrFromFn<'p, 'i, 'f, 'a> {
    mod_path: &'p Path,
    subparser: &'i Ident,
    err: &'f Err<'a>,
}
impl<'p, 'i, 'f, 'a> ToTokens for ErrFromFn<'p, 'i, 'f, 'a> {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self.err {
            Err::Generate(_, _, _) => {
                let mod_path = self.mod_path;
                let subparser = Ident::new(&self.subparser.to_string().to_case(Case::Pascal), self.subparser.span());
                tokens.extend(quote! { #mod_path::Error::#subparser })
            },

            Err::Identity(_) => tokens.extend(quote! { ::std::convert::identity }),
        }
    }
}



/// Generates an instantiation of a single variant.
pub struct ErrTryAsFromFn<'p, 'i, 'f, 'a> {
    mod_path: &'p Path,
    subparser: &'i Ident,
    err: &'f Err<'a>,
}
impl<'p, 'i, 'f, 'a> ToTokens for ErrTryAsFromFn<'p, 'i, 'f, 'a> {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self.err {
            Err::Generate(_, _, _) => {
                let mod_path = self.mod_path;
                let subparser = Ident::new(&format!("{}TryFrom", self.subparser.to_string().to_case(Case::Pascal)), self.subparser.span());
                tokens.extend(quote! { #mod_path::Error::#subparser })
            },

            Err::Identity(_) => tokens.extend(quote! { ::std::convert::identity }),
        }
    }
}





/***** AUXILLARY *****/
/// Defines everything the [`Err`] needs to know about nested parsers.
pub struct ErrInfo<'a> {
    /// Name of the variant for this error.
    pub var_name:  Ident,
    /// The type of the nested parser.
    ///
    /// Note that this is the parsed type; we're looking after `as` and such.
    pub parsed_ty: Type,
    /// Whether the parser is converting fallibly using `TryAs`. If so, will need an additional
    /// error enum variant.
    ///
    /// Contains the name of the additional variant + the target type.
    pub try_as:    Option<(Ident, &'a Type)>,
    /// Whether this parser is skipped or not.
    ///
    /// Skipped fields still need to be present in the Formatter impl to ensure there are no
    /// unused generics.
    pub skip:      bool,
}

impl<'a> ErrInfo<'a> {
    /// Constructor for the ErrInfo.
    ///
    /// # Arguments
    /// - `name`: An identifier with the name of the subparser for which we're generating the info.
    /// - `parsed_ty`: The parsed type (after As and TryAs) for this subparser.
    /// - `target_ty`: If given, indicates that there is `TryAs`-shenanigans going on and that this is
    ///   the converted-to type. This automatically makes `parsed_ty` the converted-from type.
    /// - `skip`: Whether the subparser should be skipped or not.
    pub fn new(name: &Ident, parsed_ty: Type, target_ty: Option<&'a Type>, skip: bool) -> Self {
        let camel_name = name.to_string().to_case(Case::Pascal);
        Self {
            var_name: Ident::new(&camel_name, name.span()),
            try_as: target_ty.map(|ty| (Ident::new(&format!("{camel_name}TryFrom"), name.span()), ty)),
            parsed_ty,
            skip,
        }
    }
}





/***** LIBRARY *****/
/// A codegen helper for generating everything around `Parsable` formatters.
pub enum Err<'a> {
    /// We'll be generating a new error type.
    Generate(&'a Generics, &'a Path, Vec<ErrInfo<'a>>),
    /// It's a single error - nothing needs to be done.
    Identity(ErrInfo<'a>),
}

// Constructors
impl<'a> Err<'a> {
    /// Creates a new Err.
    ///
    /// Upon running this function, it will decide whether error generation is necessary and, if
    /// not, what then.
    ///
    /// # Arguments
    /// - `gens`: A list of [`Generics`] that is equal to the type we're generating for.
    /// - `ident`: The identifier of the parent type we're generating for. Used only for debugging purposes.
    /// - `infos`: An iterator over [`ErrInfo`]s that describe each nested parser.
    ///
    /// # Returns
    /// A new Err that can be used to generate a Error implementation and every reference to
    /// it.
    pub fn new<'p: 'a, 'g: 'a, 'i: 'a>(path: &'p Path, gens: &'g Generics, infos: impl IntoIterator<Item = ErrInfo<'i>>) -> Self {
        // Decide what to do based on the number of infos
        let infos = infos.into_iter().collect::<Vec<ErrInfo<'i>>>();
        let nonskipped_count = infos.iter().filter(|i| !i.skip).count();
        if nonskipped_count == 1 {
            let info = infos.into_iter().filter(|i| !i.skip).next().unwrap();
            if info.try_as.is_none() { Self::Identity(info) } else { Self::Generate(gens, path, vec![info]) }
        } else {
            #[cfg(debug_assertions)]
            if nonskipped_count == 0 {
                panic!("Got an empty info list in `Err`, this should not happen")
            }
            Self::Generate(gens, path, infos)
        }
    }
}

// Information
impl<'a> Err<'a> {
    /// Returns whether this `Err` would generate an Error impl.
    #[inline]
    pub const fn generates(&self) -> bool { matches!(self, Self::Generate(_, _, _)) }
}

// Generation
impl<'a> Err<'a> {
    /// Returns a [`ToTokens`]-type for generating the Formatter implementation.
    ///
    /// Does nothing iff [`Fmt::will_generate()`] is false.
    pub const fn def(&self) -> ErrDef<'_, 'a> { ErrDef { err: self } }

    /// Returns a [`ToTokens`]-type for referring to the Formatter implementation's type.
    ///
    /// # Arguments
    /// - `mod_path`: The path to the module where the generated type lives.
    pub const fn ty<'p>(&self, mod_path: &'p Path) -> ErrTy<'p, '_, 'a> { ErrTy { mod_path, err: self } }

    /// Returns a [`ToTokens`]-type for referring to a specific Error variant for `from`-functions.
    ///
    /// # Arguments
    /// - `mod_path`: The path to the module where the generated type lives.
    /// - `subparser`: The specific variant to launch. Needn't be camelcase'd just yet.
    pub const fn from_fn<'p, 'i>(&self, mod_path: &'p Path, subparser: &'i Ident) -> ErrFromFn<'p, 'i, '_, 'a> {
        ErrFromFn { mod_path, subparser, err: self }
    }

    /// Returns a [`ToTokens`]-type for referring to a specific Error variant for `from`-functions
    /// when converting from a type.
    ///
    /// # Arguments
    /// - `mod_path`: The path to the module where the generated type lives.
    /// - `subparser`: The specific variant to launch. Needn't be camelcase'd just yet.
    pub const fn try_as_from_fn<'p, 'i>(&self, mod_path: &'p Path, subparser: &'i Ident) -> ErrTryAsFromFn<'p, 'i, '_, 'a> {
        ErrTryAsFromFn { mod_path, subparser, err: self }
    }
}
