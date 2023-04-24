use smart_default::SmartDefault;

#[test]
fn test_unit() {
    #[derive(Debug, PartialEq, SmartDefault)]
    struct Foo;

    assert_eq!(Foo::default(), Foo);
}

#[test]
fn test_tuple() {
    #[derive(Debug, PartialEq, SmartDefault)]
    struct Foo(
        #[default = 10] i32,
        #[default = 20] i32,
        // No default
        i32,
    );

    assert_eq!(Foo::default(), Foo(10, 20, 0));
}

#[test]
fn test_struct() {
    #[derive(Debug, PartialEq, SmartDefault)]
    struct Foo {
        #[default = 10]
        x: i32,
        #[default = 20]
        y: i32,
        // No default
        z: i32,
    }

    assert_eq!(Foo::default(), Foo { x: 10, y: 20, z: 0 });
}

#[test]
fn test_enum_of_units() {
    #[derive(Debug, PartialEq, SmartDefault)]
    pub enum Foo {
        #[allow(dead_code)]
        Bar,
        #[default]
        Baz,
        #[allow(dead_code)]
        Qux,
    }

    assert_eq!(Foo::default(), Foo::Baz);
}

#[test]
fn test_enum_of_tuples() {
    #[derive(Debug, PartialEq, SmartDefault)]
    pub enum Foo {
        #[allow(dead_code)]
        Bar(i32),
        #[default]
        Baz(#[default = 10] i32, i32),
        #[allow(dead_code)]
        Qux(i32),
    }

    assert_eq!(Foo::default(), Foo::Baz(10, 0));
}

#[test]
fn test_enum_of_structs() {
    #[derive(Debug, PartialEq, SmartDefault)]
    pub enum Foo {
        #[allow(dead_code)]
        Bar { x: i32 },
        #[default]
        Baz {
            #[default = 10]
            y: i32,
            z: i32,
        },
        #[allow(dead_code)]
        Qux { w: i32 },
    }

    assert_eq!(Foo::default(), Foo::Baz { y: 10, z: 0 });
}

#[test]
fn test_enum_mixed() {
    #[derive(Debug, PartialEq, SmartDefault)]
    enum Foo {
        #[allow(dead_code)]
        Bar,
        #[default]
        Baz(#[default = 10] i32),
        #[allow(dead_code)]
        Qux { w: i32 },
    }

    assert_eq!(Foo::default(), Foo::Baz(10));
}

#[test]
fn test_generics_type_parameters() {
    #[derive(Debug, PartialEq, SmartDefault)]
    struct Foo<T>
    where
        T: Default,
    {
        #[default(Some(Default::default()))]
        x: Option<T>,
    }

    assert_eq!(Foo::default(), Foo { x: Some(0) });
}

#[test]
fn test_generics_lifetime_parameters() {
    // NOTE: A default value makes no sense with lifetime parameters, since ::default() receives no
    // paramters and therefore can receive no lifetimes. But it does make sense if you make a variant
    // without ref fields the default.

    #[derive(Debug, PartialEq, SmartDefault)]
    enum Foo<'a> {
        #[default]
        Bar(i32),
        #[allow(dead_code)]
        Baz(&'a str),
    }

    assert_eq!(Foo::default(), Foo::Bar(0));
}

#[test]
fn test_code_hack() {
    #[derive(Debug, PartialEq, SmartDefault)]
    struct Foo {
        #[default(_code = "vec![1, 2, 3]")]
        v: Vec<u32>,
    }

    assert!(Foo::default().v == [1, 2, 3]);
}

#[test]
fn test_string_conversion() {
    #[derive(Debug, PartialEq, SmartDefault)]
    struct Foo(#[default = "one"] &'static str, #[default("two")] String);

    assert_eq!(Foo::default(), Foo("one", "two".to_owned()));
}

#[test] // https://github.com/idanarye/rust-smart-default/issues/13
fn test_issue_13_bool() {
    #[derive(Debug, PartialEq, SmartDefault)]
    struct Foo {
        #[default(1)]
        i: i32,
        #[default(true)]
        b: bool,
    }

    assert_eq!(Foo::default(), Foo { i: 1, b: true });
}

#[test] // https://github.com/idanarye/rust-smart-default/issues/13
fn test_issue_13_enum() {
    #[derive(Debug, PartialEq)]
    enum Either {
        #[allow(dead_code)]
        Left,
        Right,
    }
    #[derive(Debug, PartialEq, SmartDefault)]
    struct Foo {
        #[default(1)]
        i: i32,
        #[default(Either::Right)]
        b: Either,
    }

    assert_eq!(
        Foo::default(),
        Foo {
            i: 1,
            b: Either::Right
        }
    );
}
