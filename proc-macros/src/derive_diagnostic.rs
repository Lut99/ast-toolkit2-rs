//  DERIVE DIAGNOSTIC.rs
//    by Lut99
//
//  Description:
//!   Implements the derive macro for `Diagnostic`.
//

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream, Parser as _};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Paren;
use syn::visit_mut::VisitMut;
use syn::{
    Attribute, Data, DataUnion, DeriveInput, Error, Expr, ExprLit, ExprPath, Fields, Ident, Lit, LitStr, Meta, Path, PathSegment, Token,
    parenthesized,
};


/***** HELPER FUNCTIONS *****/
/// Generates the field expansion (i.e., `let Self { ... } = self`) of a set of [`Fields`].
fn generate_field_expansion(fields: &Fields) -> TokenStream2 {
    let mut has_names: bool = false;
    let fields: Vec<Ident> = fields
        .iter()
        .enumerate()
        .map(|(i, f)| {
            if let Some(ident) = &f.ident {
                has_names = true;
                Ident::new(&ident.to_string(), Span::mixed_site())
            } else {
                Ident::new(&format!("field{i}"), Span::mixed_site())
            }
        })
        .collect();
    if has_names {
        quote! { let Self { #(#fields),* } = self; }
    } else {
        quote! { let Self ( #(#fields),* ) = self; }
    }
}

/// Parses a format expression (either a literal or multiple expressions wrapped in `()`).
fn parse_fmt_exprs(input: ParseStream) -> Result<Punctuated<Expr, Token![,]>, Error> {
    fn parenthesized(input: ParseStream) -> Result<(Paren, Punctuated<Expr, Token![,]>), Error> {
        let content;
        let parens = parenthesized!(content in input);
        let expr: Punctuated<Expr, Token![,]> = Punctuated::parse_terminated(&content)?;
        if !content.is_empty() {
            return Err(content.error("Unexpected tokens after expression"));
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
fn parse_loc_expr(input: ParseStream) -> Result<Expr, Error> {
    if input.parse::<Token![=]>().is_ok() {
        Ok(input.parse()?)
    } else if input.is_empty() {
        Ok(Expr::Path(ExprPath {
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
        }))
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

/// Takes an optional format string, then renders it
fn render_fstr_as_string(exprs: &Option<Punctuated<Expr, Token![,]>>) -> TokenStream2 {
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
    Loc(Expr, Span),
}
impl Parse for DiagAttr {
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

/// Defines possible `#[annot(...)]`-attributes.
enum AnnotAttr {
    /// Defines the severity of an attribute row.
    Severity(Severity, Span),
    /// Defines a message
    Message(Punctuated<Expr, Token![,]>, Span),
    /// Defines a replacement
    Replace(Punctuated<Expr, Token![,]>, Span),
    /// Defines an annotation expression
    Loc(Expr, Span),
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
            input.parse::<Token![=]>()?;
            Ok(Self::Replace(parse_fmt_exprs(input)?, ident.span()))
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
    repl: Option<Punctuated<Expr, Token![,]>>,
    /// The location expression
    loc:  Expr,
}

/// Defines what we expect to learn from toplevel (struct) or variant (enum) attributes.
struct Diag {
    /// The main severity
    sev:    Severity,
    /// The main message
    msg:    Punctuated<Expr, Token![,]>,
    /// The main code
    code:   Option<Punctuated<Expr, Token![,]>>,
    /// A list of annotations
    annots: Vec<Annot>,
}
impl Diag {
    /// Generates a list tokens representing the constructions of each annotation.
    fn generate_annots(&self) -> Vec<TokenStream2> {
        self.annots
            .iter()
            .map(|a| {
                let asev: Severity = a.sev;
                let amsg: TokenStream2 = render_fstr_as_string(&a.msg);
                let arepl: TokenStream2 = render_fstr_as_string(&a.repl);
                let aloc: &Expr = &a.loc;
                quote! {
                    ::ast_toolkit2::diag::Annotation {
                        repl: #arepl,
                        msg: #amsg,
                        sev: #asev,
                        loc: #aloc,
                    }
                }
            })
            .collect()
    }

    /// Parses the VariantAttrs from a given list of attributes.
    fn parse_from_attrs(topspan: Span, attrs: &[Attribute]) -> Result<Self, Error> {
        let mut sev: Option<Severity> = None;
        let mut code: Option<Punctuated<Expr, Token![,]>> = None;
        let mut msg: Option<Punctuated<Expr, Token![,]>> = None;
        let mut loc: Option<Expr> = None;
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
                    let mut arepl: Option<Punctuated<Expr, Token![,]>> = None;
                    let mut aloc: Option<Expr> = None;

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
                            AnnotAttr::Replace(drepl, span) => {
                                if arepl.is_none() {
                                    arepl = Some(drepl);
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
                    let mut aloc: Expr =
                        aloc.ok_or_else(|| Error::new(l.path.span(), "Missing `loc`-specifier (i.e., `loc = <...>` or `loc` to imply `loc = loc`)"))?;

                    // Add the annotations
                    for expr in amsg.iter_mut().flat_map(Punctuated::iter_mut) {
                        syn::visit_mut::visit_expr_mut(&mut IdentVisitor, expr);
                    }
                    for expr in arepl.iter_mut().flat_map(Punctuated::iter_mut) {
                        syn::visit_mut::visit_expr_mut(&mut IdentVisitor, expr);
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
        let mut msg: Punctuated<Expr, Token![,]> = msg.ok_or_else(|| Error::new(topspan, "Missing main message"))?;
        if let Some(loc) = loc {
            // Generate the implicit annotation here
            annots.insert(0, Annot { sev, msg: None, repl: None, loc });
        }

        // Build the final struct
        for expr in code.iter_mut().flat_map(Punctuated::iter_mut) {
            syn::visit_mut::visit_expr_mut(&mut IdentVisitor, expr);
        }
        for expr in msg.iter_mut() {
            syn::visit_mut::visit_expr_mut(&mut IdentVisitor, expr);
        }
        Ok(Diag { sev, code, msg, annots })
    }
}





/***** LIBRARY *****/
/// Main handler for the macro.
pub fn handle(item: TokenStream2) -> Result<TokenStream2, Error> {
    let DeriveInput { attrs, ident, data, generics, .. } = syn::parse2(item)?;
    match data {
        Data::Struct(s) => {
            // Parse toplevel attributes first
            let diag = Diag::parse_from_attrs(ident.span(), &attrs)?;

            // Generate the field expansion of the struct
            let fields: TokenStream2 = generate_field_expansion(&s.fields);

            // Generate individual annotations
            let annots: Vec<TokenStream2> = diag.generate_annots();

            // Then build
            let sev: Severity = diag.sev;
            let code: TokenStream2 = render_fstr_as_string(&diag.code);
            let msg: &Punctuated<Expr, Token![,]> = &diag.msg;
            let (impl_gen, ty_gen, where_clauses) = generics.split_for_impl();
            Ok(quote! {
                impl #impl_gen ::ast_toolkit2::diag::Diagnostic for #ident #ty_gen #where_clauses {
                    #[inline]
                    fn into_diag(self) -> ::ast_toolkit2::diag::Diag {
                        #fields;
                        ::ast_toolkit2::diag::Diag {
                            theme: ::ast_toolkit2::diag::Theme::PLAIN,

                            sev: #sev,
                            code: #code,
                            msg: ::std::format!(#msg),

                            annots: ::std::vec![#(#annots),*],
                        }
                    }
                }
            })
        },
        Data::Enum(e) => {
            let (impl_gen, ty_gen, where_clauses) = generics.split_for_impl();
            Ok(quote! {
                impl #impl_gen ::ast_toolkit2::diag::Diagnostic for #ident #ty_gen #where_clauses {
                    #[inline]
                    fn into_diag(self) -> ::ast_toolkit2::diag::Diag {
                        ::std::todo!();
                    }
                }
            })
        },
        Data::Union(DataUnion { union_token, .. }) => Err(Error::new(union_token.span, "Can only derive `Term` on structs or enums")),
    }
}
