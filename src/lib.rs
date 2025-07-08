#[cfg(test)]
mod test {
    use macros3681::default_field_values;
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
            bytes: &'static[u8] = b"hello world",
        }
    }

    default_field_values! {
        #[derive(Default, Clone, Debug)]
        pub(crate) struct ExampleWithDefaultTrait {
            i: u32,
            j: u32 = 100 * 4,
        }
    }

    #[test]
    fn defaults() {
        let config = Example::new(0, None);
        assert_eq!(config.i, 3);
        assert_eq!(config.j, 0);
        assert_eq!(config.i_option, Some(1000));
        assert_eq!(config.string, "hello world".to_string());
        assert_eq!(config.os, None);
        assert_eq!(config.bytes, b"hello world");
        let c = ExampleWithDefaultTrait::new(101).clone();
        assert_eq!(c.i, 101);
        assert_eq!(c.j, 400);
        let c = ExampleWithDefaultTrait::default();
        assert_eq!(c.j, 400);
    }

    #[test]
    fn construct() {}
}
