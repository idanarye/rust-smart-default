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
    match ast.body {
        syn::Body::Struct(ref body) => {
            let body_assignment = default_body_tt(body);
            quote! {
                impl #impl_generics Default for #name #ty_generics #where_clause {
                    fn default() -> Self {
                        #name #body_assignment
                    }
                }
            }
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
            let body_assignment = default_body_tt(&default_variant.data);
            quote! {
                impl #impl_generics Default for #name #ty_generics #where_clause {
                    fn default() -> Self {
                        #name :: #default_variant_name #body_assignment
                    }
                }
            }
        }
    }
}

/// Return a token-tree for the default "body" - the part after the name that contains the values.
/// That is, the `{ ... }` part for structs, the `(...)` part for tuples, and nothing for units.
fn default_body_tt(body: &syn::VariantData) -> quote::Tokens {
    match body {
        &syn::VariantData::Struct(ref fields) => {
            let field_assignments = fields.iter().map(|field| {
                let field_name = field.ident.as_ref();
                let default = field_default_expr(field);
                quote! { #field_name : #default }
            });
            quote!{
                {
                    #( #field_assignments ),*
                }
            }
        }
        &syn::VariantData::Tuple(ref fields) => {
            let field_assignments = fields.iter().map(|field| field_default_expr(field));
            quote! {
                (
                    #( #field_assignments ),*
                )
            }
        }
        &syn::VariantData::Unit => quote!{},
    }
}

/// Return a default expression for a field based on it's `#[default = "..."]` attribute. Panic
/// if there is more than one, of if there is a `#[default]` attribute without value.
fn field_default_expr(field: &syn::Field) -> quote::Tokens {
    if let Some(default_attr) = find_only(&field.attrs, |attr| attr.name() == "default") {
        if let syn::MetaItem::NameValue(_, syn::Lit::Str(ref lit, _)) = default_attr.value {
            let field_value = syn::parse_token_trees(lit).unwrap();
            return quote! {
                #( #field_value )*
            };
        } else {
            panic!("Attribute #[default] on fields must have a value");
        }
    }
    quote! {
        Default::default()
    }
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
