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
