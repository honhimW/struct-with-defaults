#![feature(default_field_values)]

mod nightly;

#[cfg(test)]
mod test {
    use macro3681::default_field_values;
    default_field_values! {
        struct Example {
            i: u32 = 3,
            j: i128,
            i_option: Option<u64> = Some(1000),
            string: String = {
                let s = format!("{} {}", "hello", "world");
                s
            },
            os: Option<String>,
            foo: Foo = _,
            bytes: &'static[u8] = b"hello world",
        }
    }

    #[derive(Eq, PartialEq, Debug)]
    struct Foo {
        bar: String,
    }

    impl Default for Foo {
        fn default() -> Self {
            Self { bar: "bar".to_string() }
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
        let config = Example::new(0, None);
        assert_eq!(config.i, 3);
        assert_eq!(config.j, 0);
        assert_eq!(config.i_option, Some(1000));
        assert_eq!(config.string, "hello world".to_string());
        assert_eq!(config.os, None);
        assert_eq!(config.foo, Foo { bar: "bar".to_string() });
        assert_eq!(config.bytes, b"hello world");
        let c = ExampleWithDefaultTrait::new(101).clone();
        assert_eq!(c.i, 101);
        assert_eq!(c.j, 400);
        let c = ExampleWithDefaultTrait::default();
        assert_eq!(c.i, 0);
        assert_eq!(c.j, 400);
        assert_eq!(c.hello, "hello world".to_string());
    }

}
