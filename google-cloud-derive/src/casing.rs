use crate::RenameAll;

pub(crate) fn transform_variant_casing(variant: syn::Ident, rename_all: RenameAll) -> String {
    match rename_all {
        RenameAll::Lower => variant.to_string().to_ascii_lowercase(),
        RenameAll::Upper => variant.to_string().to_ascii_uppercase(),
        RenameAll::Camel => {
            let variant = variant.to_string();
            variant[..1].to_ascii_lowercase() + &variant[1..]
        }
        RenameAll::Pascal => variant.to_string(),
        RenameAll::Snake => {
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
        RenameAll::ScreamingSnake => {
            transform_variant_casing(variant, RenameAll::Snake).to_ascii_uppercase()
        }
        RenameAll::Kebab => transform_variant_casing(variant, RenameAll::Snake).replace('_', "-"),
        RenameAll::ScreamingKebab => {
            transform_variant_casing(variant, RenameAll::ScreamingSnake).replace('_', "-")
        }
    }
}

pub(crate) fn transform_field_casing(field: syn::Ident, rename_all: RenameAll) -> String {
    match rename_all {
        RenameAll::Lower => field.to_string(),
        RenameAll::Upper => field.to_string().to_ascii_uppercase(),
        RenameAll::Camel => {
            let pascal = transform_field_casing(field, RenameAll::Pascal);
            pascal[..1].to_ascii_lowercase() + &pascal[1..]
        }
        RenameAll::Pascal => {
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
        RenameAll::Snake => field.to_string(),
        RenameAll::ScreamingSnake => field.to_string().to_ascii_uppercase(),
        RenameAll::Kebab => field.to_string().replace('_', "-"),
        RenameAll::ScreamingKebab => {
            transform_field_casing(field, RenameAll::ScreamingSnake).replace('_', "-")
        }
    }
}
