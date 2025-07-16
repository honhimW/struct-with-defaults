# Macro 3681

### FYI
- [Rust rfcs-3681](https://github.com/rust-lang/rfcs/pull/3681)
- [Tracking issue for RFC 3681: Default field values](https://github.com/rust-lang/rust/issues/132162)

## About this crate

#### MSRV
Since crate `syn`'s msrv is `1.61`, that also the msrv for this crate.

- Thinking about default values in structs.  
- Implement via proc-macro.
- RFC-3681 default field values must be `const`, which is not enough for most cases e.g. `String`.

## Usage

```shell
cargo add macro3681
```

### With Default derive

Default derive will be automatically removed, and generate an impl with default field values.
If there are none-default fields, those fields will be added as constructor parameters. 
```rust
default_field_values! {
    #[derive(Debug, Default)]
    pub struct Foo<'a, 'b, T: Default, T2> where T2: Default
    {
        i: i32 = 1,
        string: String = {
            let s = format!("{} {}", "hello", "world");
            s
        }
        option_string: Option<String>,
        bytes: &'a[u8] = b"hello world",
        bytes2: &'b[u8] = b"hello world",
        b: bool,
        t: T,
        t: T2,
    }
}
```

#### Macro Expansion

```rust
#[derive(Debug)]
pub struct Foo<'a, 'b, T: Default, T2> where T2: Default {
    i: i32,
    string: String,
    option_string: Option<String>,
    bytes: &'a [u8],
    bytes2: &'b [u8],
    b: bool,
    t: T,
    t: T2,
}
impl<'a, 'b> Foo<'a, 'b> {
    pub fn new(option_string: Option<String>, b: bool, t: T, t2: T2) -> Self {
        Self {
            i: 1,
            string: {
                let s = format!("{} {}", "hello", "world");
                s
            }
            ,
            option_string,
            bytes: b"hello world",
            bytes2: b"hello world",
            b,
            t,
            t2,
        }
    }
}
impl<'a, 'b> Default for Foo<'a, 'b> {
    fn default() -> Self {
        Self {
            i: 1,
            string: {
                let s = format!("{} {}", "hello", "world");
                s
            }
            ,
            bytes: b"hello world",
            bytes2: b"hello world",
            option_string: Default::default(),
            b: Default::default(),
            t: Default::default(),
            t2: Default::default(),
        }
    }
}
```

### Without Default derive

Will not generate `Self::default()` function, only `Self::new(..)` is available
```rust
default_field_values! {
    #[derive(Clone, Debug)]
    pub(crate) struct Bar {
        pub i: u32,
        j: u32 = 100 * 4,
    }
}
```
