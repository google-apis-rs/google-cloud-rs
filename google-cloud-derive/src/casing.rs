use crate::RenameAll;

pub(crate) fn transform_variant_casing(variant: syn::Ident, rename_all: RenameAll) -> String {
    match rename_all {
        RenameAll::LowerCase => variant.to_string().to_ascii_lowercase(),
        RenameAll::UpperCase => variant.to_string().to_ascii_uppercase(),
        RenameAll::CamelCase => {
            let variant = variant.to_string();
            variant[..1].to_ascii_lowercase() + &variant[1..]
        }
        RenameAll::PascalCase => variant.to_string(),
        RenameAll::SnakeCase => {
            let variant = variant.to_string();
            let mut snake = String::new();
            for (i, ch) in variant.char_indices() {
                if i > 0 && ch.is_uppercase() {
                    snake.push('_');
                }
                snake.push(ch.to_ascii_lowercase());
            }
            snake
        }
        RenameAll::ScreamingSnakeCase => {
            transform_variant_casing(variant, RenameAll::SnakeCase).to_ascii_uppercase()
        }
        RenameAll::KebabCase => {
            transform_variant_casing(variant, RenameAll::SnakeCase).replace('_', "-")
        }
        RenameAll::ScreamingKebabCase => {
            transform_variant_casing(variant, RenameAll::ScreamingSnakeCase).replace('_', "-")
        }
    }
}

pub(crate) fn transform_field_casing(field: syn::Ident, rename_all: RenameAll) -> String {
    match rename_all {
        RenameAll::LowerCase => field.to_string(),
        RenameAll::UpperCase => field.to_string().to_ascii_uppercase(),
        RenameAll::CamelCase => {
            let pascal = transform_field_casing(field, RenameAll::PascalCase);
            pascal[..1].to_ascii_lowercase() + &pascal[1..]
        }
        RenameAll::PascalCase => {
            let field = field.to_string();
            let mut pascal = String::new();
            let mut capitalize = true;
            for ch in field.chars() {
                if ch == '_' {
                    capitalize = true;
                } else if capitalize {
                    pascal.push(ch.to_ascii_uppercase());
                    capitalize = false;
                } else {
                    pascal.push(ch);
                }
            }
            pascal
        }
        RenameAll::SnakeCase => field.to_string(),
        RenameAll::ScreamingSnakeCase => field.to_string().to_ascii_uppercase(),
        RenameAll::KebabCase => field.to_string().replace('_', "-"),
        RenameAll::ScreamingKebabCase => {
            transform_field_casing(field, RenameAll::ScreamingSnakeCase).replace('_', "-")
        }
    }
}
