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
```

#### Macro Expansion

```rust
#[derive(Debug)]
struct Example<'a, T, T2>
where
    T: Default,
{
    i: u32,
    j: i128,
    i_option: Option<u64>,
    string: String,
    os: Option<String>,
    foo: Foo,
    bytes: &'a [u8],
    t: T,
    t2: T2,
}
impl<'a, T, T2: Default> Example<'a, T, T2>
where
    T: Default,
{
    pub fn new(j: i128, os: Option<String>, t: T, t2: T2) -> Self {
        Self {
            i: 3,
            j,
            i_option: Some(1000),
            string: {
                let s = format!("{} {}", "hello", "world");
                s
            },
            os,
            foo: Default::default(),
            bytes: b"hello world",
            t,
            t2,
        }
    }
}
impl<'a, T, T2: Default> Default for Example<'a, T, T2>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            i: 3,
            i_option: Some(1000),
            string: {
                let s = format!("{} {}", "hello", "world");
                s
            },
            foo: Default::default(),
            bytes: b"hello world",
            j: Default::default(),
            os: Default::default(),
            t: Default::default(),
            t2: Default::default(),
        }
    }
}

#[derive(Debug)]
struct Tuple<'a, T> (T, #[allow(unused)]   &'a str, Foo, Option<String> );
impl<'a, T: Default> Tuple<'a, T> { pub fn new(_0: T, _3: Option<String>) -> Self { Self(_0, "abc", Default::default(), _3 ) } }
impl<'a, T: Default> Default for Tuple<'a, T> { fn default() -> Self { Self(Default::default(), "abc", Default::default(), Default::default() ) } 
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
