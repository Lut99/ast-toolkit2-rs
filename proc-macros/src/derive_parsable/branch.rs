//  BRANCH.rs
//    by Lut99
//
//  Description:
//!   Implements the logic for enums, i.e., branching parsers.
//

use proc_macro2::{Span, TokenStream as TokenStream2};
use syn::punctuated::Punctuated;
use syn::{DataEnum, Error, Generics, Ident, Path, PathArguments, PathSegment, Visibility};

use super::seq::{ErrorImpl, FieldInfo, FormatterImpl, StructTy, cook_up_error_mod_name};
use super::{OuterAttr, parse_parse_attrs, whitespace_pop_impl};


/***** HELPERS *****/
struct VariantInfo {
    /// Any generated module.
    mod_impl: TokenStream2,
    /// Any generated parsing impl.
    parsable_impl: TokenStream2,
    /// The name of the variant.
    name: Ident,
    /// Tokens representing the correct formatter type when using this variant.
    fmt_ty: TokenStream2,
    /// Tokens representing the correct error type when using this variant.
    err_ty: TokenStream2,
}





/***** LIBRARY *****/
/// Handles the implementation.
pub fn handle_impl(outer_attrs: &[OuterAttr], vis: Visibility, ident: Ident, mut gens: Generics, data: DataEnum) -> Result<TokenStream2, Error> {
    // Cook up the error module name
    let error_mod = cook_up_error_mod_name(outer_attrs, &Path {
        leading_colon: None,
        segments:      {
            let mut segs = Punctuated::new();
            segs.push(PathSegment { ident: ident.clone(), arguments: PathArguments::None });
            segs
        },
    });

    // First of all, generate the implementations for every enum variant...
    let mut variants = Vec::<VariantInfo>::new();
    for variant in &data.variants {
        // Parse its attributes as new outer attributes
        let var_attrs: Vec<OuterAttr> = parse_parse_attrs(&variant.attrs)?;
        let struct_ty = StructTy::from(&variant.fields);
        let fields = variant.fields.iter().cloned().enumerate().map(TryFrom::try_from).collect::<Result<Vec<FieldInfo>, Error>>()?;

        // Decide if we need to generate a special parser for this
        let nonskipped_count: usize = fields.iter().filter(|f| !f.skip).count();
        if nonskipped_count == 1 {
            // No, there is precisely one parser; just use the type directly.
            let field = fields.first().unwrap();
            variants.push(VariantInfo {
                mod_impl: TokenStream2::new(),
                parsable_impl: TokenStream2::new(),
                name: variant.ident.clone(),
                fmt_ty: {
                    quote! {}
                },
                err_ty: quote! {},
            });
            continue;
        } else {
            // This variant has no fields; nothing to parse
        }

        // Decide on the formatter & error impls
        let fmt = FormatterImpl::new(outer_attrs, &fields);
        let err = ErrorImpl::new(&fields);

        // Generate the mod implementation & main implementation
        let mod_impl = generate_mod_impl(&error_mod, &vis, &constr, &gens, &fmt, &err);
        let main_impl = generate_parsable_impl(&error_mod, outer_attrs, &ident, &constr, gens, struct_ty, &fields, &fmt, &err);
    }

    // Get some whitespace info
    let (whitespace_init, whitespace_pop) = whitespace_pop_impl("rem", outer_attrs);



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
