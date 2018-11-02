
extern crate smart_default;

use smart_default::SmartDefault;

#[derive(PartialEq, SmartDefault, Debug)]
#[allow(dead_code)]
enum Foo {
    Bar,
    #[default]
    Baz {
        #[default(12)]
        a: i32,
        b: i32,
        #[default(Some(Default::default()))]
        c: Option<i32>
    },
    Qux(i32),
}

fn main() {
    assert!(Foo::default() == Foo::Baz { a: 12, b: 0, c: Some(0) });
}
