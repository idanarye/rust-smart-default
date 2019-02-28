use proc_macro2::TokenStream;
use syn::parse::Error;
use syn::spanned::Spanned;
use quote::ToTokens;

use util::{find_only, single_value};

pub fn find_default_attr_value(attrs: &[syn::Attribute]) -> Result<Option<Option<TokenStream>>, Error> {
    if let Some(default_attr) = find_only(attrs.iter(), |attr| is_default_attr(attr))? {
        match default_attr.parse_meta() {
            Ok(syn::Meta::Word(_)) => Ok(Some(None)),
            Ok(syn::Meta::List(meta)) => {
                if let Some (field_value) = parse_code_hack(&meta)? { // #[default(_code = "...")]
                    Ok(Some(Some(field_value.into_token_stream())))
                } else if let Some(field_value) = single_value(meta.nested.iter()) { // #[default(...)]
                    Ok(Some(Some(field_value.into_token_stream())))
                } else {
                    return Err(Error::new(
                            if meta.nested.is_empty() {
                                meta.span()
                            } else {
                                meta.nested.span()
                            },
                            "Expected signle value in #[default(...)]"));
                }
            }
            Ok(syn::Meta::NameValue(meta)) => {
                Ok(Some(Some(meta.lit.into_token_stream())))
            }
            Err(error) => {
                if let syn::Expr::Paren(as_parens) = syn::parse(default_attr.tts.clone().into())? {
                    Ok(Some(Some(as_parens.expr.into_token_stream())))
                } else {
                    Err(error)
                }
            }
        }
    } else {
        Ok(None)
    }
}

fn is_default_attr(attr: &syn::Attribute) -> Result<bool, Error> {
    let path = &attr.path;
    if path.leading_colon.is_some() {
        return Ok(false);
    }
    let segment = if let Some(segment) = single_value(path.segments.iter()) {
        segment
    } else {
        return Ok(false);
    };

    if let syn::PathArguments::None = segment.arguments {
    } else {
        return Ok(false);
    }

    Ok(segment.ident.to_string() == "default")
}

fn parse_code_hack(meta: &syn::MetaList) -> Result<Option<TokenStream>, Error> {
    for meta in meta.nested.iter() {
        if let syn::NestedMeta::Meta(syn::Meta::NameValue(meta)) = meta {
            if meta.ident != "_code" {
                continue;
            }
            if let syn::Lit::Str(lit) = &meta.lit {
                use std::str::FromStr;
                return Ok(Some(TokenStream::from_str(&lit.value())?));
            }
        };
    }
    Ok(None)
}
