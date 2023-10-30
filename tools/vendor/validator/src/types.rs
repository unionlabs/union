use std::borrow::Cow;
use std::collections::{hash_map::Entry::Vacant, BTreeMap, HashMap};

use serde::ser::Serialize;
use serde_derive::{Deserialize, Serialize};
use serde_json::{to_value, Value};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub code: Cow<'static, str>,
    pub message: Option<Cow<'static, str>>,
    pub params: HashMap<Cow<'static, str>, Value>,
}

impl ValidationError {
    pub fn new(code: &'static str) -> ValidationError {
        ValidationError { code: Cow::from(code), message: None, params: HashMap::new() }
    }

    pub fn add_param<T: Serialize>(&mut self, name: Cow<'static, str>, val: &T) {
        self.params.insert(name, to_value(val).unwrap());
    }
}

impl std::error::Error for ValidationError {
    fn description(&self) -> &str {
        &self.code
    }
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

#[derive(Debug, Serialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum ValidationErrorsKind {
    Struct(Box<ValidationErrors>),
    List(BTreeMap<usize, Box<ValidationErrors>>),
    Field(Vec<ValidationError>),
}

#[derive(Default, Debug, Serialize, Clone, PartialEq)]
pub struct ValidationErrors(HashMap<&'static str, ValidationErrorsKind>);

impl ValidationErrors {
    pub fn new() -> ValidationErrors {
        ValidationErrors(HashMap::new())
    }

    /// Returns a boolean indicating whether a validation result includes validation errors for a
    /// given field. May be used as a condition for performing nested struct validations on a field
    /// in the absence of field-level validation errors.
    #[must_use]
    pub fn has_error(result: &Result<(), ValidationErrors>, field: &'static str) -> bool {
        match result {
            Ok(()) => false,
            Err(ref errs) => errs.contains_key(field),
        }
    }

    /// Returns the combined outcome of a struct's validation result along with the nested
    /// validation result for one of its fields.
    pub fn merge(
        parent: Result<(), ValidationErrors>,
        field: &'static str,
        child: Result<(), ValidationErrors>,
    ) -> Result<(), ValidationErrors> {
        match child {
            Ok(()) => parent,
            Err(errors) => {
                parent.and_then(|_| Err(ValidationErrors::new())).map_err(|mut parent_errors| {
                    parent_errors.add_nested(field, ValidationErrorsKind::Struct(Box::new(errors)));
                    parent_errors
                })
            }
        }
    }

    /// Returns the combined outcome of a struct's validation result along with the nested
    /// validation result for one of its fields where that field is a vector of validating structs.
    pub fn merge_all(
        parent: Result<(), ValidationErrors>,
        field: &'static str,
        children: Vec<Result<(), ValidationErrors>>,
    ) -> Result<(), ValidationErrors> {
        let errors = children
            .into_iter()
            .enumerate()
            .filter_map(|(i, res)| res.err().map(|mut err| (i, err.remove(field))))
            .filter_map(|(i, entry)| match entry {
                Some(ValidationErrorsKind::Struct(errors)) => Some((i, errors)),
                _ => None,
            })
            .collect::<BTreeMap<_, _>>();

        if errors.is_empty() {
            parent
        } else {
            parent.and_then(|_| Err(ValidationErrors::new())).map_err(|mut parent_errors| {
                parent_errors.add_nested(field, ValidationErrorsKind::List(errors));
                parent_errors
            })
        }
    }

    /// Returns a map of field-level validation errors found for the struct that was validated and
    /// any of it's nested structs that are tagged for validation.
    pub fn errors(&self) -> &HashMap<&'static str, ValidationErrorsKind> {
        &self.0
    }

    /// Returns a mutable map of field-level validation errors found for the struct that was validated and
    /// any of it's nested structs that are tagged for validation.
    pub fn errors_mut(&mut self) -> &mut HashMap<&'static str, ValidationErrorsKind> {
        &mut self.0
    }

    /// Consume the struct, returning the validation errors found
    pub fn into_errors(self) -> HashMap<&'static str, ValidationErrorsKind> {
        self.0
    }

    /// Returns a map of only field-level validation errors found for the struct that was validated.
    pub fn field_errors(&self) -> HashMap<&'static str, &Vec<ValidationError>> {
        self.0
            .iter()
            .filter_map(|(k, v)| {
                if let ValidationErrorsKind::Field(errors) = v {
                    Some((*k, errors))
                } else {
                    None
                }
            })
            .collect::<HashMap<_, _>>()
    }

    pub fn add(&mut self, field: &'static str, error: ValidationError) {
        if let ValidationErrorsKind::Field(ref mut vec) =
            self.0.entry(field).or_insert_with(|| ValidationErrorsKind::Field(vec![]))
        {
            vec.push(error);
        } else {
            panic!("Attempt to add field validation to a non-Field ValidationErrorsKind instance");
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn add_nested(&mut self, field: &'static str, errors: ValidationErrorsKind) {
        if let Vacant(entry) = self.0.entry(field) {
            entry.insert(errors);
        } else {
            panic!("Attempt to replace non-empty ValidationErrors entry");
        }
    }

    #[must_use]
    fn contains_key(&self, field: &'static str) -> bool {
        self.0.contains_key(field)
    }

    fn remove(&mut self, field: &'static str) -> Option<ValidationErrorsKind> {
        self.0.remove(field)
    }
}

impl std::error::Error for ValidationErrors {
    fn description(&self) -> &str {
        "Validation failed"
    }
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}
