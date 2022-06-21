extern crate proc_macro;

use proc_macro::TokenStream;

use darling::{FromDeriveInput, FromField, FromMeta, FromVariant};
use quote::quote;
use syn::parse_macro_input;

mod casing;

use crate::casing::{transform_field_casing, transform_variant_casing};

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromMeta)]
pub(crate) enum RenameAll {
    #[darling(rename = "lowercase")]
    Lower,
    #[darling(rename = "UPPERCASE")]
    Upper,
    #[darling(rename = "PascalCase")]
    Pascal,
    #[darling(rename = "camelCase")]
    Camel,
    #[darling(rename = "snake_case")]
    Snake,
    #[darling(rename = "SCREAMING_SNAKE_CASE")]
    ScreamingSnake,
    #[darling(rename = "kebab-case")]
    Kebab,
    #[darling(rename = "SCREAMING-KEBAB-CASE")]
    ScreamingKebab,
}

impl Default for RenameAll {
    fn default() -> RenameAll {
        RenameAll::Camel
    }
}

#[derive(Debug, Clone, PartialEq, Eq, FromDeriveInput)]
#[darling(attributes(datastore), supports(struct_named, enum_unit))]
struct Container {
    pub ident: syn::Ident,
    // pub vis: syn::Visibility,
    // pub generics: syn::Generics,
    pub data: darling::ast::Data<VariantContainer, FieldContainer>,
    // pub attrs: Vec<syn::Attribute>,
    #[darling(default)]
    pub rename_all: RenameAll,
}

#[derive(Debug, Clone, PartialEq, Eq, FromVariant)]
#[darling(attributes(datastore))]
struct VariantContainer {
    pub ident: syn::Ident,
    #[darling(default)]
    pub rename: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, FromField)]
#[darling(attributes(datastore))]
struct FieldContainer {
    pub ident: Option<syn::Ident>,
    #[darling(default)]
    pub rename: Option<String>,
}

fn derive_into_value_struct(
    ident: syn::Ident,
    fields: Vec<FieldContainer>,
    rename_all: RenameAll,
) -> TokenStream {
    let idents: Vec<syn::Ident> = fields
        .iter()
        .map(|field| field.ident.clone().unwrap())
        .collect();
    let names: Vec<syn::LitStr> = fields
        .into_iter()
        .map(|field| {
            let renamed = field.rename;
            let field = field.ident.unwrap();
            let span = field.span();
            let name = renamed.unwrap_or_else(|| transform_field_casing(field, rename_all));
            syn::LitStr::new(name.as_str(), span)
        })
        .collect();

    let capacity = names.len();

    let tokens = quote! {
        impl ::google_cloud::datastore::IntoValue for #ident {
            fn into_value(self) -> ::google_cloud::datastore::Value {
                let mut props = ::std::collections::HashMap::with_capacity(#capacity);
                #(props.insert(::std::string::String::from(#names), self.#idents.into_value());)*
                ::google_cloud::datastore::Value::EntityValue(props)
            }
        }
    };

    tokens.into()
}

fn derive_into_value_enum(
    ident: syn::Ident,
    variants: Vec<VariantContainer>,
    rename_all: RenameAll,
) -> TokenStream {
    let idents: Vec<syn::Ident> = variants
        .iter()
        .map(|variant| variant.ident.clone())
        .collect();
    let names: Vec<syn::LitStr> = variants
        .into_iter()
        .map(|variant| {
            let renamed = variant.rename;
            let variant = variant.ident;
            let span = variant.span();
            let name = renamed.unwrap_or_else(|| transform_variant_casing(variant, rename_all));
            syn::LitStr::new(name.as_str(), span)
        })
        .collect();

    let tokens = quote! {
        impl ::google_cloud::datastore::IntoValue for #ident {
            fn into_value(self) -> ::google_cloud::datastore::Value {
                match self {
                    #(#ident::#idents => ::google_cloud::datastore::Value::StringValue(#names.to_string()),)*
                }
            }
        }
    };

    tokens.into()
}

#[proc_macro_derive(IntoValue, attributes(datastore))]
pub fn derive_into_value(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let container = Container::from_derive_input(&input).unwrap();

    let ident = container.ident;
    let rename_all = container.rename_all;

    match container.data {
        darling::ast::Data::Enum(variants) => derive_into_value_enum(ident, variants, rename_all),
        darling::ast::Data::Struct(darling::ast::Fields { fields, .. }) => {
            derive_into_value_struct(ident, fields, rename_all)
        }
    }
}

fn derive_from_value_struct(
    ident: syn::Ident,
    fields: Vec<FieldContainer>,
    rename_all: RenameAll,
) -> TokenStream {
    let idents: Vec<syn::Ident> = fields
        .iter()
        .map(|field| field.ident.clone().unwrap())
        .collect();
    let names: Vec<syn::LitStr> = fields
        .into_iter()
        .map(|field| {
            let renamed = field.rename;
            let field = field.ident.unwrap();
            let span = field.span();
            let name = renamed.unwrap_or_else(|| transform_field_casing(field, rename_all));
            syn::LitStr::new(name.as_str(), span)
        })
        .collect();

    let tokens = quote! {
        impl ::google_cloud::datastore::FromValue for #ident {
            fn from_value(value: ::google_cloud::datastore::Value) -> ::std::result::Result<#ident, ::google_cloud::error::ConvertError> {
                let mut props = match value {
                    ::google_cloud::datastore::Value::EntityValue(props) => props,
                    _ => return ::std::result::Result::Err(
                        ::google_cloud::error::ConvertError::UnexpectedPropertyType {
                            expected: ::std::string::String::from("entity"),
                            got: ::std::string::String::from(value.type_name()),
                        }
                    ),
                };
                let value = #ident {
                    #(#idents: {
                        let prop = props
                            .remove(#names)
                            .ok_or_else(|| {
                                ::google_cloud::error::ConvertError::MissingProperty(::std::string::String::from(#names))
                            })?;
                        let value = ::google_cloud::datastore::FromValue::from_value(prop)?;
                        value
                    },)*
                };
                ::std::result::Result::Ok(value)
            }
        }
    };

    tokens.into()
}

fn derive_from_value_enum(
    ident: syn::Ident,
    variants: Vec<VariantContainer>,
    rename_all: RenameAll,
) -> TokenStream {
    let idents: Vec<syn::Ident> = variants
        .iter()
        .map(|variant| variant.ident.clone())
        .collect();
    let names: Vec<syn::LitStr> = variants
        .into_iter()
        .map(|variant| {
            let renamed = variant.rename;
            let variant = variant.ident;
            let span = variant.span();
            let name = renamed.unwrap_or_else(|| transform_variant_casing(variant, rename_all));
            syn::LitStr::new(name.as_str(), span)
        })
        .collect();

    let tokens = quote! {
        impl ::google_cloud::datastore::FromValue for #ident {
            fn from_value(value: ::google_cloud::datastore::Value) -> ::std::result::Result<#ident, ::google_cloud::error::ConvertError> {
                let value = match value {
                    ::google_cloud::datastore::Value::StringValue(value) => value,
                    _ => return ::std::result::Result::Err(
                        ::google_cloud::error::ConvertError::UnexpectedPropertyType {
                            expected: ::std::string::String::from("entity"),
                            got: ::std::string::String::from(value.type_name()),
                        }
                    ),
                };
                match value.as_str() {
                    #(#names => ::std::result::Result::Ok(#ident::#idents),)*
                    _ => todo!("[datastore-derive] unknown enum variant encountered"),
                }
            }
        }
    };

    tokens.into()
}

#[proc_macro_derive(FromValue, attributes(datastore))]
pub fn derive_from_value(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let container = Container::from_derive_input(&input).unwrap();

    let ident = container.ident;
    let rename_all = container.rename_all;

    match container.data {
        darling::ast::Data::Enum(variants) => derive_from_value_enum(ident, variants, rename_all),
        darling::ast::Data::Struct(darling::ast::Fields { fields, .. }) => {
            derive_from_value_struct(ident, fields, rename_all)
        }
    }
}
