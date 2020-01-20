use gcp_derive::{FromValue, IntoValue};

#[derive(FromValue, IntoValue)]
pub struct Foo {
    bar: Bar,
    qux: bool,
}

#[derive(FromValue, IntoValue)]
pub struct Bar {
    baz: String,
}

fn main() {}
