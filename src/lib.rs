mod nightly;

#[cfg(test)]
mod test {
    use macro3681::default_field_values;
    default_field_values! {
        #[derive(Default, Debug)]
        struct Example<'a, T, T2: Default> where T: Default {
            i: u32 = 3,
            j: i128,
            i_option: Option<u64> = Some(1000),
            string: String = {
                let s = format!("{} {}", "hello", "world");
                s
            },
            os: Option<String>,
            foo: Foo = _, // #[derive(Default, Debug)] struct Foo { .. }
            bytes: &'a[u8] = b"hello world",
            t: T,
            t2: T2,
        }

        #[derive(Default, Debug)]
        struct Tuple<'a, T: Default>(T, #[allow(unused)] &'a str = "abc", Foo = _, Option<String>);
    }

    // default_field_values! {
    //     #[derive(Default, Debug)]
    //     struct Tuple<'a, T: Default>(T, #[allow(unused)] &'a str = "abc", Foo = _, Option<String>);
    // }

    #[derive(Eq, PartialEq, Debug)]
    struct Foo {
        bar: String,
    }

    impl Default for Foo {
        fn default() -> Self {
            Self {
                bar: "bar".to_string(),
            }
        }
    }

    default_field_values! {
        #[derive(Default, Clone, Debug)]
        pub(crate) struct ExampleWithDefaultTrait {
            i: u32,
            j: u32 = 100 * 4,
            hello: String = get(),
        }
    }

    fn get() -> String {
        "hello world".to_string()
    }

    #[test]
    fn defaults() {
        let config = Example::new(0, None, 1, "foo");
        assert_eq!(config.i, 3);
        assert_eq!(config.j, 0);
        assert_eq!(config.i_option, Some(1000));
        assert_eq!(config.string, "hello world".to_string());
        assert_eq!(config.os, None);
        assert_eq!(
            config.foo,
            Foo {
                bar: "bar".to_string()
            }
        );
        assert_eq!(config.bytes, b"hello world");
        assert_eq!(config.t, 1);
        assert_eq!(config.t2, "foo");

        let c = ExampleWithDefaultTrait::new(101).clone();
        assert_eq!(c.i, 101);
        assert_eq!(c.j, 400);
        let c = ExampleWithDefaultTrait::default();
        assert_eq!(c.i, 0);
        assert_eq!(c.j, 400);
        assert_eq!(c.hello, "hello world".to_string());

        let tuple = Tuple::<bool>::default();
        assert_eq!(tuple.0, false);
        assert_eq!(tuple.1, "abc");
        assert_eq!(
            tuple.2,
            Foo {
                bar: "bar".to_string()
            }
        );
        assert_eq!(tuple.3, None);
    }
}
