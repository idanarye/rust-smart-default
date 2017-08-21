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
//!         #[default = "12"]
//!         a: i32,
//!         b: i32,
//!         #[default = r#""hello""#]
//!         c: &'static str,
//!     },
//!     Qux(i32),
//! }
//!
//! assert!(Foo::default() == Foo::Baz { a: 12, b: 0, c: "hello" });
//! # }
//! ```
//!
//! * `Baz` has the `#[default]` attribute. This means that the default `Foo` is a `Foo::Baz`. Only
//!   one variant may have a `#[default]` attribute, and that attribute must have no value.
//! * `a` has a `#[default = "12"]` attribute. This means that it's default value is `12`.
//!   Currently custom attributes can only be strings, so the default value must be encoded in a
//!   string as well.
//! * `b` has no `#[default = "..."]` attribute. It's default value will `i32`'s default value
//!   instead - `0`.
//! * `c` is a string, and thus it's default value - a string - must be escaped inside that
//!   attribute. You can't use `#[default = "hello"]` here - that will look for a constant named
//!   `hello` and use it's value as `c`'s default.
//! * Documentation for the `impl Default` section is generated automatically, specifying the
//!   default value returned from `::default()`.

extern crate proc_macro;
extern crate syn;

#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

#[doc(hidden)]
#[proc_macro_derive(SmartDefault, attributes(default))]
pub fn derive_smart_default(input: TokenStream) -> TokenStream {
    let ast = syn::parse_derive_input(&input.to_string()).unwrap();
    impl_my_derive(&ast).parse().unwrap()
}

fn impl_my_derive(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let (default_expr, doc) = match ast.body {
        syn::Body::Struct(ref body) => {
            let (body_assignment, doc) = default_body_tt(body);
            (quote! {
                #name #body_assignment
            }, format!("Return `{}{}`", name, doc))
        }
        syn::Body::Enum(ref variants) => {
            let default_variant = find_only(variants, |variant| {
                if let Some(default_attr) =
                    find_only(&variant.attrs, |attr| attr.name() == "default") {
                    if let syn::MetaItem::Word(_) = default_attr.value {
                        true
                    } else {
                        panic!("Attribute #[default] on variants should have no value");
                    }
                } else {
                    false
                }
            });
            let default_variant = default_variant.expect("No default variant");
            let default_variant_name = &default_variant.ident;
            let (body_assignment, doc) = default_body_tt(&default_variant.data);
            (quote! {
                #name :: #default_variant_name #body_assignment
            }, format!("Return `{}::{}{}`", name, default_variant_name, doc))
        }
    };
    quote! {
        impl #impl_generics Default for #name #ty_generics #where_clause {
            #[doc = #doc]
            fn default() -> Self {
                #default_expr
            }
        }
    }
}

/// Return a token-tree for the default "body" - the part after the name that contains the values.
/// That is, the `{ ... }` part for structs, the `(...)` part for tuples, and nothing for units.
fn default_body_tt(body: &syn::VariantData) -> (quote::Tokens, String) {
    let mut doc = String::new();
    use std::fmt::Write;
    let body_tt = match body {
        &syn::VariantData::Struct(ref fields) => {
            doc.push_str(" {");
            let result = {
                let field_assignments = fields.iter().map(|field| {
                    let field_name = field.ident.as_ref();
                    let (default_value, default_doc) = field_default_expr_and_doc(field);
                    write!(&mut doc, "\n    {}: {},", field_name.expect("field value in struct is empty"), default_doc).unwrap();
                    quote! { #field_name : #default_value }
                });
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
        &syn::VariantData::Tuple(ref fields) => {
            doc.push('(');
            let result = {
                let field_assignments = fields.iter().map(|field| {
                    let (default_value, default_doc) = field_default_expr_and_doc(field);
                    write!(&mut doc, "{}, ", default_doc).unwrap();
                    default_value
                });
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
        &syn::VariantData::Unit => quote!{},
    };
    (body_tt, doc)
}

/// Return a default expression for a field based on it's `#[default = "..."]` attribute. Panic
/// if there is more than one, of if there is a `#[default]` attribute without value.
fn field_default_expr_and_doc(field: &syn::Field) -> (quote::Tokens, &str) {
    if let Some(default_attr) = find_only(&field.attrs, |attr| attr.name() == "default") {
        if let syn::MetaItem::NameValue(_, syn::Lit::Str(ref lit, _)) = default_attr.value {
            let field_value = syn::parse_token_trees(lit).unwrap();
            return (quote! {
                #( #field_value )*
            }, lit);
        } else {
            panic!("Attribute #[default] on fields must have a value");
        }
    }
    (quote! {
        Default::default()
    }, "Default::default()")
}

/// Return the value that fulfills the predicate if there is one in the slice. Panic if there is
/// more than one.
fn find_only<T, F>(iter: &[T], pred: F) -> Option<&T>
where
	F: Fn(&T) -> bool,
{
    let mut result = None;
    for item in iter {
        if pred(item) {
            assert!(result.is_none(), "Multiple defaults");
            result = Some(item);
        }
    }
    result
}
