use google_cloud_derive::{FromValue, IntoValue};

#[derive(FromValue, IntoValue)]
pub struct Foo {
    bar: String,
    baz: i64,
    qux: bool,
}

fn main() {}
