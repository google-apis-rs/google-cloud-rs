extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::{Fields, FieldsNamed, Ident, ItemStruct, LitInt, LitStr};

#[proc_macro_derive(IntoValue)]
pub fn derive_into_value(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);

    let name = &input.ident;
    let (keys, fields) = if let Fields::Named(FieldsNamed { ref named, .. }) = input.fields {
        let fields: Vec<&Ident> = named
            .iter()
            .map(|field| field.ident.as_ref().unwrap())
            .collect();
        let keys: Vec<LitStr> = fields
            .iter()
            .map(|ident| LitStr::new(ident.to_string().as_str(), ident.span()))
            .collect();
        (keys, fields)
    } else {
        todo!()
    };

    let capacity = LitInt::new(keys.len().to_string().as_str(), name.span());

    let tokens = quote! {
        impl ::google_cloud::datastore::IntoValue for #name {
            fn into_value(self) -> ::google_cloud::datastore::Value {
                let mut props = ::std::collections::HashMap::with_capacity(#capacity);
                #(props.insert(String::from(#keys), self.#fields.into_value());)*
                ::google_cloud::datastore::Value::EntityValue(props)
            }
        }
    };

    tokens.into()
}

#[proc_macro_derive(FromValue)]
pub fn derive_from_value(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);

    let name = &input.ident;
    let (keys, fields) = if let Fields::Named(FieldsNamed { ref named, .. }) = input.fields {
        let fields: Vec<&Ident> = named
            .iter()
            .map(|field| field.ident.as_ref().unwrap())
            .collect();
        let keys: Vec<LitStr> = fields
            .iter()
            .map(|ident| LitStr::new(ident.to_string().as_str(), ident.span()))
            .collect();
        (keys, fields)
    } else {
        todo!()
    };

    let tokens = quote! {
        impl ::google_cloud::datastore::FromValue for #name {
            fn from_value(value: ::google_cloud::datastore::Value) -> std::result::Result<#name, ::google_cloud::error::ConvertError> {
                let mut props = match value {
                    ::google_cloud::datastore::Value::EntityValue(props) => props,
                    _ => return Err(::google_cloud::error::ConvertError::UnexpectedPropertyType {
                        expected: String::from("entity"),
                        got: String::from(value.type_name()),
                    }),
                };
                let value = #name {
                    #(#fields: {
                        let prop = props
                            .remove(#keys)
                            .ok_or_else(|| {
                                ::google_cloud::error::ConvertError::MissingProperty(String::from(#keys))
                            })?;
                        let value = ::google_cloud::datastore::FromValue::from_value(prop)?;
                        value
                    },)*
                };
                Ok(value)
            }
        }
    };

    tokens.into()
}
