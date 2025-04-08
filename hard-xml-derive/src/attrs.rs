use proc_macro2::Span;
use quote::ToTokens;
use crate::types::{FieldKind, StrictMode};
use crate::utils::Context;
use syn::{Attribute, Expr, Token};
use syn::Error;
use syn::Lit;
use syn::{LitStr, ExprPath};
use syn::Meta;
use syn::meta::ParseNestedMeta;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;

pub(crate) struct Container {
    pub(crate) tags: Vec<LitStr>,
    pub(crate) strict_mode: StrictMode,
}
impl Container {
    pub(crate) fn parse(ctx: &mut Context, attrs: Vec<Attribute>) -> Self {
        let mut tags = Vec::new();
        let mut strict_mode = StrictMode::empty();

        for meta in attrs.iter().filter_map(get_xml_meta).flatten() {
            match meta {
                Meta::NameValue(m) if m.path.is_ident("tag") => {
                    if let Expr::Lit(expr) = &m.value {
                        if let Lit::Str(lit) = &expr.lit {
                            tags.push(lit.clone());
                        }
                    } else {
                        ctx.push_spanned_error(m.value, "expected a string literal");
                    }
                },
                Meta::List(m) if m.path.is_ident("strict") => {
                    let _ = m.parse_nested_meta(|nested| {
                        if nested.path.is_ident("unknown_attribute") {
                            strict_mode |= StrictMode::UNKNOWN_ATTRIBUTE;
                        } else if nested.path.is_ident("unknown_element") {
                            strict_mode |= StrictMode::UNKNOWN_ELEMENT;
                        } else {
                            ctx.push_spanned_error(
                                nested.path,
                                "unsupported argument to `strict`",
                            );
                        }
                        Ok(())
                    });
                }
                _ => {}
            }
        }

        Self { tags, strict_mode }
    }
}

pub (crate) enum Prefix {
    Prefix,
    Startswith
}

pub(crate) struct Field {
    pub(crate) default: bool,
    pub(crate) attr_tag: Option<LitStr>,
    pub(crate) child_tags: Vec<LitStr>,
    pub(crate) is_text: bool,
    pub(crate) flatten_text_tag: Option<LitStr>,
    pub(crate) is_cdata: bool,
    pub(crate) prefix: Option<Prefix>,
    pub(crate) with: Option<ExprPath>,
}

impl Field {
    pub(crate) fn parse(context: &mut Context, attrs: Vec<Attribute>) -> Self {
        let mut default = false;
        let mut attr_tag = None;
        let mut child_tags = Vec::new();
        let mut is_text = false;
        let mut flatten_text_tag = None;
        let mut is_cdata = false;
        let mut prefix = None;
        let mut with = None;


        // TODO can this be handled more cleanly?
        for meta in attrs.iter().filter_map(get_xml_meta).flatten() {
            match meta {
                Meta::NameValue(p) if p.path.is_ident("default") => {
                    if default {
                        context.push(Error::new_spanned(p, "duplicate `default` attribute"));
                    } else {
                        default = true;
                    }
                }
                Meta::NameValue(m) if m.path.is_ident("attr") => {
                    if let Expr::Lit(lit) = &m.value {
                        if attr_tag.is_some() {
                            context.push(Error::new_spanned(&m.path, "duplicate `attr` attribute"));
                        } else if is_text {
                            context.push(Error::new_spanned(
                                &m.path,
                                "`attr` attribute and `text` attribute is disjoint",
                            ));
                        } else if is_cdata {
                            context.push(Error::new_spanned(
                                &m.path,
                                "`attr` attribute and `cdata` attribute is disjoint",
                            ))
                        } else if !child_tags.is_empty() {
                            context.push(Error::new_spanned(
                                &m.path,
                                "`attr` attribute and `child` attribute is disjoint",
                            ));
                        } else if flatten_text_tag.is_some() {
                            context.push(Error::new_spanned(
                                &m.path,
                                "`attr` attribute and `flatten_text` attribute is disjoint",
                            ));
                        } else if let Lit::Str(lit) = &lit.lit {
                            attr_tag = Some(lit.clone());
                        }
                    } else {
                        context.push(Error::new_spanned(&m.value, "expected a string literal"));
                    }
                }
                Meta::NameValue(m) if m.path.is_ident("prefix") => {
                    if let Expr::Lit(lit) = &m.value {
                        if attr_tag.is_some() {
                            context.push(Error::new_spanned(&m.path, "duplicate `prefix` attribute"));
                        } else if is_text {
                            context.push(Error::new_spanned(
                                &m.path,
                                "`prefix` attribute and `text` attribute is disjoint",
                            ));
                        } else if is_cdata {
                            context.push(Error::new_spanned(
                                &m.path,
                                "`prefix` attribute and `cdata` attribute is disjoint",
                            ))
                        } else if !child_tags.is_empty() {
                            context.push(Error::new_spanned(
                                &m.path,
                                "`prefix` attribute and `child` attribute is disjoint",
                            ));
                        } else if flatten_text_tag.is_some() {
                            context.push(Error::new_spanned(
                                &m.path,
                                "`prefix` attribute and `flatten_text` attribute is disjoint",
                            ));
                        } else if let Lit::Str(lit) = &lit.lit {
                            attr_tag = Some(lit.clone());
                            prefix = Some(Prefix::Prefix);
                        }
                    } else {
                        context.push(Error::new_spanned(&m.value, "expected a string literal"));
                    }
                }
                Meta::NameValue(m) if m.path.is_ident("startswith") => {
                    if let Expr::Lit(lit) = &m.value {
                        if attr_tag.is_some() {
                            context.push(Error::new_spanned(&m.path, "duplicate `startswith` attribute"));
                        } else if is_text {
                            context.push(Error::new_spanned(
                                &m.path,
                                "`startswith` attribute and `text` attribute is disjoint",
                            ));
                        } else if is_cdata {
                            context.push(Error::new_spanned(
                                &m.path,
                                "`startswith` attribute and `cdata` attribute is disjoint",
                            ))
                        } else if !child_tags.is_empty() {
                            context.push(Error::new_spanned(
                                &m.path,
                                "`startswith` attribute and `child` attribute is disjoint",
                            ));
                        } else if flatten_text_tag.is_some() {
                            context.push(Error::new_spanned(
                                &m.path,
                                "`startswith` attribute and `flatten_text` attribute is disjoint",
                            ));
                        } else if let Lit::Str(lit) = &lit.lit{
                            attr_tag = Some(lit.clone());
                            prefix = Some(Prefix::Startswith);
                        }
                    } else {
                        context.push(Error::new_spanned(&m.value, "expected a string literal"));
                    }
                }
                Meta::Path(ref p) if p.is_ident("text") => {
                    if is_text {
                        context.push(Error::new_spanned(p, "Duplicate `text` attribute."));
                    } else if attr_tag.is_some() {
                        context.push(Error::new_spanned(
                            p,
                            "`text` attribute and `attr` attribute is disjoint.",
                        ));
                    } else if !child_tags.is_empty() {
                        context.push(Error::new_spanned(
                            p,
                            "`text` attribute and `child` attribute is disjoint.",
                        ));
                    } else if flatten_text_tag.is_some() {
                        context.push(Error::new_spanned(
                            p,
                            "`text` attribute and `flatten_text` attribute is disjoint.",
                        ));
                    } else {
                        is_text = true;
                    }
                }
                Meta::Path(ref p) if p.is_ident("cdata") => {
                    if is_cdata {
                        context.push(Error::new_spanned(p, "Duplicate `cdata` attribute."));
                    } else if attr_tag.is_some() {
                        context.push(Error::new_spanned(
                            p,
                            "`text` attribute and `attr` attribute is disjoint.",
                        ));
                    } else if !child_tags.is_empty() {
                        context.push(Error::new_spanned(
                            p,
                            "`text` attribute and `child` attribute is disjoint.",
                        ));
                    } else {
                        is_cdata = true;
                    }
                }
                Meta::NameValue(m) if m.path.is_ident("child") => {
                    if let Expr::Lit(lit) = &m.value {
                        if is_text {
                            context.push(Error::new_spanned(
                                &m.path,
                                "`child` attribute and `text` attribute is disjoint.",
                            ));
                        } else if attr_tag.is_some() {
                            context.push(Error::new_spanned(
                                &m.path,
                                "`child` attribute and `attr` attribute is disjoint.",
                            ));
                        } else if is_cdata {
                            context.push(Error::new_spanned(
                                &m.path,
                                "`child` attribute and `cdata` attribute is disjoint.",
                            ))
                        } else if flatten_text_tag.is_some() {
                            context.push(Error::new_spanned(
                                &m.path,
                                "`child` attribute and `flatten_text` attribute is disjoint.",
                            ));
                        } else if let Lit::Str(lit) = &lit.lit{
                            child_tags.push(lit.clone());
                        }
                    } else {
                        context.push(Error::new_spanned(&m.value, "Expected a string literal."));
                    }
                }
                Meta::NameValue(m) if m.path.is_ident("flatten_text") => {
                    if let Expr::Lit(lit) = &m.value {
                        if is_text {
                            context.push(Error::new_spanned(
                                &m.path,
                                "`flatten_text` attribute and `text` attribute is disjoint.",
                            ));
                        } else if !child_tags.is_empty() {
                            context.push(Error::new_spanned(
                                &m.path,
                                "`flatten_text` attribute and `child` attribute is disjoint.",
                            ));
                        } else if attr_tag.is_some() {
                            context.push(Error::new_spanned(
                                &m.path,
                                "`flatten_text` attribute and `attr` attribute is disjoint.",
                            ));
                        } else if flatten_text_tag.is_some() {
                            context.push(Error::new_spanned(
                                &m.path,
                                "Duplicate `flatten_text` attribute.",
                            ));
                        } else if let Lit::Str(lit) = &lit.lit{
                            flatten_text_tag = Some(lit.clone());
                        }
                    } else {
                        context.push(Error::new_spanned(&m.value, "Expected a string literal."));
                    }
                }
                Meta::NameValue(m) if m.path.is_ident("with") => {
                    if let Expr::Lit(lit) = &m.value {
                        if let Lit::Str(lit) = &lit.lit {
                            with = Some(lit.parse().unwrap())
                        }
                    } else {
                        context.push(Error::new_spanned(&m.value, "Expected a string literal."));
                    }
                },
                _ => (),
            }
        }

        Self {
            default,
            attr_tag,
            child_tags,
            is_text,
            flatten_text_tag,
            is_cdata,
            prefix,
            with,
        }
    }
}

pub(crate) fn get_xml_meta(attr: &Attribute) -> Option<impl Iterator<Item = Meta>> {
    if attr.path().is_ident("xml") {
        Some(attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated).unwrap().into_iter())
    } else {
        None
    }
}
