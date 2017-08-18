[![Build Status](https://api.travis-ci.org/idanarye/rust-smart-default.svg?branch=master)](https://travis-ci.org/idanarye/rust-smart-default)
[![Latest Version](https://img.shields.io/crates/v/smart-default.svg)](https://crates.io/crates/smart-default)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://idanarye.github.io/rust-smart-default/)

# Rust SmartDefault

Custom derive for automatically implementing the `Default` trait with customized default values:

```rust
#[derive(SmartDefault)]
enum Foo {
    Bar,
    #[default]
    Baz {
        #[default = "12"]
        a: i32,
        b: i32,
        #[default = r#""hello""#]
        c: &'static str,
    },
    Qux(i32),
}

assert!(Foo::default() == Foo::Baz { a: 12, b: 0, c: "hello" });
```
