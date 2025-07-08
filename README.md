# Macro 3681

### FYI
- [Rust rfcs-3681](https://github.com/rust-lang/rfcs/pull/3681)
- [Tracking issue for RFC 3681: Default field values](https://github.com/rust-lang/rust/issues/132162)

## About this crate

> Thinking about default values in structs.  
> Implement via proc-macro.  
> This is just a toy. Do not use in production.

## Usage

### With Default derive

Default derive will be automatically removed, and generate an impl with default field values.
If there are none-default fields, those fields will be added as constructor parameters. 
```rust
default_field_values! {
    #[derive(Debug, Default)]
    pub struct Foo<'a, 'b>
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
    }
}
```

#### Macro Expansion

```rust
#[derive(Debug)]
pub struct Foo<'a, 'b> {
    i: i32,
    string: String,
    option_string: Option<String>,
    bytes: &'a [u8],
    bytes2: &'b [u8],
    b: bool,
}
impl<'a, 'b> Foo<'a, 'b> {
    pub fn new(option_string: Option<String>, b: bool) -> Self {
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

## Limitation

> Cannot work with Generics. 
