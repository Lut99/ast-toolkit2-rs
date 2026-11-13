//  DERIVE DIAGNOSTIC.rs
//    by Lut99
//
//  Description:
//!   Implements the derive macro for `Diagnostic`.
//

use std::hash::{DefaultHasher, Hasher as _};

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream, Parser as _};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Paren;
use syn::visit_mut::VisitMut;
use syn::{
    Attribute, Data, DataUnion, DeriveInput, Error, Expr, ExprLit, ExprPath, Fields, Ident, Lit, LitStr, Meta, Path, PathArguments, PathSegment,
    Stmt, Token, Type, TypePath, parenthesized,
};

use crate::common::{inject_trait_bound_with_args, split_for_impl_with_extra};


/***** HELPER FUNCTIONS *****/
/// Generates the default element type path.
#[inline]
fn default_elemty() -> Type {
    Type::Path(TypePath {
        attrs: Vec::new(),
        qself: None,
        path:  Path {
            leading_colon: None,
            segments:      {
                let mut segs = Punctuated::new();
                segs.push(PathSegment { ident: Ident::new("__E", Span::mixed_site()), arguments: syn::PathArguments::None });
                segs
            },
        },
    })
}



/// Generates the field expansion (i.e., `{...}` in `let Self { ... } = self`) of a set of [`Fields`].
///
/// The boolean indicates whether the fields have names (i.e., struct syntax) or are implicit
/// (i.e., tuple syntax).
fn generate_field_expansion(fields: &Fields) -> TokenStream2 {
    let fields: Vec<Ident> = fields
        .iter()
        .enumerate()
        .map(|(i, f)| {
            if let Some(ident) = &f.ident {
                Ident::new(&ident.to_string(), Span::mixed_site())
            } else {
                Ident::new(&format!("f{i}"), Span::mixed_site())
            }
        })
        .collect();
    quote!(#(#fields),*)
}

/// Generates the field expansion for structs (`let Self { ... } = self`).
#[inline]
fn generate_field_expansion_struct(fields: &Fields) -> TokenStream2 {
    let exp = generate_field_expansion(fields);
    match fields {
        Fields::Named(_) => quote! { let Self { #exp } = self; },
        Fields::Unnamed(_) => quote! { let Self ( #exp ) = self; },
        Fields::Unit => TokenStream2::new(),
    }
}

/// Generates the field expansion for enum variants (`Self::$variant { ... }`).
#[inline]
fn generate_field_expansion_variant(variant: &Ident, fields: &Fields) -> TokenStream2 {
    let exp = generate_field_expansion(fields);
    match fields {
        Fields::Named(_) => quote! { Self::#variant { #exp } },
        Fields::Unnamed(_) => quote! { Self::#variant ( #exp ) },
        Fields::Unit => quote! { Self::#variant },
    }
}

/// Parses a format expression (either a literal or multiple expressions wrapped in `()`).
fn parse_fmt_exprs(input: ParseStream) -> Result<Punctuated<Expr, Token![,]>, Error> {
    fn parenthesized(input: ParseStream) -> Result<(Paren, Punctuated<Expr, Token![,]>), Error> {
        let content;
        let parens = parenthesized!(content in input);
        let mut expr: Punctuated<Expr, Token![,]> = Punctuated::parse_terminated(&content)?;
        if !content.is_empty() {
            return Err(content.error("Unexpected tokens after expression"));
        }
        // Ensure to process `{0}` and such
        for expr in &mut expr {
            UnnamedFieldVisitor.visit_expr_mut(expr);
        }
        Ok((parens, expr))
    }


    if let Ok(lit) = input.parse::<LitStr>() {
        let mut exprs = Punctuated::new();
        exprs.push(Expr::Lit(ExprLit { attrs: Vec::new(), lit: Lit::Str(lit) }));
        Ok(exprs)
    } else if let Ok((_, exprs)) = parenthesized(input) {
        Ok(exprs)
    } else {
        Err(input.error("Expected string literal or format expression wrapped in parenthesis (`()`)"))
    }
}

/// Parses a loc expressions (either `loc = ...` or just `loc`, which aliases to `loc = loc`).
fn parse_loc_expr(input: ParseStream) -> Result<Option<Expr>, Error> {
    if input.parse::<Token![=]>().is_ok() {
        let mut expr = input.parse::<Expr>()?;
        UnnamedFieldVisitor.visit_expr_mut(&mut expr);
        Ok(Some(expr))
    } else if input.is_empty() {
        Ok(None)
    } else {
        Err(input.error("Expected either nothing or `= <expr>`"))
    }
}

/// Takes some [`syn`] object and ensures that all identifiers are resolved in the mixed scope.
struct IdentVisitor;
impl VisitMut for IdentVisitor {
    fn visit_ident_mut(&mut self, ident: &mut Ident) { ident.set_span(Span::mixed_site()); }
    fn visit_lit_str_mut(&mut self, lstr: &mut LitStr) { lstr.set_span(Span::mixed_site()); }
}

struct UnnamedFieldVisitor;
impl VisitMut for UnnamedFieldVisitor {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        // Delve to discover any expression of the shape `{ <LITINT> }`
        let Expr::Block(block) = expr else { return syn::visit_mut::visit_expr_mut(self, expr) };
        let [stmt] = block.block.stmts.as_mut_slice() else { return self.visit_expr_block_mut(block) };
        let Stmt::Expr(inner, _) = stmt else { return self.visit_stmt_mut(stmt) };
        let Expr::Lit(ExprLit { attrs: _, lit: Lit::Int(lit) }) = inner else { return self.visit_expr_mut(inner) };

        // Translate it into a field identifier
        *expr = Expr::Path(ExprPath {
            attrs: block.attrs.drain(..).collect(),
            qself: None,
            path:  Path {
                leading_colon: None,
                segments:      {
                    let mut segs = Punctuated::new();
                    segs.push(PathSegment {
                        ident:     Ident::new(&format!("f{}", lit.base10_digits()), lit.span()),
                        arguments: PathArguments::None,
                    });
                    segs
                },
            },
        });
    }
}

/// Takes whether there is a format string for the main message and renders a build  string for it if not.
struct RenderMsgBuild(Ident, bool);
impl RenderMsgBuild {
    #[inline]
    fn new(ident: &Ident, has_format_string: bool) -> Self {
        Self(
            Ident::new(
                &format!("msg_{:x}", {
                    let mut hasher = DefaultHasher::new();
                    hasher.write(ident.to_string().as_bytes());
                    hasher.finish()
                }),
                ident.span(),
            ),
            has_format_string,
        )
    }
}
impl ToTokens for RenderMsgBuild {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        if !self.1 {
            let ident = &self.0;
            tokens.extend(quote!(let #ident = ::std::string::ToString::to_string(&self);));
        }
    }
}
/// Takes a format string for the main message and renders it.
struct RenderMainMsg<'b, 'e>(&'b RenderMsgBuild, &'e Option<Punctuated<Expr, Token![,]>>);
impl<'b, 'e> ToTokens for RenderMainMsg<'b, 'e> {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self.1 {
            Some(exprs) => tokens.extend(quote!(::std::format!(#exprs))),
            None => {
                let ident = &self.0.0;
                tokens.extend(quote!(#ident))
            },
        }
    }
}

/// Takes a format string for any other format and renders it.
///
/// In contrast to [`render_main_msg()`], this renders something optionally.
fn render_fstr(exprs: &Option<Punctuated<Expr, Token![,]>>) -> TokenStream2 {
    match exprs {
        Some(exprs) => quote! { ::std::option::Option::Some(::std::format!(#exprs)) },
        None => quote! { ::std::option::Option::None },
    }
}





/***** ATTRIBUTES *****/
/// Shadows the original `ast-toolkit2`s `Severity`
#[derive(Clone, Copy, Eq, PartialEq)]
enum Severity {
    Error,
    Help,
    Suggestion,
    Warning,
}
impl ToTokens for Severity {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            Self::Error => tokens.extend(quote! { ::ast_toolkit2::diag::Severity::Error }),
            Self::Help => tokens.extend(quote! { ::ast_toolkit2::diag::Severity::Help }),
            Self::Suggestion => tokens.extend(quote! { ::ast_toolkit2::diag::Severity::Suggestion }),
            Self::Warning => tokens.extend(quote! { ::ast_toolkit2::diag::Severity::Warning }),
        }
    }
}



/// Defines possible `#[diag(...)]`-attributes.
enum DiagAttr {
    /// Defines the severity of an attribute row.
    Severity(Severity, Span),
    /// Defines a code
    Code(Punctuated<Expr, Token![,]>, Span),
    /// Defines a message
    Message(Punctuated<Expr, Token![,]>, Span),
    /// Defines an annotation expression
    Loc(Option<Expr>, Span),
}
impl DiagAttr {
    /// Does the parsing but without the check for toplevelattr
    fn parse_ident(ident: &Ident, input: ParseStream) -> syn::Result<Self> {
        if ident == "error" {
            Ok(Self::Severity(Severity::Error, ident.span()))
        } else if ident == "help" {
            Ok(Self::Severity(Severity::Help, ident.span()))
        } else if ident == "suggestion" {
            Ok(Self::Severity(Severity::Suggestion, ident.span()))
        } else if ident == "warn" || ident == "warning" {
            Ok(Self::Severity(Severity::Warning, ident.span()))
        } else if ident == "code" {
            // Then do the equals-value thingy
            input.parse::<Token![=]>()?;
            Ok(Self::Code(parse_fmt_exprs(input)?, ident.span()))
        } else if ident == "msg" || ident == "message" {
            // Then do the equals-value thingy
            input.parse::<Token![=]>()?;
            Ok(Self::Message(parse_fmt_exprs(input)?, ident.span()))
        } else if ident == "loc" || ident == "location" {
            // Then do the equals-value thingy
            Ok(Self::Loc(parse_loc_expr(input)?, ident.span()))
        } else {
            Err(Error::new(ident.span(), "Unknown struct- or variant attribute"))
        }
    }
}
impl Parse for DiagAttr {
    #[inline]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Ignore anything that's a toplevelattr
        let ident: Ident = input.parse()?;
        Self::parse_ident(&ident, input)
    }
}

/// Defines possible `#[annot(...)]`-attributes.
enum AnnotAttr {
    /// Defines the severity of an attribute row.
    Severity(Severity, Span),
    /// Defines a message
    Message(Punctuated<Expr, Token![,]>, Span),
    /// Defines a replacement
    Replace(Type, Expr, Span),
    /// Defines an annotation expression
    Loc(Option<Expr>, Span),
}
impl Parse for AnnotAttr {
    #[inline]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        if ident == "error" {
            Ok(Self::Severity(Severity::Error, ident.span()))
        } else if ident == "help" {
            Ok(Self::Severity(Severity::Help, ident.span()))
        } else if ident == "suggestion" {
            Ok(Self::Severity(Severity::Suggestion, ident.span()))
        } else if ident == "warn" || ident == "warning" {
            Ok(Self::Severity(Severity::Warning, ident.span()))
        } else if ident == "repl" || ident == "replace" {
            // Then do the equals-value thingy
            input.parse::<Token![<]>()?;
            let ty: Type = input.parse()?;
            input.parse::<Token![>]>()?;
            input.parse::<Token![=]>()?;
            let mut expr = input.parse::<Expr>()?;
            UnnamedFieldVisitor.visit_expr_mut(&mut expr);
            Ok(Self::Replace(ty, expr, ident.span()))
        } else if ident == "msg" || ident == "message" {
            // Then do the equals-value thingy
            input.parse::<Token![=]>()?;
            Ok(Self::Message(parse_fmt_exprs(input)?, ident.span()))
        } else if ident == "loc" || ident == "location" {
            // Then do the equals-value thingy
            Ok(Self::Loc(parse_loc_expr(input)?, ident.span()))
        } else {
            Err(Error::new(ident.span(), "Unknown struct- or variant attribute"))
        }
    }
}



/// Defines what we expect to learn about a single, non-main annotation.
struct Annot {
    /// The severity
    sev:  Severity,
    /// The message (if any)
    msg:  Option<Punctuated<Expr, Token![,]>>,
    /// The replacement (if any)
    repl: Option<(Type, Expr)>,
    /// The location expression
    loc:  Expr,
}

/// Defines what we expect to learn from toplevel (struct) or variant (enum) attributes.
struct Diag {
    /// The type of element to use.
    elemty: Option<Type>,

    /// The main severity
    sev:    Severity,
    /// The main message
    msg:    Option<Punctuated<Expr, Token![,]>>,
    /// The main code
    code:   Option<Punctuated<Expr, Token![,]>>,
    /// A list of annotations
    annots: Vec<Annot>,
}
impl Diag {
    /// Generates a list tokens representing the constructions of each annotation.
    fn generate_annots(&self, diag_gen: &Type) -> Vec<TokenStream2> {
        self.annots
            .iter()
            .map(|a| {
                let asev: Severity = a.sev;
                let amsg: TokenStream2 = render_fstr(&a.msg);
                let arepl: TokenStream2 = a
                    .repl
                    .as_ref()
                    .map(|(_, e)| quote! { ::std::option::Option::Some(::std::convert::Into::<::std::vec::Vec<#diag_gen>>::into(#e)) })
                    .unwrap_or_else(|| quote! { ::std::option::Option::None });
                let aloc: &Expr = &a.loc;
                quote! {
                    ::ast_toolkit2::diag::Annotation::<#diag_gen> {
                        repl: #arepl,
                        msg: #amsg,
                        sev: #asev,
                        loc: ::ast_toolkit2::loc::Located::loc(&#aloc),
                    }
                }
            })
            .collect()
    }

    /// Parses the VariantAttrs from a given list of attributes.
    fn parse_from_attrs(topspan: Span, has_named_fields: bool, attrs: &[Attribute]) -> Result<Self, Error> {
        let mut elemty: Option<Type> = None;
        let mut sev: Option<Severity> = None;
        let mut code: Option<Punctuated<Expr, Token![,]>> = None;
        let mut msg: Option<Punctuated<Expr, Token![,]>> = None;
        let mut loc: Option<Option<Expr>> = None;
        let mut annots: Vec<Annot> = Vec::with_capacity(4);
        for attr in attrs {
            // Filter out our own attributes only
            match &attr.meta {
                // We look for `#[diag(...)]`...
                Meta::List(l) if l.path.is_ident("diag") => {
                    // Good. Now parse the list as meta attributes themselves...
                    let inner: Punctuated<DiagAttr, Token![,]> = Punctuated::parse_terminated.parse2(l.tokens.clone())?;
                    for attr in inner {
                        match attr {
                            DiagAttr::Severity(dsev, span) => {
                                if let Some(old_sev) = &sev
                                    && &dsev != old_sev
                                {
                                    return Err(Error::new(span, "Conflicting main severity specified"));
                                } else {
                                    sev = Some(dsev)
                                }
                            },
                            DiagAttr::Code(dcode, span) => {
                                if code.is_none() {
                                    code = Some(dcode);
                                } else {
                                    return Err(Error::new(span, "Conflicting main message code specified"));
                                }
                            },
                            DiagAttr::Message(dmsg, span) => {
                                if msg.is_none() {
                                    msg = Some(dmsg);
                                } else {
                                    return Err(Error::new(span, "Conflicting main message specified"));
                                }
                            },
                            DiagAttr::Loc(dloc, span) => {
                                if loc.is_none() {
                                    loc = Some(dloc);
                                } else {
                                    return Err(Error::new(span, "Conflicting main location reference specified"));
                                }
                            },
                        }
                    }
                },
                // ...and `#[annot(...)]`
                Meta::List(l) if l.path.is_ident("annot") => {
                    let mut asev: Option<Severity> = None;
                    let mut amsg: Option<Punctuated<Expr, Token![,]>> = None;
                    let mut arepl: Option<(Type, Expr)> = None;
                    let mut aloc: Option<Option<Expr>> = None;

                    // Good. Now parse the list as meta attributes themselves...
                    let inner: Punctuated<AnnotAttr, Token![,]> = Punctuated::parse_terminated.parse2(l.tokens.clone())?;
                    for attr in inner {
                        match attr {
                            AnnotAttr::Severity(dsev, span) => {
                                if let Some(old_sev) = &asev
                                    && &dsev != old_sev
                                {
                                    return Err(Error::new(span, "Conflicting severity specified"));
                                } else {
                                    asev = Some(dsev)
                                }
                            },
                            AnnotAttr::Message(dmsg, span) => {
                                if amsg.is_none() {
                                    amsg = Some(dmsg);
                                } else {
                                    return Err(Error::new(span, "Conflicting message specified"));
                                }
                            },
                            AnnotAttr::Replace(dty, drepl, span) => {
                                if arepl.is_none() {
                                    if let Some(elemty) = &elemty {
                                        if elemty != &dty {
                                            return Err(Error::new(dty.span(), "All replacement element types must be the same"));
                                        }
                                    } else {
                                        elemty = Some(dty.clone());
                                    }
                                    arepl = Some((dty, drepl));
                                } else {
                                    return Err(Error::new(span, "Conflicting replacement specified"));
                                }
                            },
                            AnnotAttr::Loc(dloc, span) => {
                                if aloc.is_none() {
                                    aloc = Some(dloc);
                                } else {
                                    return Err(Error::new(span, "Conflicting location reference specified"));
                                }
                            },
                        }
                    }

                    // Ensure we have at least a severity and loc
                    let asev: Severity = asev.ok_or_else(|| {
                        Error::new(l.path.span(), "Missing severity specifier (i.e., `error`, `help`, `suggestion` or `warn`/`warning`)")
                    })?;
                    let mut aloc: Expr = aloc
                        .ok_or_else(|| Error::new(l.path.span(), "Missing `loc`-specifier (i.e., `loc = <...>` or `loc` to imply `loc = loc`)"))?
                        .unwrap_or_else(|| {
                            if has_named_fields {
                                Expr::Path(ExprPath {
                                    attrs: Vec::new(),
                                    qself: None,
                                    path:  Path {
                                        leading_colon: None,
                                        segments:      {
                                            let mut segs = Punctuated::new();
                                            segs.push(PathSegment {
                                                ident:     Ident::new("loc", Span::mixed_site()),
                                                arguments: syn::PathArguments::None,
                                            });
                                            segs
                                        },
                                    },
                                })
                            } else {
                                Expr::Path(ExprPath {
                                    attrs: Vec::new(),
                                    qself: None,
                                    path:  Path {
                                        leading_colon: None,
                                        segments:      {
                                            let mut segs = Punctuated::new();
                                            segs.push(PathSegment {
                                                ident:     Ident::new("f0", Span::mixed_site()),
                                                arguments: syn::PathArguments::None,
                                            });
                                            segs
                                        },
                                    },
                                })
                            }
                        });

                    // Add the annotations
                    for expr in amsg.iter_mut().flat_map(Punctuated::iter_mut) {
                        syn::visit_mut::visit_expr_mut(&mut IdentVisitor, expr);
                    }
                    if let Some((_, arepl)) = &mut arepl {
                        syn::visit_mut::visit_expr_mut(&mut IdentVisitor, arepl);
                    }
                    syn::visit_mut::visit_expr_mut(&mut IdentVisitor, &mut aloc);
                    annots.push(Annot { sev: asev, msg: amsg, repl: arepl, loc: aloc });
                },

                // The rest is recognizably ours but wrong
                Meta::Path(p) if p.is_ident("diag") => {
                    return Err(Error::new(p.span(), "Cannot give plain `diag`, please follow it up with list arguments (i.e., `diag(...)`)"));
                },
                Meta::Path(p) if p.is_ident("annot") => {
                    return Err(Error::new(p.span(), "Cannot give plain `annot`, please follow it up with list arguments (i.e., `annot(...)`)"));
                },
                Meta::NameValue(nv) if nv.path.is_ident("diag") => {
                    return Err(Error::new(nv.path.span(), "Cannot give plain `diag`, please follow it up with list arguments (i.e., `diag(...)`)"));
                },
                Meta::NameValue(nv) if nv.path.is_ident("annot") => {
                    return Err(Error::new(
                        nv.path.span(),
                        "Cannot give plain `annot`, please follow it up with list arguments (i.e., `annot(...)`)",
                    ));
                },

                // We politely ignore other attributes
                _ => continue,
            }
        }

        // Unwrap the toplevel values
        let sev: Severity =
            sev.ok_or_else(|| Error::new(topspan, "Missing main severity specifier (i.e., `error`, `help`, `suggestion` or `warn`/`warning`)"))?;
        if let Some(loc) = loc {
            let loc = loc.unwrap_or_else(|| {
                if has_named_fields {
                    Expr::Path(ExprPath {
                        attrs: Vec::new(),
                        qself: None,
                        path:  Path {
                            leading_colon: None,
                            segments:      {
                                let mut segs = Punctuated::new();
                                segs.push(PathSegment { ident: Ident::new("loc", Span::mixed_site()), arguments: syn::PathArguments::None });
                                segs
                            },
                        },
                    })
                } else {
                    Expr::Path(ExprPath {
                        attrs: Vec::new(),
                        qself: None,
                        path:  Path {
                            leading_colon: None,
                            segments:      {
                                let mut segs = Punctuated::new();
                                segs.push(PathSegment { ident: Ident::new("f0", Span::mixed_site()), arguments: syn::PathArguments::None });
                                segs
                            },
                        },
                    })
                }
            });

            // Generate the implicit annotation here
            annots.insert(0, Annot { sev, msg: None, repl: None, loc });
        }

        // Build the final struct
        for expr in code.iter_mut().flat_map(Punctuated::iter_mut) {
            syn::visit_mut::visit_expr_mut(&mut IdentVisitor, expr);
        }
        if let Some(msg) = &mut msg {
            for expr in msg.iter_mut() {
                syn::visit_mut::visit_expr_mut(&mut IdentVisitor, expr);
            }
        }
        Ok(Diag { elemty, sev, code, msg, annots })
    }
}





/***** LIBRARY *****/
/// Main handler for the macro.
pub fn handle(item: TokenStream2) -> Result<TokenStream2, Error> {
    let DeriveInput { attrs, ident, data, mut generics, .. } = syn::parse2(item)?;
    match data {
        Data::Struct(s) => {
            // Parse toplevel attributes first
            let diag = Diag::parse_from_attrs(ident.span(), matches!(s.fields, Fields::Named(_)), &attrs)?;
            let inject_e: bool = diag.elemty.is_none();
            let elemty: Type = diag.elemty.clone().unwrap_or_else(default_elemty);

            // Generate the field expansion of the struct
            let fields: TokenStream2 = generate_field_expansion_struct(&s.fields);

            // Generate individual annotations
            let annots: Vec<TokenStream2> = diag.generate_annots(&elemty);

            // Then build
            let sev: Severity = diag.sev;
            let code: TokenStream2 = render_fstr(&diag.code);
            let msg_build = RenderMsgBuild::new(&ident, diag.msg.is_some());
            let msg = RenderMainMsg(&msg_build, &diag.msg);
            let to_inject: &[&str] = if inject_e { &["__E"] } else { &[] };
            inject_trait_bound_with_args(["ast_toolkit2", "diag", "Diagnostic"], to_inject, &mut generics);
            let (impl_gen, ty_gen, where_clauses) = split_for_impl_with_extra(to_inject, &generics);
            Ok(quote! {
                impl #impl_gen ::ast_toolkit2::diag::Diagnostic<#elemty> for #ident #ty_gen #where_clauses {
                    #[inline]
                    fn into_diag(self) -> ::ast_toolkit2::diag::Diag<#elemty> {
                        #msg_build
                        #fields;
                        ::ast_toolkit2::diag::Diag {
                            theme: ::ast_toolkit2::diag::Theme::PLAIN,

                            sev: #sev,
                            code: #code,
                            msg: #msg,

                            annots: ::std::vec![#(#annots),*],
                        }
                    }
                }
            })
        },
        Data::Enum(e) => {
            let diags = e
                .variants
                .iter()
                .map(|v| Diag::parse_from_attrs(v.ident.span(), matches!(v.fields, Fields::Named(_)), &v.attrs))
                .collect::<Result<Vec<Diag>, Error>>()?;
            let mut elemty: Option<Type> = None;
            for diag in &diags {
                // Parse toplevel attributes first
                if let Some(ty) = &diag.elemty {
                    if let Some(ety) = &elemty {
                        if ty != ety {
                            return Err(Error::new(ty.span(), "All replacement element types must be the same"));
                        }
                    } else {
                        elemty = Some(ty.clone());
                    }
                }
            }
            let inject_e: bool = elemty.is_none();
            let elemty: Type = elemty.unwrap_or_else(default_elemty);

            // Go through each variant
            let mut msg_build = RenderMsgBuild::new(&ident, true);
            let mut variants: Vec<TokenStream2> = Vec::with_capacity(e.variants.len());
            for (variant, diag) in e.variants.into_iter().zip(diags) {
                // Generate the variant expansion
                let match_arm: TokenStream2 = generate_field_expansion_variant(&variant.ident, &variant.fields);

                // Generate individual annotations
                let annots: Vec<TokenStream2> = diag.generate_annots(&elemty);

                // Then build the individual match arm
                let sev: Severity = diag.sev;
                let code: TokenStream2 = render_fstr(&diag.code);
                msg_build.1 &= diag.msg.is_some();
                let msg = RenderMainMsg(&msg_build, &diag.msg);
                variants.push(quote! {
                    #match_arm => ::ast_toolkit2::diag::Diag {
                        theme: ::ast_toolkit2::diag::Theme::PLAIN,

                        sev: #sev,
                        code: #code,
                        msg: #msg,

                        annots: ::std::vec![#(#annots),*],
                    },
                });
            }

            // Now build the overall thing
            let to_inject: &[&str] = if inject_e { &["__E"] } else { &[] };
            let (impl_gen, ty_gen, where_clauses) = split_for_impl_with_extra(to_inject, &generics);
            Ok(quote! {
                impl #impl_gen ::ast_toolkit2::diag::Diagnostic<#elemty> for #ident #ty_gen #where_clauses {
                    #[inline]
                    fn into_diag(self) -> ::ast_toolkit2::diag::Diag<#elemty> {
                        #msg_build
                        match self {
                            #(#variants)*
                        }
                    }
                }
            })
        },
        Data::Union(DataUnion { union_token, .. }) => Err(Error::new(union_token.span, "Can only derive `Term` on structs or enums")),
    }
}
