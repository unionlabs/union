use proc_macro2::Span;
use syn::{Expr, Type};

/// Contains all the validators that can be used
///
/// In this crate as it's not allowed to export more than the proc macro
/// in a proc macro crate
#[derive(Debug, Clone)]
pub enum Validator {
    Email,
    Url,
    Custom {
        /// This is the name of the function that should be called
        function: String,
        /// This is the argument type that can be passed in with a macro
        argument: Box<Option<CustomArgument>>,
    },
    // String is the name of the field to match
    MustMatch(String),
    // value is a &str or a HashMap<String, ..>
    Contains(String),
    // No implementation in this crate, it's all in validator_derive
    Regex(String),
    Range {
        min: Option<ValueOrPath<f64>>,
        max: Option<ValueOrPath<f64>>,
    },
    // Any value that impl HasLen can be validated with Length
    Length {
        min: Option<ValueOrPath<u64>>,
        max: Option<ValueOrPath<u64>>,
        equal: Option<ValueOrPath<u64>>,
    },
    #[cfg(feature = "card")]
    CreditCard,
    #[cfg(feature = "phone")]
    Phone,
    Nested,
    #[cfg(feature = "unic")]
    NonControlCharacter,
    Required,
    RequiredNested,
    DoesNotContain(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueOrPath<T: std::fmt::Debug + Clone + PartialEq> {
    Value(T),
    Path(String),
}

/// This struct stores information about defined custom arguments that will be passed in
/// by the user in the validation step.
#[derive(Debug, Clone)]
pub struct CustomArgument {
    /// The span of type definition, this can be used in combination with `quote_spanned!` for
    /// better error reporting
    pub def_span: Span,
    /// The type of the argument. This can use `'v_a` as a lifetime but has to be Sized. This
    /// means that the type size has to be known at compile time
    pub arg_type: Type,
    /// This is the way we can access the value from the provided arguments. This will usually
    /// look something like `args.0`.
    pub arg_access: Option<Expr>,
}

impl CustomArgument {
    pub fn new(def_span: Span, arg_type: Type) -> Self {
        CustomArgument { def_span, arg_type, arg_access: None }
    }
}

impl Validator {
    pub fn code(&self) -> &'static str {
        match *self {
            Validator::MustMatch(_) => "must_match",
            Validator::Email => "email",
            Validator::Url => "url",
            Validator::Custom { .. } => "custom",
            Validator::Contains(_) => "contains",
            Validator::Regex(_) => "regex",
            Validator::Range { .. } => "range",
            Validator::Length { .. } => "length",
            #[cfg(feature = "card")]
            Validator::CreditCard => "credit_card",
            #[cfg(feature = "phone")]
            Validator::Phone => "phone",
            Validator::Nested => "nested",
            #[cfg(feature = "unic")]
            Validator::NonControlCharacter => "non_control_character",
            Validator::Required => "required",
            Validator::RequiredNested => "required_nested",
            Validator::DoesNotContain(_) => "does_not_contain",
        }
    }

    /// This returns the defined custom argument if it was defined
    pub fn get_custom_argument(&self) -> Option<&CustomArgument> {
        match self {
            Validator::Custom { argument, .. } => (**argument).as_ref(),
            _ => None,
        }
    }

    /// This returns the defined custom argument if it was defined
    pub fn get_custom_argument_mut(&mut self) -> Option<&mut CustomArgument> {
        match self {
            Validator::Custom { argument, .. } => (**argument).as_mut(),
            _ => None,
        }
    }

    pub fn has_custom_argument(&self) -> bool {
        self.get_custom_argument().is_some()
    }
}
