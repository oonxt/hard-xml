use proc_macro2::TokenStream;
use quote::quote;
use syn::{ExprPath, Ident, LitStr};

use crate::types::{Field, Type};

pub fn write(tag: &LitStr, ele_name: TokenStream, fields: &[Field]) -> TokenStream {
    let write_attributes = fields.iter().filter_map(|field| match field {
        Field::Attribute { tag, bind, ty, with, .. } => Some(write_attrs(tag, bind, ty, with, &ele_name)),
        _ => None,
    });

    let write_text = fields.iter().filter_map(|field| match field {
        Field::Text {
            bind, ty, with, is_cdata, ..
        } => Some(write_text(tag, bind, ty, with, &ele_name, *is_cdata)),
        _ => None,
    });

    let write_flatten_text = fields.iter().filter_map(|field| match field {
        Field::FlattenText {
            tag,
            bind,
            ty,
            with,
            is_cdata,
            ..
        } => Some(write_flatten_text(tag, bind, ty, with, &ele_name, *is_cdata)),
        _ => None,
    });

    let write_child = fields.iter().filter_map(|field| match field {
        Field::Child { bind, ty, .. } => Some(write_child(bind, ty, &ele_name)),
        _ => None,
    });

    let write_maps = fields.iter().filter_map(|field| match field {
        Field::Prefix { tag, bind, ty, .. } => Some(write_prefix(tag, bind, ty, &ele_name)),
        Field::Startswith { tag, bind, ty, .. } => Some(write_starts(tag, bind, ty, &ele_name)),
        _ => None,
    });

    let is_leaf_element = fields
        .iter()
        .all(|field| matches!(field, Field::Attribute { .. }));

    let is_text_element = fields
        .iter()
        .any(|field| matches!(field, Field::Text { .. }));

    let can_self_close = fields.iter().all(|field| match field {
        Field::Child { ty, .. } | Field::FlattenText { ty, .. } => ty.is_vec() || ty.is_option(),
        _ => true,
    });

    let content_is_empty = fields.iter().filter_map(|field| match field {
        Field::Child { ty, bind, .. } | Field::FlattenText { ty, bind, .. } => {
            if ty.is_vec() {
                Some(quote! { #bind.is_empty() })
            } else if ty.is_option() {
                Some(quote! { #bind.is_none() })
            } else {
                None
            }
        }
        _ => None,
    });

    let write_element_end = if is_leaf_element {
        quote! { writer.write_element_end_empty()?; }
    } else if is_text_element {
        quote! { #( #write_text )* }
    } else {
        quote! {
            if #can_self_close #( && #content_is_empty )* {
                writer.write_element_end_empty()?;
            } else {
                writer.write_element_end_open()?;
                #( #write_child )*
                #( #write_flatten_text )*
                writer.write_element_end_close(#tag)?;
            }
        }
    };

    quote! {
        hard_xml::log_start_writing!(#ele_name);

        writer.write_element_start(#tag)?;

        #( #write_attributes )*

        #( #write_maps )*

        #write_element_end

        hard_xml::log_finish_writing!(#ele_name);
    }
}

fn write_attrs(tag: &LitStr, name: &Ident, ty: &Type, with: &Option<ExprPath>, ele_name: &TokenStream) -> TokenStream {
    let to_str = to_str(ty, with, true);

    if ty.is_vec() {
        panic!("`attr` attribute doesn't support Vec.");
    } else if ty.is_option() {
        quote! {
            hard_xml::log_start_writing_field!(#ele_name, #name);

            if let Some(__value) = #name {
                writer.write_attribute(#tag, #to_str)?;
            }

            hard_xml::log_finish_writing_field!(#ele_name, #name);
        }
    } else {
        quote! {
            hard_xml::log_start_writing_field!(#ele_name, #name);

            let __value = #name;
            writer.write_attribute(#tag, #to_str)?;

            hard_xml::log_finish_writing_field!(#ele_name, #name);
        }
    }
}

fn write_child(name: &Ident, ty: &Type, ele_name: &TokenStream) -> TokenStream {
    match ty {
        Type::OptionT(_) => quote! {
            hard_xml::log_start_writing_field!(#ele_name, #name);

            if let Some(ref ele) = #name {
                ele.to_writer(&mut writer)?;
            }

            hard_xml::log_finish_writing_field!(#ele_name, #name);
        },
        Type::VecT(_) => quote! {
            hard_xml::log_start_writing_field!(#ele_name, #name);

            for ele in #name {
                ele.to_writer(&mut writer)?;
            }

            hard_xml::log_finish_writing_field!(#ele_name, #name);
        },
        Type::T(_) => quote! {
            hard_xml::log_start_writing_field!(#ele_name, #name);

            #name.to_writer(&mut writer)?;

            hard_xml::log_finish_writing_field!(#ele_name, #name);
        },
        _ => panic!("`child` attribute only supports Vec<T>, Option<T> and T."),
    }
}

fn write_text(
    tag: &LitStr,
    name: &Ident,
    ty: &Type,
    with: &Option<ExprPath>,
    ele_name: &TokenStream,
    is_cdata: bool,
) -> TokenStream {
    let to_str = to_str(ty, with, false);
    let write_fn = if is_cdata {
        quote!(write_cdata_text)
    } else {
        quote!(write_text)
    };

    quote! {
        writer.write_element_end_open()?;

        hard_xml::log_start_writing_field!(#ele_name, #name);

        let __value = &#name;

        writer.#write_fn(#to_str)?;

        hard_xml::log_finish_writing_field!(#ele_name, #name);

        writer.write_element_end_close(#tag)?;
    }
}

fn write_flatten_text(
    tag: &LitStr,
    name: &Ident,
    ty: &Type,
    with: &Option<ExprPath>,
    ele_name: &TokenStream,
    is_cdata: bool,
) -> TokenStream {
    let to_str = to_str(ty, with, false);

    if ty.is_vec() {
        quote! {
            hard_xml::log_finish_writing_field!(#ele_name, #name);

            for __value in #name {
                writer.write_flatten_text(#tag, #to_str, #is_cdata)?;
            }

            hard_xml::log_finish_writing_field!(#ele_name, #name);
        }
    } else if ty.is_option() {
        quote! {
            hard_xml::log_finish_writing_field!(#ele_name, #name);

            if let Some(__value) = #name {
                writer.write_flatten_text(#tag, #to_str, #is_cdata)?;
            }

            hard_xml::log_finish_writing_field!(#ele_name, #name);
        }
    } else {
        quote! {
            hard_xml::log_finish_writing_field!(#ele_name, #name);

            let __value = &#name;
            writer.write_flatten_text(#tag, #to_str, #is_cdata)?;

            hard_xml::log_finish_writing_field!(#ele_name, #name);
        }
    }
}


fn write_prefix(tag: &LitStr, name: &Ident, ty: &Type, ele_name: &TokenStream) -> TokenStream {
    if !ty.is_map() {
        panic!("`prefix` attribute only support Map.");
    } else if ty.is_option() {
        quote! {
            hard_xml::log_start_writing_field!(#ele_name, #name);

            if let Some(__value) = #name {
                for (k, v) in __value {
                    writer.write_attribute(&format!("{}:{}", #tag, k), &v.to_string())?;
                }
            }

            hard_xml::log_finish_writing_field!(#ele_name, #name);
        }
    } else {
        quote! {
            hard_xml::log_start_writing_field!(#ele_name, #name);

            let __value = #name;
            for (k, v) in __value {
                writer.write_attribute(&format!("{}:{}", #tag, k), &v.to_string())?;
            }

            hard_xml::log_finish_writing_field!(#ele_name, #name);
        }
    }
}

fn write_starts(tag: &LitStr, name: &Ident, ty: &Type, ele_name: &TokenStream) -> TokenStream {
    if !ty.is_map() {
        panic!("`startswith` attribute only support Map.");
    } else if ty.is_option() {
        quote! {
            hard_xml::log_start_writing_field!(#ele_name, #name);

            if let Some(__value) = #name {
                for (k, v) in __value {
                    if k.is_empty() {
                        writer.write_attribute(&format!("{}", #tag), &v.to_string())?;
                    } else {
                        writer.write_attribute(&format!("{}:{}", #tag, k), &v.to_string())?;
                    }
                }
            }

            hard_xml::log_finish_writing_field!(#ele_name, #name);
        }
    } else {
        quote! {
            hard_xml::log_start_writing_field!(#ele_name, #name);

            let __value = #name;
            for (k, v) in __value {
                if k.is_empty() {
                    writer.write_attribute(&format!("{}", #tag), &v.to_string())?;
                } else {
                    writer.write_attribute(&format!("{}:{}", #tag, k), &v.to_string())?;
                }
            }

            hard_xml::log_finish_writing_field!(#ele_name, #name);
        }
    }
}

fn to_str(ty: &Type, with: &Option<ExprPath>, convert: bool) -> TokenStream {
    if let Some(with_mod) = with {
        return quote! {
            {
                let r: hard_xml::XmlResult<_> = #with_mod::to_xml(&__value);
                std::convert::AsRef::<str>::as_ref(&r?)
            }
        };
    }

    match &ty {
        Type::CowStr | Type::OptionCowStr | Type::VecCowStr => {
            quote! { __value }
        }
        Type::Bool | Type::OptionBool | Type::VecBool => if convert {
            quote! {
                match __value {
                    true => "1",
                    false => "0",
                }
            }
        } else {
            quote! {
                match __value {
                    true => "true",
                    false => "false",
                }
            }
        },
        Type::T(_) | Type::OptionT(_) | Type::VecT(_) => {
            quote! { &format!("{}", __value) }
        }
        Type::Map(_, _) | Type::OptionMap(_, _) | Type::VecTuple(_, _) | Type::OptionVecTuple(_, _) => {
            quote! { &format!("{}", __value)}
        }
    }
}
