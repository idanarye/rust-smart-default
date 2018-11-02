//! # Smart Default
//!
//! This crate provides a custom derive for `SmartDefault`. `SmartDefault` is not a real type -
//! deriving it will actually `impl Default`. The difference from regular `#[derive(Default)]` is
//! that `#[derive(SmartDefault)]` allows you to use `#[default = "..."]` attributes to customize
//! the `::default()` method and to support `struct`s that don't have `Default` for all their
//! fields - and even `enum`s!
//!
//! # Examples
//!
//! ```
//! #[macro_use]
//! extern crate smart_default;
//!
//! # fn main() {
//! #[derive(SmartDefault)]
//! # #[derive(PartialEq)]
//! # #[allow(dead_code)]
//! enum Foo {
//!     Bar,
//!     #[default]
//!     Baz {
//!         #[default = 12]
//!         a: i32,
//!         b: i32,
//!         #[default(Some(Default::default()))]
//!         c: Option<i32>
//!     },
//!     Qux(i32),
//! }
//!
//! assert!(Foo::default() == Foo::Baz { a: 12, b: 0, c: Some(0) });
//! # }
//! ```
//!
//! * `Baz` has the `#[default]` attribute. This means that the default `Foo` is a `Foo::Baz`. Only
//!   one variant may have a `#[default]` attribute, and that attribute must have no value.
//! * `a` has a `#[default = 12]` attribute. This means that it's default value is `12`.
//! * `b` has no `#[default = ...]` attribute. It's default value will `i32`'s default value
//!   instead - `0`.
//! * `c` is an `Option<i32>`, and it's default is `Some(Default::default())`. Rust cannot (currently)
//!   parse `#[default = Some(Default::default())]` and therefore we have to use a special syntax:
//!   `#[default(Some(Default::default))]`
//! * Documentation for the `impl Default` section is generated automatically, specifying the
//!   default value returned from `::default()`.

extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;

extern crate quote;

use proc_macro2::TokenStream;

use syn::{
    parse_macro_input,
    DeriveInput,
};
use syn::spanned::Spanned;
use syn::parse::Error;
use quote::{quote, ToTokens};

#[doc(hidden)]
#[proc_macro_derive(SmartDefault, attributes(default))]
pub fn derive_smart_default(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match impl_my_derive(&input) {
        Ok(output) => {
            output.into()
        },
        Err(error) =>{
            error.to_compile_error().into()
        }
    }
}

fn impl_my_derive(input: &DeriveInput) -> Result<TokenStream, Error> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let (default_expr, doc) = match input.data {
        syn::Data::Struct(ref body) => {
            let (body_assignment, doc) = default_body_tt(&body.fields)?;
            (quote! {
                #name #body_assignment
            }, format!("Return `{}{}`", name, doc))
        }
        syn::Data::Enum(ref body) => {
            let default_variant = find_only(body.variants.iter(), |variant| {
                if let Some(meta) = find_default_attr_value(&variant.attrs)? {
                    if meta.is_none() {
                        Ok(true)
                    } else {
                        Err(Error::new(meta.span(), "Attribute #[default] on variants should have no value"))
                    }
                } else {
                    Ok(false)
                }
            })?.ok_or_else(|| Error::new(input.span(), "No default variant"))?;
            let default_variant_name = &default_variant.ident;
            let (body_assignment, doc) = default_body_tt(&default_variant.fields)?;
            (quote! {
                #name :: #default_variant_name #body_assignment
            }, format!("Return `{}::{}{}`", name, default_variant_name, doc))
        }
        syn::Data::Union(_) => {
            panic!()
        }
    };
    Ok(quote! {
        impl #impl_generics Default for #name #ty_generics #where_clause {
            #[doc = #doc]
            fn default() -> Self {
                #default_expr
            }
        }
    })
}

/// Return a token-tree for the default "body" - the part after the name that contains the values.
/// That is, the `{ ... }` part for structs, the `(...)` part for tuples, and nothing for units.
fn default_body_tt(body: &syn::Fields) -> Result<(TokenStream, String), Error> {
    let mut doc = String::new();
    use std::fmt::Write;
    let body_tt = match body {
        &syn::Fields::Named(ref fields) => {
            doc.push_str(" {");
            let result = {
                let field_assignments = fields.named.iter().map(|field| {
                    let field_name = field.ident.as_ref();
                    let (default_value, default_doc) = field_default_expr_and_doc(field)?;
                    write!(&mut doc, "\n    {}: {},", field_name.expect("field value in struct is empty"), default_doc).unwrap();
                    // let default_value = default_value.into_token_stream();
                    Ok(quote! { #field_name : #default_value })
                }).collect::<Result<Vec<_>, Error>>()?;
                quote!{
                    {
                        #( #field_assignments ),*
                    }
                }
            };
            if (&mut doc).ends_with(",") {
                doc.pop();
                doc.push('\n');
            };
            doc.push('}');
            result
        }
        &syn::Fields::Unnamed(ref fields) => {
            doc.push('(');
            let result = {
                let field_assignments = fields.unnamed.iter().map(|field| {
                    let (default_value, default_doc) = field_default_expr_and_doc(field)?;
                    write!(&mut doc, "{}, ", default_doc).unwrap();
                    Ok(default_value)
                }).collect::<Result<Vec<TokenStream>, Error>>()?;
                quote! {
                    (
                        #( #field_assignments ),*
                    )
                }
            };
            if (&mut doc).ends_with(", ") {
                doc.pop();
                doc.pop();
            };
            doc.push(')');
            result
        }
        &syn::Fields::Unit => quote!{},
    };
    Ok((body_tt, doc))
}

/// Return a default expression for a field based on it's `#[default = "..."]` attribute. Panic
/// if there is more than one, of if there is a `#[default]` attribute without value.
fn field_default_expr_and_doc(field: &syn::Field) -> Result<(TokenStream, String), Error> {
    if let Some(field_value) = find_default_attr_value(&field.attrs)? {
        let field_value = field_value.ok_or_else(|| {
            Error::new(field.span(), "Expected #[default = ...] or #[default(...)]")})?;
        let field_doc = format!("{}", field_value);
        Ok((field_value, field_doc))
    } else {
        Ok((quote! {
            Default::default()
        }, "Default::default()".to_owned()))
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

    if segment.arguments != syn::PathArguments::None {
        return Ok(false);
    }

    Ok(segment.ident.to_string() == "default")
}

fn find_default_attr_value(attrs: &[syn::Attribute]) -> Result<Option<Option<TokenStream>>, Error> {
    if let Some(default_attr) = find_only(attrs.iter(), |attr| is_default_attr(attr))? {
        match default_attr.parse_meta() {
            Ok(syn::Meta::Word(_)) => Ok(Some(None)),
            Ok(syn::Meta::List(meta)) => {
                if let Some(field_value) = single_value(meta.nested.iter()) {
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

/// Return the value that fulfills the predicate if there is one in the slice. Panic if there is
/// more than one.
fn find_only<T, F>(iter: impl Iterator<Item = T>, pred: F) -> Result<Option<T>, Error>
where T: Spanned,
      F: Fn(&T) -> Result<bool, Error>,
{
    let mut result = None;
    for item in iter {
        if pred(&item)? {
            if result.is_some() {
                return Err(Error::new(item.span(), "Multiple defaults"));
            }
            result = Some(item);
        }
    }
    Ok(result)
}

fn single_value<T>(mut it: impl Iterator<Item = T>) -> Option<T> {
    if let Some(result) = it.next() {
        if it.next().is_none() {
            return Some(result)
        }
    }
    None
}
