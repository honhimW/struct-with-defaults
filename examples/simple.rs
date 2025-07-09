use macro3681::default_field_values;

default_field_values! {
    #[derive(Debug, Default)]
    pub struct Foo<'a, 'b> {
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

fn main() {
    let foo = Foo::new(None, true);
    dbg!("{:?}", foo);
}
