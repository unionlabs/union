#![recursion_limit = "128"]

use std::{collections::HashMap, unreachable};

use if_chain::if_chain;
use proc_macro2::Span;
use proc_macro_error::{abort, proc_macro_error};
use quote::ToTokens;
use quote::{quote, quote_spanned};
use syn::{parse_quote, spanned::Spanned, GenericParam, Lifetime, LifetimeDef, Type};

use asserts::{assert_has_len, assert_has_range, assert_string_type, assert_type_matches};
use lit::*;
use quoting::{quote_schema_validations, quote_validator, FieldQuoter};
use validation::*;
use validator_types::{CustomArgument, Validator};

use crate::asserts::assert_custom_arg_type;

mod asserts;
mod lit;
mod quoting;
mod validation;

#[proc_macro_derive(Validate, attributes(validate))]
#[proc_macro_error]
pub fn derive_validation(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_validate(&ast).into()
}

fn impl_validate(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    // Collecting the validators
    let mut fields_validations = collect_field_validations(ast);
    let mut struct_validations = find_struct_validations(&ast.attrs);
    let (arg_type, has_arg) =
        construct_validator_argument_type(&mut fields_validations, &mut struct_validations);
    let (validations, nested_validations) = quote_field_validations(fields_validations);

    let schema_validations = quote_schema_validations(&struct_validations);

    // Struct specific definitions
    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    // The Validate trait implementation
    let validate_trait_impl = if !has_arg {
        quote!(
            impl #impl_generics ::validator::Validate for #ident #ty_generics #where_clause {
                fn validate(&self) -> ::std::result::Result<(), ::validator::ValidationErrors> {
                    use ::validator::ValidateArgs;
                    self.validate_args(())
                }
            }
        )
    } else {
        quote!()
    };

    // Adding the validator lifetime 'v_a
    let mut expanded_generic = ast.generics.clone();
    expanded_generic
        .params
        .insert(0, GenericParam::Lifetime(LifetimeDef::new(Lifetime::new("'v_a", ast.span()))));

    let (impl_generics, _, _) = expanded_generic.split_for_impl();

    // Implementing ValidateArgs
    let impl_ast = quote!(
        #validate_trait_impl

        // We need this here to prevent formatting lints that can be caused by `quote_spanned!`
        // See: rust-lang/rust-clippy#6249 for more reference
        #[allow(clippy::all)]
        // Triggers when single_use_lifetimes rustc lint is configured in user project and there are no
        // usages of 'v_a lifetime in the generated impl definition
        #[allow(single_use_lifetimes)]
        impl #impl_generics ::validator::ValidateArgs<'v_a> for #ident #ty_generics #where_clause {
            type Args = #arg_type;

            #[allow(unused_mut)]
            #[allow(unused_variable)]
            fn validate_args(&self, args: Self::Args) -> ::std::result::Result<(), ::validator::ValidationErrors> {
                let mut errors = ::validator::ValidationErrors::new();

                #(#validations)*

                #(#schema_validations)*

                let mut result = if errors.is_empty() {
                    ::std::result::Result::Ok(())
                } else {
                    ::std::result::Result::Err(errors)
                };

                #(#nested_validations)*
                result
            }
        }
    );

    // println!("{}", impl_ast.to_string());

    impl_ast
}

fn collect_fields(ast: &syn::DeriveInput) -> Vec<syn::Field> {
    match ast.data {
        syn::Data::Struct(syn::DataStruct { ref fields, .. }) => {
            if fields.iter().any(|field| field.ident.is_none()) {
                abort!(
                    fields.span(),
                    "struct has unnamed fields";
                    help = "#[derive(Validate)] can only be used on structs with named fields";
                );
            }
            fields.iter().cloned().collect::<Vec<_>>()
        }
        _ => abort!(ast.span(), "#[derive(Validate)] can only be used with structs"),
    }
}

fn collect_field_validations(ast: &syn::DeriveInput) -> Vec<FieldInformation> {
    let mut fields = collect_fields(ast);

    let field_types = find_fields_type(&fields);
    fields.drain(..).fold(vec![], |mut acc, field| {
        let key = field.ident.clone().unwrap().to_string();
        let (name, validations) = find_validators_for_field(&field, &field_types);
        acc.push(FieldInformation::new(
            field,
            field_types.get(&key).unwrap().clone(),
            name,
            validations,
        ));
        acc
    })
}

fn construct_validator_argument_type(
    fields_validations: &mut [FieldInformation],
    struct_validations: &mut [SchemaValidation],
) -> (proc_macro2::TokenStream, bool) {
    const ARGS_PARAMETER_NAME: &str = "args";

    // This iterator only holds custom validations with a argument_type
    let mut customs: Vec<&mut CustomArgument> = fields_validations
        .iter_mut()
        .flat_map(|x| {
            x.validations.iter_mut().filter_map(|x| x.validator.get_custom_argument_mut())
        })
        .collect();

    let mut schemas: Vec<&mut CustomArgument> =
        struct_validations.iter_mut().filter_map(|x| x.args.as_mut()).collect();

    customs.append(&mut schemas);

    if customs.is_empty() {
        // Just the default empty type if no types are defined
        (quote!(()), false)
    } else if customs.len() == 1 {
        // A single parameter will not be wrapped in a tuple
        let arg = customs.pop().unwrap();
        arg.arg_access = Some(syn::parse_str(ARGS_PARAMETER_NAME).unwrap());

        let type_stream: &Type = &arg.arg_type;
        let span = arg.def_span;
        (quote_spanned!(span=> #type_stream), true)
    } else {
        // Multiple times will be wrapped in a tuple
        let mut index = 0;
        let params = customs.iter_mut().fold(quote!(), |acc, arg| {
            let arg_access_string = format!("{}.{}", ARGS_PARAMETER_NAME, index);
            arg.arg_access = Some(syn::parse_str(arg_access_string.as_str()).unwrap());
            index += 1;

            let type_stream: &Type = &arg.arg_type;
            let span = arg.def_span;

            if index == 1 {
                quote_spanned!(span=> #type_stream)
            } else {
                quote_spanned!(span=> #acc, #type_stream)
            }
        });

        (quote!((#params)), true)
    }
}

fn quote_field_validations(
    mut fields: Vec<FieldInformation>,
) -> (Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>) {
    let mut validations = vec![];
    let mut nested_validations = vec![];

    fields.drain(..).for_each(|x| {
        let field_ident = x.field.ident.clone().unwrap();
        let field_quoter = FieldQuoter::new(field_ident, x.name, x.field_type);

        for validation in &x.validations {
            quote_validator(&field_quoter, validation, &mut validations, &mut nested_validations);
        }
    });

    (validations, nested_validations)
}

/// Find if a struct has some schema validation and returns the info if so
fn find_struct_validation(attr: &syn::Attribute) -> SchemaValidation {
    let error = |span: Span, msg: &str| -> ! {
        abort!(span, "Invalid schema level validation: {}", msg);
    };

    if_chain! {
        if let Ok(syn::Meta::List(syn::MetaList { ref nested, .. })) = attr.parse_meta();
        if let syn::NestedMeta::Meta(syn::Meta::List(syn::MetaList { ref path, ref nested, .. })) = nested[0];

        then {
            let ident = path.get_ident().unwrap();
            if ident != "schema" {
                error(attr.span(), "Only `schema` is allowed as validator on a struct")
            }

            let mut function = String::new();
            let mut skip_on_field_errors = true;
            let mut code = None;
            let mut message = None;
            let mut args = None;

            for arg in nested {
                if_chain! {
                    if let syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue { ref path, ref lit, .. })) = *arg;

                    then {
                        let ident = path.get_ident().unwrap();
                        match ident.to_string().as_ref() {
                            "function" => {
                                function = match lit_to_string(lit) {
                                    Some(s) => s,
                                    None => error(lit.span(), "invalid argument type for `function` \
                                    : only a string is allowed"),
                                };
                            },
                            "skip_on_field_errors" => {
                                skip_on_field_errors = match lit_to_bool(lit) {
                                    Some(s) => s,
                                    None => error(lit.span(), "invalid argument type for `skip_on_field_errors` \
                                    : only a bool is allowed"),
                                };
                            },
                            "code" => {
                                code = match lit_to_string(lit) {
                                    Some(s) => Some(s),
                                    None => error(lit.span(), "invalid argument type for `code` \
                                    : only a string is allowed"),
                                };
                            },
                            "message" => {
                                message = match lit_to_string(lit) {
                                    Some(s) => Some(s),
                                    None => error(lit.span(), "invalid argument type for `message` \
                                    : only a string is allowed"),
                                };
                            },
                            "arg" => {
                                match lit_to_string(lit) {
                                    Some(s) => {
                                        match syn::parse_str::<syn::Type>(s.as_str()) {
                                            Ok(arg_type) => {
                                                assert_custom_arg_type(&lit.span(), &arg_type);
                                                args = Some(CustomArgument::new(lit.span(), arg_type));
                                            }
                                            Err(_) => {
                                                let mut msg = "invalid argument type for `arg` of `schema` validator: The string has to be a single type.".to_string();
                                                msg.push_str("\n(Tip: You can combine multiple types into one tuple.)");
                                                error(lit.span(), msg.as_str());
                                            }
                                        }
                                    }
                                    None => error(lit.span(), "invalid argument type for `arg` of `custom` validator: expected a string")
                                };
                            },
                            _ => error(lit.span(), "Unknown argument")
                        }
                    } else {
                        error(arg.span(), "Unexpected args")
                    }
                }
            }

            if function.is_empty() {
                error(path.span(), "`function` is required");
            }

            SchemaValidation {
                function,
                args,
                skip_on_field_errors,
                code,
                message,
            }
        } else {
            error(attr.span(), "Unexpected struct validator")
        }
    }
}

/// Finds all struct schema validations
fn find_struct_validations(struct_attrs: &[syn::Attribute]) -> Vec<SchemaValidation> {
    struct_attrs
        .iter()
        .filter(|attribute| attribute.path == parse_quote!(validate))
        .map(find_struct_validation)
        .collect()
}

/// Find the types (as string) for each field of the struct
/// Needed for the `must_match` filter
fn find_fields_type(fields: &[syn::Field]) -> HashMap<String, String> {
    let mut types = HashMap::new();

    for field in fields {
        let field_ident = field.ident.clone().unwrap().to_string();
        let field_type = match field.ty {
            syn::Type::Path(syn::TypePath { ref path, .. }) => {
                let mut tokens = proc_macro2::TokenStream::new();
                path.to_tokens(&mut tokens);
                tokens.to_string().replace(' ', "")
            }
            syn::Type::Reference(syn::TypeReference { ref lifetime, ref elem, .. }) => {
                let mut tokens = proc_macro2::TokenStream::new();
                elem.to_tokens(&mut tokens);
                let mut name = tokens.to_string().replace(' ', "");
                if lifetime.is_some() {
                    name.insert(0, '&')
                }
                name
            }
            syn::Type::Group(syn::TypeGroup { ref elem, .. }) => {
                let mut tokens = proc_macro2::TokenStream::new();
                elem.to_tokens(&mut tokens);
                tokens.to_string().replace(' ', "")
            }
            _ => {
                let mut field_type = proc_macro2::TokenStream::new();
                field.ty.to_tokens(&mut field_type);
                field_type.to_string().replace(' ', "")
            }
        };

        //println!("{:?}", field_type);
        types.insert(field_ident, field_type);
    }

    types
}

/// Find everything we need to know about a field: its real name if it's changed from the serialization
/// and the list of validators to run on it
fn find_validators_for_field(
    field: &syn::Field,
    field_types: &HashMap<String, String>,
) -> (String, Vec<FieldValidation>) {
    let rust_ident = field.ident.clone().unwrap().to_string();
    let mut field_ident = field.ident.clone().unwrap().to_string();

    let error = |span: Span, msg: &str| -> ! {
        abort!(
            span,
            "Invalid attribute #[validate] on field `{}`: {}",
            field.ident.clone().unwrap().to_string(),
            msg
        );
    };

    let field_type = field_types.get(&field_ident).unwrap();

    let mut validators = vec![];
    let mut has_validate = false;

    for attr in &field.attrs {
        if attr.path != parse_quote!(validate) && attr.path != parse_quote!(serde) {
            continue;
        }

        if attr.path == parse_quote!(validate) {
            has_validate = true;
        }

        match attr.parse_meta() {
            Ok(syn::Meta::List(syn::MetaList { ref nested, .. })) => {
                let meta_items = nested.iter().collect::<Vec<_>>();
                // original name before serde rename
                if attr.path == parse_quote!(serde) {
                    if let Some(s) = find_original_field_name(&meta_items) {
                        field_ident = s;
                    }
                    continue;
                }

                // only validation from there on
                for meta_item in meta_items {
                    match *meta_item {
                        syn::NestedMeta::Meta(ref item) => match *item {
                            // email, url, phone, credit_card, non_control_character
                            syn::Meta::Path(ref name) => {
                                match name.get_ident().unwrap().to_string().as_ref() {
                                    "email" => {
                                        assert_string_type("email", field_type, &field.ty);
                                        validators.push(FieldValidation::new(Validator::Email));
                                    }
                                    "url" => {
                                        assert_string_type("url", field_type, &field.ty);
                                        validators.push(FieldValidation::new(Validator::Url));
                                    }
                                    #[cfg(feature = "phone")]
                                    "phone" => {
                                        assert_string_type("phone", field_type, &field.ty);
                                        validators.push(FieldValidation::new(Validator::Phone));
                                    }
                                    #[cfg(feature = "card")]
                                    "credit_card" => {
                                        assert_string_type("credit_card", field_type, &field.ty);
                                        validators
                                            .push(FieldValidation::new(Validator::CreditCard));
                                    }
                                    #[cfg(feature = "unic")]
                                    "non_control_character" => {
                                        assert_string_type(
                                            "non_control_character",
                                            field_type,
                                            &field.ty,
                                        );
                                        validators.push(FieldValidation::new(
                                            Validator::NonControlCharacter,
                                        ));
                                    }
                                    "required" => {
                                        validators.push(FieldValidation::new(Validator::Required));
                                    }
                                    "required_nested" => {
                                        validators.push(FieldValidation::new(Validator::Required));
                                        validators.push(FieldValidation::new(Validator::Nested));
                                    }
                                    _ => {
                                        let mut ident = proc_macro2::TokenStream::new();
                                        name.to_tokens(&mut ident);
                                        abort!(name.span(), "Unexpected validator: {}", ident)
                                    }
                                }
                            }
                            // custom, contains, must_match, regex
                            syn::Meta::NameValue(syn::MetaNameValue {
                                ref path, ref lit, ..
                            }) => {
                                let ident = path.get_ident().unwrap();
                                match ident.to_string().as_ref() {
                                    "custom" => {
                                        match lit_to_string(lit) {
                                            Some(s) => validators.push(FieldValidation::new(Validator::Custom {
                                                function: s,
                                                argument: Box::new(None),
                                            })),
                                            None => error(lit.span(), "invalid argument for `custom` validator: only strings are allowed"),
                                        };
                                    }
                                    "contains" => {
                                        match lit_to_string(lit) {
                                            Some(s) => validators.push(FieldValidation::new(Validator::Contains(s))),
                                            None => error(lit.span(), "invalid argument for `contains` validator: only strings are allowed"),
                                        };
                                    }
                                    "does_not_contain" => {
                                        match lit_to_string(lit) {
                                            Some(s) => validators.push(FieldValidation::new(Validator::DoesNotContain(s))),
                                            None => error(lit.span(), "invalid argument for `does_not_contain` validator: only strings are allowed"),
                                        };
                                    }
                                    "regex" => {
                                        match lit_to_string(lit) {
                                            Some(s) => validators.push(FieldValidation::new(Validator::Regex(s))),
                                            None => error(lit.span(), "invalid argument for `regex` validator: only strings are allowed"),
                                        };
                                    }
                                    "must_match" => {
                                        match lit_to_string(lit) {
                                            Some(s) => {
                                                assert_type_matches(rust_ident.clone(), field_type, field_types.get(&s), attr);
                                                validators.push(FieldValidation::new(Validator::MustMatch(s)));
                                            }
                                            None => error(lit.span(), "invalid argument for `must_match` validator: only strings are allowed"),
                                        };
                                    }
                                    v => abort!(
                                        path.span(),
                                        "unexpected name value validator: {:?}",
                                        v
                                    ),
                                };
                            }
                            // Validators with several args
                            syn::Meta::List(syn::MetaList { ref path, ref nested, .. }) => {
                                let meta_items = nested.iter().cloned().collect::<Vec<_>>();
                                let ident = path.get_ident().unwrap();
                                match ident.to_string().as_ref() {
                                    "length" => {
                                        assert_has_len(rust_ident.clone(), field_type, &field.ty);
                                        validators.push(extract_length_validation(
                                            rust_ident.clone(),
                                            attr,
                                            &meta_items,
                                        ));
                                    }
                                    "range" => {
                                        assert_has_range(rust_ident.clone(), field_type, &field.ty);
                                        validators.push(extract_range_validation(
                                            rust_ident.clone(),
                                            attr,
                                            &meta_items,
                                        ));
                                    }
                                    "custom" => {
                                        validators.push(extract_custom_validation(
                                            rust_ident.clone(),
                                            attr,
                                            &meta_items,
                                        ));
                                    }
                                    "email"
                                    | "url"
                                    | "phone"
                                    | "credit_card"
                                    | "non_control_character"
                                    | "required" => {
                                        validators.push(extract_argless_validation(
                                            ident.to_string(),
                                            rust_ident.clone(),
                                            &meta_items,
                                        ));
                                    }
                                    "contains" | "does_not_contain" => {
                                        validators.push(extract_one_arg_validation(
                                            "pattern",
                                            ident.to_string(),
                                            rust_ident.clone(),
                                            &meta_items,
                                        ));
                                    }
                                    "regex" => {
                                        validators.push(extract_one_arg_validation(
                                            "path",
                                            ident.to_string(),
                                            rust_ident.clone(),
                                            &meta_items,
                                        ));
                                    }
                                    "must_match" => {
                                        let validation = extract_one_arg_validation(
                                            "other",
                                            ident.to_string(),
                                            rust_ident.clone(),
                                            &meta_items,
                                        );
                                        if let Validator::MustMatch(ref t2) = validation.validator {
                                            assert_type_matches(
                                                rust_ident.clone(),
                                                field_type,
                                                field_types.get(t2),
                                                attr,
                                            );
                                        }
                                        validators.push(validation);
                                    }
                                    v => abort!(path.span(), "unexpected list validator: {:?}", v),
                                }
                            }
                        },
                        _ => unreachable!("Found a non Meta while looking for validators"),
                    };
                }
            }
            Ok(syn::Meta::Path(_)) => validators.push(FieldValidation::new(Validator::Nested)),
            Ok(syn::Meta::NameValue(_)) => abort!(attr.span(), "Unexpected name=value argument"),
            Err(e) => {
                let error_string = format!("{:?}", e);
                if error_string == "Error(\"expected literal\")" {
                    abort!(attr.span(),
                        "This attributes for the field `{}` seem to be misformed, please validate the syntax with the documentation",
                        field_ident
                    );
                } else {
                    abort!(
                        attr.span(),
                        "Unable to parse this attribute for the field `{}` with the error: {:?}",
                        field_ident,
                        e
                    );
                }
            }
        }

        if has_validate && validators.is_empty() {
            error(attr.span(), "it needs at least one validator");
        }
    }

    (field_ident, validators)
}

/// Serde can be used to rename fields on deserialization but most of the times
/// we want the error on the original field.
///
/// For example a JS frontend might send camelCase fields and Rust converts them to snake_case
/// but we want to send the errors back with the original name
fn find_original_field_name(meta_items: &[&syn::NestedMeta]) -> Option<String> {
    let mut original_name = None;

    for meta_item in meta_items {
        match **meta_item {
            syn::NestedMeta::Meta(ref item) => match *item {
                syn::Meta::Path(_) => continue,
                syn::Meta::NameValue(syn::MetaNameValue { ref path, ref lit, .. }) => {
                    let ident = path.get_ident().unwrap();
                    if ident == "rename" {
                        original_name = Some(lit_to_string(lit).unwrap());
                    }
                }
                syn::Meta::List(syn::MetaList { ref nested, .. }) => {
                    return find_original_field_name(&nested.iter().collect::<Vec<_>>());
                }
            },
            _ => unreachable!(),
        };

        if original_name.is_some() {
            return original_name;
        }
    }

    original_name
}
