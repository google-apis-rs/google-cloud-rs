use google_cloud::datastore::{FromValue, IntoValue};
use google_cloud::error::ConvertError;

#[derive(Debug, FromValue, IntoValue)]
#[datastore(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum Foo {
    Bar,
    Baz,
    TrickyOne,
    AndAnotherOne,
    AreWeThereYet,
}

fn main() {
    let foo = Foo::AreWeThereYet;
    println!("original: {:?}", foo);

    let converted = foo.into_value();
    println!("converted: {:?}", converted);

    let recovered: Result<Foo, ConvertError> = Foo::from_value(converted);
    println!("recovered: {:?}", recovered);
}
