use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::meta::ParseNestedMeta;
use syn::LitStr;
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
                syn::Meta::Path(_) => Ok(Some(Self {
                    code: None,
                    conversion_strategy: None,
                })),
                syn::Meta::List(meta) => {
                    let mut code = None;
                    // If the meta contains exactly (_code = "...") take the string literal as the
                    // expression
                    // meta.parse_nested_meta(|meta| parse_code_hack(meta, &mut code))
                    //     .unwrap();
                    if meta
                        .parse_nested_meta(|meta| parse_code_hack(meta, &mut code))
                        .is_ok()
                    {
                        if let Some(code) = code {
                            Ok(Some(Self {
                                code: Some(code),
                                conversion_strategy: Some(ConversionStrategy::NoConversion),
                            }))
                        } else {
                            Err(Error::new_spanned(
                                meta,
                                "Expected single value in #[default(...)]",
                            ))
                        }
                    } else {
                        Ok(Some(Self {
                            code: Some(meta.tokens.clone()),
                            conversion_strategy: None,
                        }))
                    }
                }
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

fn parse_code_hack(meta: ParseNestedMeta, code: &mut Option<TokenStream>) -> Result<(), Error> {
    // panic!("{:?}", (meta.path, meta.input));
    if meta.path.is_ident("_code") {
        let str: LitStr = meta.value()?.parse()?;
        *code = Some(str.parse()?)
    }
    Ok(())
}
