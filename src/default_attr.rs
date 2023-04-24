use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{parse::Error, MetaNameValue};

use crate::util::find_only;

#[derive(Debug, Clone, Copy)]
pub enum ConversionStrategy {
    NoConversion,
    Into,
}

pub struct DefaultAttr {
    pub code: Option<TokenStream>,
    conversion_strategy: Option<ConversionStrategy>,
}

impl DefaultAttr {
    pub fn find_in_attributes(attrs: &[syn::Attribute]) -> Result<Option<Self>, Error> {
        if let Some(default_attr) =
            find_only(attrs.iter(), |attr| Ok(attr.path().is_ident("default")))?
        {
            match &default_attr.meta {
                // #[default]
                syn::Meta::Path(_) => Ok(Some(Self {
                    code: None,
                    conversion_strategy: None,
                })),
                syn::Meta::List(meta) => {
                    // #[default(_code = "<expr>")]
                    if let Some(field_value) = parse_code_hack(&meta)? {
                        Ok(Some(Self {
                            code: Some(field_value.into_token_stream()),
                            conversion_strategy: Some(ConversionStrategy::NoConversion),
                        }))
                    // #[default(<expr>)]
                    } else if let Ok(field_value) = syn::parse2::<syn::Expr>(meta.tokens.clone()) {
                        Ok(Some(Self {
                            code: Some(field_value.into_token_stream()),
                            conversion_strategy: None,
                        }))
                    } else {
                        Err(Error::new_spanned(
                            meta,
                            "Expected single value in #[default(...)]",
                        ))
                    }
                }
                // #[default = <expr>]
                syn::Meta::NameValue(MetaNameValue { value, .. }) => Ok(Some(Self {
                    code: Some(value.into_token_stream()),
                    conversion_strategy: None,
                })),
            }
        } else {
            Ok(None)
        }
    }

    pub fn conversion_strategy(&self) -> ConversionStrategy {
        if let Some(conversion_strategy) = self.conversion_strategy {
            // Conversion strategy already set
            return conversion_strategy;
        }
        let code = if let Some(code) = &self.code {
            code
        } else {
            // #[default] - so no conversion (`Default::default()` already has the correct type)
            return ConversionStrategy::NoConversion;
        };
        match syn::parse::<syn::Lit>(code.clone().into()) {
            Ok(syn::Lit::Str(_)) | Ok(syn::Lit::ByteStr(_)) => {
                // A string literal - so we need a conversion in case we need to make it a `String`
                return ConversionStrategy::Into;
            }
            _ => {}
        }
        // Not handled by one of the rules, so we don't convert it to avoid causing trouble
        ConversionStrategy::NoConversion
    }
}

fn parse_code_hack(meta: &syn::MetaList) -> Result<Option<TokenStream>, Error> {
    if let Ok(inner) = syn::parse2::<MetaNameValue>(meta.tokens.clone()) {
        if inner.path.is_ident("_code") {
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(lit),
                ..
            }) = &inner.value
            {
                use std::str::FromStr;
                return Ok(Some(TokenStream::from_str(&lit.value())?));
            } else {
                return Ok(Some(inner.value.to_token_stream()));
            }
        }
    }
    Ok(None)
}
