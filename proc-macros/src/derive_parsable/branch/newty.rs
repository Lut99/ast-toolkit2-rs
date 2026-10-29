//  NEWTYPE IMPL.rs
//    by Lut99
//
//  Description:
//!   Defines a codegen helper for generating variant-specific parsing types.
//

use std::borrow::Cow;

use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::punctuated::Punctuated;
use syn::{Error, Generics, Ident, Path, PathArguments, PathSegment, Token, Type, TypePath, Variant, Visibility};

use super::super::VariantAttr;
use super::super::aux::{Err, ErrInfo, Fmt, FmtInfo, identity_path, inject_parsable_bound};
use super::super::seq::{FieldInfo, StructTy, cook_up_error_mod_name};
use crate::derive_parsable::parse_parse_attrs;


/***** GENERATORS *****/
pub struct NewtyDef<'s, 'a> {
    newty: &'s Newty<'a>,
}
impl<'s, 'a> ToTokens for NewtyDef<'s, 'a> {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        // Only do something if we're a (non-skipped!) generated variant
        let Newty::Generate(info) = self.newty else { return };
        if VariantAttr::has_skip(&info.var_attrs) {
            return;
        }

        // Generate the type definition itself
        let gens = info.gens;
        let (_, ty_gen, _) = gens.split_for_impl();
        let enum_ident = info.enum_ident;
        let var_ident = info.var_ident;
        let name = var_ident;
        tokens.extend(quote! {
            #[doc = ::std::concat!("Helper type for parsing [`", ::std::stringify!(#enum_ident), "::", ::std::stringify!(#var_ident), "`]")]
            pub struct #name #gens(pub #enum_ident #ty_gen);
        });


        /* Now generate the parser impl for the newtype. */
        // Do some outer attribute processing
        let error_mod_path = identity_path(info.error_mod.clone());

        // Decide on the formatter & error impls
        let pname = identity_path(name.clone());
        let fmt = Fmt::new("then", &info.var_attrs, &pname, gens, info.fields.iter().map(FmtInfo::from));
        let err = Err::new(&pname, &gens, info.fields.iter().map(ErrInfo::from));

        // Generate the mod implementation & main implementation logic
        let mod_impl = super::super::seq::generate_mod_impl(&info.error_mod, &Visibility::Public(Default::default()), name, &fmt, &err);
        let main_logic = super::super::seq::generate_parsable_impl_logic(&info.error_mod, &info.var_attrs, &info.fields, &err);

        // Generate the instantiation at the end of the parse
        let inst_impl: Punctuated<TokenStream2, Token![,]> = info.fields.iter().map(|field| field.name.to_token_stream()).collect();
        let inst_impl = match info.struct_ty {
            StructTy::Named => quote! { #name(#enum_ident::#var_ident { #inst_impl }) },
            StructTy::Unnamed => quote! { #name(#enum_ident::#var_ident(#inst_impl)) },
            StructTy::Unit => quote! { #name(#enum_ident::#var_ident) },
        };

        // Prepare generics by inserting the bound
        let mut gens = gens.clone();
        inject_parsable_bound(&mut gens);
        let (impl_gen, ty_gen, where_bounds) = gens.split_for_impl();

        // Done, generate the mod & impl for real
        let fmt_ty = fmt.ty(&error_mod_path);
        let fmt_inst = fmt.inst(&error_mod_path);
        let err_ty = err.ty(&error_mod_path);
        tokens.extend(quote! {
            #mod_impl

            #[automatically_derived]
            impl #impl_gen ::ast_toolkit2::nibble::Parsable<::std::primitive::u8> for #name #ty_gen #where_bounds {
                type Formatter = #fmt_ty;

                type Error = #err_ty;

                fn expects() -> Self::Formatter { #fmt_inst }

                fn parse(rem: ::ast_toolkit2::nibble::Slice<::std::primitive::u8>) -> ::std::result::Result<(Self, ::ast_toolkit2::nibble::Slice<::std::primitive::u8>), ::ast_toolkit2::nibble::error::NibbleError<Self::Formatter, Self::Error>> {
                    #main_logic
                    ::std::result::Result::Ok((#inst_impl, rem))
                }
            }
        })
    }
}



pub struct NewtyVarInst<'s, 'a> {
    newty: &'s Newty<'a>,
}
impl<'s, 'a> ToTokens for NewtyVarInst<'s, 'a> {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self.newty {
            Newty::Generate(_) => tokens.extend(quote! { res.0 }),

            Newty::Identity(info) => {
                let var_ident = info.var_ident;
                match &info.field_name {
                    Some(name) => tokens.extend(quote! { Self::#var_ident { #name: res } }),
                    None => tokens.extend(quote! { Self::#var_ident(res) }),
                }
            },
        }
    }
}





/***** AUXILLARY *****/
/// Defines everything we need to know per variant going in.
pub struct NewtyInfo<'a> {
    /// Error module for this newtype.
    pub error_mod: Ident,
    /// Variant attributes.
    pub var_attrs: Vec<VariantAttr>,
    /// The enum generics to store.
    pub gens: &'a Generics,
    /// The main type to generate for. This is the enum itself.
    pub enum_ident: &'a Ident,
    /// The specific variant name for this newty.
    pub var_ident: &'a Ident,
    /// The shape of the struct.
    pub struct_ty: StructTy,
    /// The fields in the variant.
    pub fields: Vec<FieldInfo>,
}

// Constructors
impl<'a> NewtyInfo<'a> {
    /// Constructor for the NewtyInfo
    ///
    /// # Arguments
    /// - `gens`: The [`Generics`] of the main enum type.
    /// - `enum_ident`: The main type to generate for. This is the enum itself.
    /// - `var_ident`: The specific variant name for this newty.
    /// - `fields`: The [`Fields`] in the variant.
    ///
    /// # Returns
    /// A new NewtyInfo with some additional fields generated.
    pub fn new<'g: 'a, 'e: 'a, 'v: 'a>(gens: &'g Generics, enum_ident: &'e Ident, var: &'v Variant) -> Result<Self, Error> {
        // Parse the attributes
        let var_attrs: Vec<VariantAttr> = parse_parse_attrs(&var.attrs)?;

        // Cook up the error module name
        let error_mod = cook_up_error_mod_name(&var_attrs, &var.ident);

        // Parse the fields
        let struct_ty = StructTy::from(&var.fields);
        let fields = var.fields.iter().cloned().enumerate().map(FieldInfo::try_from).collect::<Result<Vec<FieldInfo>, Error>>()?;
        if fields.iter().filter(|f| !f.skip).count() == 0 {
            return Err(Error::new(
                var.ident.span(),
                "Cannot derive `Parsable` on variant `{enum_ident}::{var_ident}` without any (unskipped) fields",
            ));
        }

        // Return self
        Ok(Self { error_mod, var_attrs, gens, enum_ident, var_ident: &var.ident, struct_ty, fields })
    }
}



pub struct IdentityInfo<'a> {
    /// The specific variant name for this newty.
    pub var_ident: &'a Ident,
    /// The name of the variable. If missing, is unnamed.
    pub field_name: Option<Ident>,
    /// The parsed type. Conversion source.
    pub parsed_ty: Type,
    /// The target type. Conversion target.
    pub target_ty: Option<Type>,
    /// Whether this variant is skipped or not.
    pub skip: bool,
}





/***** LIBRARY *****/
/// A codegen helper for generating variant-specific parsing types.
pub enum Newty<'a> {
    /// Generate a newtype for this variant.
    Generate(NewtyInfo<'a>),
    /// There is a single field - no need to generate anything.
    Identity(IdentityInfo<'a>),
}

// Constructors
impl<'a> Newty<'a> {
    /// Constructor for the Newty.
    ///
    /// Decides whether to actually generate anything or not.
    ///
    /// # Arguments
    /// - `info`: The [`NewtyInfo`] describing the variant for which to (potentially) generate a
    ///   struct.
    pub fn new(info: NewtyInfo<'a>) -> Self {
        // Shortcut to identity if there is precisely one field...
        let nonskipped_fields = info.fields.iter().filter(|f| !f.skip).count();
        if nonskipped_fields == 1 {
            let field = info.fields.into_iter().filter(|f| !f.skip).next().unwrap();
            let (parsed_ty, target_ty) = match (field.ty, field.as_ty) {
                (ty, Some(as_ty)) => (as_ty, Some(ty)),
                (ty, None) => (ty, None),
            };
            Self::Identity(IdentityInfo {
                var_ident: info.var_ident,
                field_name: if matches!(info.struct_ty, StructTy::Named) { Some(field.name) } else { None },
                parsed_ty,
                target_ty,
                skip: VariantAttr::has_skip(&info.var_attrs),
            })
        } else {
            #[cfg(debug_assertions)]
            if nonskipped_fields == 0 {
                panic!("Got empty field list oin Newty, this should never hoippen!")
            }
            Self::Generate(info)
        }
    }
}

// Information
impl<'a> Newty<'a> {
    /// Returns the name of the newtype.
    #[inline]
    pub fn name(&self) -> &Ident {
        match self {
            Self::Generate(info) => info.var_ident,
            Self::Identity(info) => info.var_ident,
        }
    }
    /// Returns the name of the variant for which this newtype is built.
    #[inline]
    pub fn var_name(&self) -> &Ident {
        match self {
            Self::Generate(info) => info.var_ident,
            Self::Identity(info) => info.var_ident,
        }
    }

    /// Returns the parsed type of the newtype.
    ///
    /// This is either the type, or the "as = ..." or "try_as = ..." type
    ///
    /// # Arguments
    /// - `error_mod`: The path pointing to the global type, if relevant.
    /// - `newty_error_mod`: If true, will also prepend the newtype's error mod.
    #[inline]
    pub fn parsed_ty(&self, error_mod: Option<&Ident>, newty_error_mod: bool) -> Cow<'_, Type> {
        match self {
            Self::Generate(info) => Cow::Owned(Type::Path(TypePath {
                attrs: Vec::new(),
                qself: None,
                path:  Path {
                    leading_colon: None,
                    segments:      {
                        let mut segs = Punctuated::new();
                        if let Some(error_mod) = error_mod {
                            segs.push(PathSegment { ident: error_mod.clone(), arguments: PathArguments::None });
                        }
                        if newty_error_mod {
                            segs.push(PathSegment { ident: info.error_mod.clone(), arguments: PathArguments::None });
                        }
                        segs.push(PathSegment { ident: info.var_ident.clone(), arguments: PathArguments::None });
                        segs
                    },
                },
            })),
            Self::Identity(info) => Cow::Borrowed(&info.parsed_ty),
        }
    }
    /// Returns the target type of the newtype.
    ///
    /// This is always the original type, but only given if there exists an "as = ..." or
    /// "try_as = ...".
    #[inline]
    pub fn target_ty(&self) -> Option<&Type> {
        match self {
            Self::Generate(_) => None,
            Self::Identity(info) => info.target_ty.as_ref(),
        }
    }

    /// Whether or not this variant is skipped.
    #[inline]
    pub fn skip(&self) -> bool {
        match self {
            Self::Generate(info) => VariantAttr::has_skip(&info.var_attrs),
            Self::Identity(info) => info.skip,
        }
    }
}

// Generators
impl<'a> Newty<'a> {
    /// Returns a [`ToTokens`]-type that generates the implementation of the newtype.
    pub const fn def(&self) -> NewtyDef<'_, 'a> { NewtyDef { newty: self } }

    /// Turns a variable called `res` into a new instance of the variant.
    pub const fn var_inst(&self) -> NewtyVarInst<'_, 'a> { NewtyVarInst { newty: self } }
}
