use if_chain::if_chain;
use proc_macro2::{self, Span};
use quote::quote;

use validator_types::Validator;

use crate::asserts::{COW_TYPE, NUMBER_TYPES};
use crate::lit::{option_to_tokens, value_or_path_to_tokens};
use crate::validation::{FieldValidation, SchemaValidation};

/// Pass around all the information needed for creating a validation
#[derive(Debug)]
pub struct FieldQuoter {
    ident: syn::Ident,
    /// The field name
    name: String,
    /// The field type
    _type: String,
}

impl FieldQuoter {
    pub fn new(ident: syn::Ident, name: String, _type: String) -> FieldQuoter {
        FieldQuoter { ident, name, _type }
    }

    /// Don't put a & in front a pointer since we are going to pass
    /// a reference to the validator
    /// Also just use the ident without if it's optional and will go through
    /// a if let first
    pub fn quote_validator_param(&self) -> proc_macro2::TokenStream {
        let ident = &self.ident;

        if self._type.starts_with("Option<") {
            quote!(#ident)
        } else if COW_TYPE.is_match(self._type.as_ref()) {
            quote!(self.#ident.as_ref())
        } else if self._type.starts_with('&') || NUMBER_TYPES.contains(&self._type.as_ref()) {
            quote!(self.#ident)
        } else {
            quote!(&self.#ident)
        }
    }

    pub fn quote_validator_field(&self) -> proc_macro2::TokenStream {
        let ident = &self.ident;

        if self._type.starts_with("Option<") || is_list(&self._type) || is_map(&self._type) {
            quote!(#ident)
        } else if COW_TYPE.is_match(self._type.as_ref()) {
            quote!(self.#ident.as_ref())
        } else {
            quote!(self.#ident)
        }
    }

    pub fn get_optional_validator_param(&self) -> proc_macro2::TokenStream {
        let ident = &self.ident;
        if self._type.starts_with("Option<&")
            || self._type.starts_with("Option<Option<&")
            || NUMBER_TYPES.contains(&self._type.as_ref())
        {
            quote!(#ident)
        } else {
            quote!(ref #ident)
        }
    }

    /// Wrap the quoted output of a validation with a if let Some if
    /// the field type is an option
    pub fn wrap_if_option(&self, tokens: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        let field_ident = &self.ident;
        let optional_pattern_matched = self.get_optional_validator_param();
        if self._type.starts_with("Option<Option<") {
            return quote!(
                if let Some(Some(#optional_pattern_matched)) = self.#field_ident {
                    #tokens
                }
            );
        } else if self._type.starts_with("Option<") {
            return quote!(
                if let Some(#optional_pattern_matched) = self.#field_ident {
                    #tokens
                }
            );
        }

        tokens
    }

    /// Wrap the quoted output of a validation with a for loop if
    /// the field type is a vector
    pub fn wrap_if_collection(&self, tokens: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        let field_ident = &self.ident;
        let field_name = &self.name;

        // When we're using an option, we'll have the field unwrapped, so we should not access it
        // through `self`.
        let prefix = (!self._type.starts_with("Option<")).then(|| quote! { self. });

        // When iterating over a list, the iterator has Item=T, while a map yields Item=(K, V), and
        // we're only interested in V.
        let args = if is_list(&self._type) {
            quote! { #field_ident }
        } else if is_map(&self._type) {
            quote! { (_, #field_ident) }
        } else {
            return tokens;
        };

        quote! {
            if !::validator::ValidationErrors::has_error(&result, #field_name) {
                let results: Vec<_> = #prefix #field_ident.iter().map(|#args| {
                    let mut result = ::std::result::Result::Ok(());
                    #tokens
                    result
                }).collect();
                result = ::validator::ValidationErrors::merge_all(result, #field_name, results);
            }
        }
    }
}

fn is_map(_type: &str) -> bool {
    if let Some(stripped) = _type.strip_prefix("Option<") {
        is_map(stripped)
    } else if let Some(stripped) = _type.strip_prefix("&") {
        is_map(stripped)
    } else {
        _type.starts_with("HashMap<")
            || _type.starts_with("FxHashMap<")
            || _type.starts_with("FnvHashMap<")
            || _type.starts_with("BTreeMap<")
            || _type.starts_with("IndexMap<")
    }
}

fn is_list(_type: &str) -> bool {
    if let Some(stripped) = _type.strip_prefix("&") {
        is_list(stripped)
    } else if let Some(stripped) = _type.strip_prefix("Option<") {
        is_list(stripped)
    } else {
        _type.starts_with("Vec<")
            || _type.starts_with("HashSet<")
            || _type.starts_with("BTreeSet<")
            || _type.starts_with("IndexSet<")
            || _type.starts_with("[")
    }
}

/// Quote an actual end-user error creation automatically
fn quote_error(validation: &FieldValidation) -> proc_macro2::TokenStream {
    let code = &validation.code;
    let add_message_quoted = if let Some(ref m) = validation.message {
        quote!(err.message = Some(::std::borrow::Cow::from(#m));)
    } else {
        quote!()
    };

    quote!(
        let mut err = ::validator::ValidationError::new(#code);
        #add_message_quoted
    )
}

pub fn quote_length_validation(
    field_quoter: &FieldQuoter,
    validation: &FieldValidation,
) -> proc_macro2::TokenStream {
    let field_name = &field_quoter.name;
    let validator_param = field_quoter.quote_validator_param();

    if let Validator::Length { min, max, equal } = &validation.validator {
        let min_err_param_quoted = if let Some(v) = min {
            let v = value_or_path_to_tokens(v);
            quote!(err.add_param(::std::borrow::Cow::from("min"), &#v);)
        } else {
            quote!()
        };
        let max_err_param_quoted = if let Some(v) = max {
            let v = value_or_path_to_tokens(v);
            quote!(err.add_param(::std::borrow::Cow::from("max"), &#v);)
        } else {
            quote!()
        };
        let equal_err_param_quoted = if let Some(v) = equal {
            let v = value_or_path_to_tokens(v);
            quote!(err.add_param(::std::borrow::Cow::from("equal"), &#v);)
        } else {
            quote!()
        };

        let min_tokens = option_to_tokens(
            &min.clone().as_ref().map(value_or_path_to_tokens).map(|x| quote!(#x as u64)),
        );
        let max_tokens = option_to_tokens(
            &max.clone().as_ref().map(value_or_path_to_tokens).map(|x| quote!(#x as u64)),
        );
        let equal_tokens = option_to_tokens(
            &equal.clone().as_ref().map(value_or_path_to_tokens).map(|x| quote!(#x as u64)),
        );

        let quoted_error = quote_error(validation);
        let quoted = quote!(
            if !::validator::validate_length(
                #validator_param,
                #min_tokens,
                #max_tokens,
                #equal_tokens
            ) {
                #quoted_error
                #min_err_param_quoted
                #max_err_param_quoted
                #equal_err_param_quoted
                err.add_param(::std::borrow::Cow::from("value"), &#validator_param);
                errors.add(#field_name, err);
            }
        );

        return field_quoter.wrap_if_option(quoted);
    }

    unreachable!()
}

pub fn quote_range_validation(
    field_quoter: &FieldQuoter,
    validation: &FieldValidation,
) -> proc_macro2::TokenStream {
    let field_name = &field_quoter.name;
    let quoted_ident = field_quoter.quote_validator_param();

    if let Validator::Range { ref min, ref max } = validation.validator {
        let min_err_param_quoted = if let Some(v) = min {
            let v = value_or_path_to_tokens(v);
            quote!(err.add_param(::std::borrow::Cow::from("min"), &#v);)
        } else {
            quote!()
        };
        let max_err_param_quoted = if let Some(v) = max {
            let v = value_or_path_to_tokens(v);
            quote!(err.add_param(::std::borrow::Cow::from("max"), &#v);)
        } else {
            quote!()
        };

        // Can't interpolate None
        let min_tokens =
            min.clone().map(|x| value_or_path_to_tokens(&x)).map(|x| quote!(#x as f64));
        let min_tokens = option_to_tokens(&min_tokens);

        let max_tokens =
            max.clone().map(|x| value_or_path_to_tokens(&x)).map(|x| quote!(#x as f64));
        let max_tokens = option_to_tokens(&max_tokens);

        let quoted_error = quote_error(validation);
        let quoted = quote!(
            if !::validator::validate_range(
                #quoted_ident as f64,
                #min_tokens,
                #max_tokens
            ) {
                #quoted_error
                #min_err_param_quoted
                #max_err_param_quoted
                err.add_param(::std::borrow::Cow::from("value"), &#quoted_ident);
                errors.add(#field_name, err);
            }
        );

        return field_quoter.wrap_if_option(quoted);
    }

    unreachable!()
}

#[cfg(feature = "card")]
pub fn quote_credit_card_validation(
    field_quoter: &FieldQuoter,
    validation: &FieldValidation,
) -> proc_macro2::TokenStream {
    let field_name = &field_quoter.name;
    let validator_param = field_quoter.quote_validator_param();

    let quoted_error = quote_error(validation);
    let quoted = quote!(
        if !::validator::validate_credit_card(#validator_param) {
            #quoted_error
            err.add_param(::std::borrow::Cow::from("value"), &#validator_param);
            errors.add(#field_name, err);
        }
    );

    field_quoter.wrap_if_option(quoted)
}

#[cfg(feature = "phone")]
pub fn quote_phone_validation(
    field_quoter: &FieldQuoter,
    validation: &FieldValidation,
) -> proc_macro2::TokenStream {
    let field_name = &field_quoter.name;
    let validator_param = field_quoter.quote_validator_param();

    let quoted_error = quote_error(validation);
    let quoted = quote!(
        if !::validator::validate_phone(#validator_param) {
            #quoted_error
            err.add_param(::std::borrow::Cow::from("value"), &#validator_param);
            errors.add(#field_name, err);
        }
    );

    field_quoter.wrap_if_option(quoted)
}

#[cfg(feature = "unic")]
pub fn quote_non_control_character_validation(
    field_quoter: &FieldQuoter,
    validation: &FieldValidation,
) -> proc_macro2::TokenStream {
    let field_name = &field_quoter.name;
    let validator_param = field_quoter.quote_validator_param();

    let quoted_error = quote_error(validation);
    let quoted = quote!(
        if !::validator::validate_non_control_character(#validator_param) {
            #quoted_error
            err.add_param(::std::borrow::Cow::from("value"), &#validator_param);
            errors.add(#field_name, err);
        }
    );

    field_quoter.wrap_if_option(quoted)
}

pub fn quote_url_validation(
    field_quoter: &FieldQuoter,
    validation: &FieldValidation,
) -> proc_macro2::TokenStream {
    let field_name = &field_quoter.name;
    let validator_param = field_quoter.quote_validator_param();

    let quoted_error = quote_error(validation);
    let quoted = quote!(
        if !::validator::validate_url(#validator_param) {
            #quoted_error
            err.add_param(::std::borrow::Cow::from("value"), &#validator_param);
            errors.add(#field_name, err);
        }
    );

    field_quoter.wrap_if_option(quoted)
}

pub fn quote_email_validation(
    field_quoter: &FieldQuoter,
    validation: &FieldValidation,
) -> proc_macro2::TokenStream {
    let field_name = &field_quoter.name;
    let validator_param = field_quoter.quote_validator_param();

    let quoted_error = quote_error(validation);
    let quoted = quote!(
        if !::validator::validate_email(#validator_param) {
            #quoted_error
            err.add_param(::std::borrow::Cow::from("value"), &#validator_param);
            errors.add(#field_name, err);
        }
    );

    field_quoter.wrap_if_option(quoted)
}

pub fn quote_must_match_validation(
    field_quoter: &FieldQuoter,
    validation: &FieldValidation,
) -> proc_macro2::TokenStream {
    let ident = &field_quoter.ident;
    let field_name = &field_quoter.name;

    if let Validator::MustMatch(ref other) = validation.validator {
        let other_ident = syn::Ident::new(other, Span::call_site());
        let quoted_error = quote_error(validation);
        let quoted = quote!(
            if !::validator::validate_must_match(&self.#ident, &self.#other_ident) {
                #quoted_error
                err.add_param(::std::borrow::Cow::from("value"), &self.#ident);
                err.add_param(::std::borrow::Cow::from("other"), &self.#other_ident);
                errors.add(#field_name, err);
            }
        );

        return field_quoter.wrap_if_option(quoted);
    }

    unreachable!();
}

pub fn quote_custom_validation(
    field_quoter: &FieldQuoter,
    validation: &FieldValidation,
) -> proc_macro2::TokenStream {
    let field_name = &field_quoter.name;
    let validator_param = field_quoter.quote_validator_param();

    if let Validator::Custom { function, argument, .. } = &validation.validator {
        let fn_ident: syn::Path = syn::parse_str(function).unwrap();

        let access = if_chain! {
            if let Some(argument) = &**argument;
            if let Some(access) = &argument.arg_access;
            then {
                quote!(, #access)
            } else {
                quote!()
            }
        };

        let add_message_quoted = if let Some(ref m) = validation.message {
            quote!(err.message = Some(::std::borrow::Cow::from(#m));)
        } else {
            quote!()
        };

        let quoted = quote!(
            match #fn_ident(#validator_param #access) {
                ::std::result::Result::Ok(()) => (),
                ::std::result::Result::Err(mut err) => {
                    #add_message_quoted
                    err.add_param(::std::borrow::Cow::from("value"), &#validator_param);
                    errors.add(#field_name, err);
                },
            };
        );

        return field_quoter.wrap_if_option(quoted);
    }

    unreachable!();
}

pub fn quote_contains_validation(
    field_quoter: &FieldQuoter,
    validation: &FieldValidation,
) -> proc_macro2::TokenStream {
    let field_name = &field_quoter.name;
    let validator_param = field_quoter.quote_validator_param();

    if let Validator::Contains(ref needle) = validation.validator {
        let quoted_error = quote_error(validation);
        let quoted = quote!(
            if !::validator::validate_contains(#validator_param, &#needle) {
                #quoted_error
                err.add_param(::std::borrow::Cow::from("value"), &#validator_param);
                err.add_param(::std::borrow::Cow::from("needle"), &#needle);
                errors.add(#field_name, err);
            }
        );

        return field_quoter.wrap_if_option(quoted);
    }

    unreachable!();
}

pub fn quote_regex_validation(
    field_quoter: &FieldQuoter,
    validation: &FieldValidation,
) -> proc_macro2::TokenStream {
    let field_name = &field_quoter.name;
    let validator_param = field_quoter.quote_validator_param();

    if let Validator::Regex(ref re) = validation.validator {
        let re_ident: syn::Path = syn::parse_str(re).unwrap();
        let quoted_error = quote_error(validation);
        let quoted = quote!(
            if !#re_ident.is_match(#validator_param) {
                #quoted_error
                err.add_param(::std::borrow::Cow::from("value"), &#validator_param);
                errors.add(#field_name, err);
            }
        );

        return field_quoter.wrap_if_option(quoted);
    }

    unreachable!();
}

pub fn quote_nested_validation(field_quoter: &FieldQuoter) -> proc_macro2::TokenStream {
    let field_name = &field_quoter.name;
    let validator_field = field_quoter.quote_validator_field();
    let quoted = quote!(result = ::validator::ValidationErrors::merge(result, #field_name, #validator_field.validate()););
    field_quoter.wrap_if_option(field_quoter.wrap_if_collection(quoted))
}

pub fn quote_validator(
    field_quoter: &FieldQuoter,
    validation: &FieldValidation,
    validations: &mut Vec<proc_macro2::TokenStream>,
    nested_validations: &mut Vec<proc_macro2::TokenStream>,
) {
    match validation.validator {
        Validator::Length { .. } => {
            validations.push(quote_length_validation(field_quoter, validation))
        }
        Validator::Range { .. } => {
            validations.push(quote_range_validation(field_quoter, validation))
        }
        Validator::Email => validations.push(quote_email_validation(field_quoter, validation)),
        Validator::Url => validations.push(quote_url_validation(field_quoter, validation)),
        Validator::MustMatch(_) => {
            validations.push(quote_must_match_validation(field_quoter, validation))
        }
        Validator::Custom { .. } => {
            validations.push(quote_custom_validation(field_quoter, validation))
        }
        Validator::Contains(_) => {
            validations.push(quote_contains_validation(field_quoter, validation))
        }
        Validator::Regex(_) => validations.push(quote_regex_validation(field_quoter, validation)),
        #[cfg(feature = "card")]
        Validator::CreditCard => {
            validations.push(quote_credit_card_validation(field_quoter, validation))
        }
        #[cfg(feature = "phone")]
        Validator::Phone => validations.push(quote_phone_validation(field_quoter, validation)),
        Validator::Nested => nested_validations.push(quote_nested_validation(field_quoter)),
        #[cfg(feature = "unic")]
        Validator::NonControlCharacter => {
            validations.push(quote_non_control_character_validation(field_quoter, validation))
        }
        Validator::Required | Validator::RequiredNested => {
            validations.push(quote_required_validation(field_quoter, validation))
        }
        Validator::DoesNotContain(_) => {
            validations.push(quote_does_not_contain_validation(field_quoter, validation))
        }
    }
}

pub fn quote_schema_validation(v: &SchemaValidation) -> proc_macro2::TokenStream {
    let fn_ident: syn::Path = syn::parse_str(&v.function).unwrap();

    let arg_quoted = if let Some(ref args) = v.args {
        let arg_type = &args.arg_access;
        quote!(self, #arg_type)
    } else {
        quote!(self)
    };

    let add_message_quoted = if let Some(ref m) = v.message {
        quote!(err.message = Some(::std::borrow::Cow::from(#m));)
    } else {
        quote!()
    };

    let mut_err_token = if v.message.is_some() { quote!(mut) } else { quote!() };

    let quoted = quote!(
        match #fn_ident(#arg_quoted) {
            ::std::result::Result::Ok(()) => (),
            ::std::result::Result::Err(#mut_err_token err) => {
                #add_message_quoted
                errors.add("__all__", err);
            },
        };
    );

    if !v.skip_on_field_errors {
        return quoted;
    }

    quote!(
        #quoted
    )
}

pub fn quote_schema_validations(validation: &[SchemaValidation]) -> Vec<proc_macro2::TokenStream> {
    validation.iter().map(quote_schema_validation).collect()
}

pub fn quote_required_validation(
    field_quoter: &FieldQuoter,
    validation: &FieldValidation,
) -> proc_macro2::TokenStream {
    let field_name = &field_quoter.name;
    let ident = &field_quoter.ident;
    let validator_param = quote!(&self.#ident);

    let quoted_error = quote_error(validation);
    let quoted = quote!(
        if !::validator::validate_required(#validator_param) {
            #quoted_error
            err.add_param(::std::borrow::Cow::from("value"), &#validator_param);
            errors.add(#field_name, err);
        }
    );

    quoted
}

pub fn quote_does_not_contain_validation(
    field_quoter: &FieldQuoter,
    validation: &FieldValidation,
) -> proc_macro2::TokenStream {
    let field_name = &field_quoter.name;
    let validator_param = field_quoter.quote_validator_param();

    if let Validator::DoesNotContain(ref needle) = validation.validator {
        let quoted_error = quote_error(validation);
        let quoted = quote!(
            if !::validator::validate_does_not_contain(#validator_param, &#needle) {
                #quoted_error
                err.add_param(::std::borrow::Cow::from("value"), &#validator_param);
                err.add_param(::std::borrow::Cow::from("needle"), &#needle);
                errors.add(#field_name, err);
            }
        );

        return field_quoter.wrap_if_option(quoted);
    }

    unreachable!();
}
