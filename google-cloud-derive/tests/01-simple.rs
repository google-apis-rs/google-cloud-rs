use google_cloud::datastore::{FromValue, IntoValue};
use google_cloud::error::ConvertError;

#[derive(Debug, FromValue, IntoValue)]
pub struct Foo {
    bar: String,
    baz: i64,
    qux: bool,
}

fn main() {
    let foo = Foo {
        bar: String::from("test"),
        baz: 63,
        qux: true,
    };
    println!("original: {:?}", foo);

    let converted = foo.into_value();
    println!("converted: {:?}", converted);

    let recovered: Result<Foo, ConvertError> = Foo::from_value(converted);
    println!("recovered: {:?}", recovered);
}
