#[cfg(test)]
#[cfg(nightly)]
mod test {

    #[derive(Debug)]
    struct Foo<'a> {
        a: i32 = 1,
        t: &'a str = "hello",
    }

    #[test]
    fn new() {
        let foo = Foo { .. };
        dbg!(foo);
        println!("in nightly");
    }

}
