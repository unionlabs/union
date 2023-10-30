use proc_macro2::Span;
use regex::Regex;

use lazy_static::lazy_static;
use proc_macro_error::abort;
use syn::spanned::Spanned;

lazy_static! {
    pub static ref COW_TYPE: Regex = Regex::new(r"Cow<'[a-z]+,str>").unwrap();
    pub static ref LEN_TYPE: Regex =
        Regex::new(r"(Option<)?((Vec|HashMap|HashSet|BTreeMap|BTreeSet|IndexMap|IndexSet)<|\[)")
            .unwrap();
}

static CUSTOM_ARG_LIFETIME: &str = "v_a";

static CUSTOM_ARG_ALLOWED_COPY_TYPES: [&str; 14] = [
    "usize", "u8", "u16", "u32", "u64", "u128", "isize", "i8", "i16", "i32", "i64", "i128", "f32",
    "f64",
];

pub static NUMBER_TYPES: [&str; 38] = [
    "usize",
    "u8",
    "u16",
    "u32",
    "u64",
    "u128",
    "isize",
    "i8",
    "i16",
    "i32",
    "i64",
    "i128",
    "f32",
    "f64",
    "Option<usize>",
    "Option<u8>",
    "Option<u16>",
    "Option<u32>",
    "Option<u64>",
    "Option<isize>",
    "Option<i8>",
    "Option<i16>",
    "Option<i32>",
    "Option<i64>",
    "Option<f32>",
    "Option<f64>",
    "Option<Option<usize>>",
    "Option<Option<u8>>",
    "Option<Option<u16>>",
    "Option<Option<u32>>",
    "Option<Option<u64>>",
    "Option<Option<isize>>",
    "Option<Option<i8>>",
    "Option<Option<i16>>",
    "Option<Option<i32>>",
    "Option<Option<i64>>",
    "Option<Option<f32>>",
    "Option<Option<f64>>",
];

pub fn assert_string_type(name: &str, type_name: &str, field_type: &syn::Type) {
    if !type_name.contains("String") && !type_name.contains("str") {
        abort!(
            field_type.span(),
            "`{}` validator can only be used on String, &str, Cow<'_,str> or an Option of those",
            name
        );
    }
}

pub fn assert_type_matches(
    field_name: String,
    field_type: &str,
    field_type2: Option<&String>,
    field_attr: &syn::Attribute,
) {
    if let Some(t2) = field_type2 {
        if field_type != t2 {
            abort!(field_attr.span(), "Invalid argument for `must_match` validator of field `{}`: types of field can't match", field_name);
        }
    } else {
        abort!(field_attr.span(), "Invalid argument for `must_match` validator of field `{}`: the other field doesn't exist in struct", field_name);
    }
}

pub fn assert_has_len(field_name: String, type_name: &str, field_type: &syn::Type) {
    if let syn::Type::Reference(ref tref) = field_type {
        let elem = &tref.elem;
        let type_name = format!("{}", quote::quote! { #elem }).replace(' ', "");

        if type_name == "str" {
            return;
        }
        assert_has_len(field_name, &type_name, elem);
        return;
    }

    if !type_name.contains("String")
        && !type_name.contains("str")
        && !LEN_TYPE.is_match(type_name)
        // a bit ugly
        && !COW_TYPE.is_match(type_name)
    {
        abort!(field_type.span(),
                "Validator `length` can only be used on types `String`, `&str`, Cow<'_,str>, `Vec`, slice, or map/set types (BTree/Hash/Index) but found `{}` for field `{}`",
                type_name, field_name
            );
    }
}

pub fn assert_has_range(field_name: String, type_name: &str, field_type: &syn::Type) {
    if !NUMBER_TYPES.contains(&type_name) {
        abort!(
            field_type.span(),
            "Validator `range` can only be used on number types but found `{}` for field `{}`",
            type_name,
            field_name
        );
    }
}

pub fn assert_custom_arg_type(field_span: &Span, field_type: &syn::Type) {
    match field_type {
        syn::Type::Reference(reference) => {
            if let Some(lifetime) = &reference.lifetime {
                let lifetime_ident = lifetime.ident.to_string();
                if lifetime_ident != CUSTOM_ARG_LIFETIME {
                    abort!(
                        field_span,
                        "Invalid argument reference: The lifetime `'{}` is not supported. Please use the validator lifetime `'{}`",
                        lifetime_ident,
                        CUSTOM_ARG_LIFETIME
                    );
                }
            } else {
                abort!(
                    field_span,
                    "Invalid argument reference: All references need to use the validator lifetime `'{}`",
                    CUSTOM_ARG_LIFETIME
                );
            }
        }
        // trigger nested validation
        syn::Type::Paren(paren) => {
            assert_custom_arg_type(field_span, &paren.elem);
        }
        syn::Type::Tuple(tuple) => {
            tuple.elems.iter().for_each(|x| assert_custom_arg_type(field_span, x));
        }
        // assert idents
        syn::Type::Path(path) => {
            let segments = &path.path.segments;
            if segments.len() == 1 {
                let ident = &segments.first().unwrap().ident.to_string();
                if CUSTOM_ARG_ALLOWED_COPY_TYPES.contains(&ident.as_str()) {
                    // A known copy type that can be passed without a reference
                    return;
                }
            }

            abort!(
                field_span,
                "Invalid argument type: All types except numbers and tuples need be passed by reference using the lifetime `'{}`",
                CUSTOM_ARG_LIFETIME,
            );
        }
        // Not allows
        _ => {
            abort!(
                field_span,
                "Invalid argument type: Custom arguments only allow tuples, number types and references using the lifetime `'{}` ",
                CUSTOM_ARG_LIFETIME,
            );
        }
    }
}
