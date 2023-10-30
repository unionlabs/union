use crate::flatten::Merge;
use crate::gen::SchemaGenerator;
use crate::schema::{Metadata, Schema, SchemaObject};
use crate::JsonSchema;
use serde::Serialize;
use serde_json::Value;

// Helper for generating schemas for flattened `Option` fields.
pub fn json_schema_for_flatten<T: ?Sized + JsonSchema>(
    gen: &mut SchemaGenerator,
    required: bool,
) -> Schema {
    let mut schema = T::_schemars_private_non_optional_json_schema(gen);

    if T::_schemars_private_is_option() && !required {
        if let Schema::Object(SchemaObject {
            object: Some(ref mut object_validation),
            ..
        }) = schema
        {
            object_validation.required.clear();
        }
    }

    schema
}

pub fn apply_metadata(schema: Schema, metadata: Metadata) -> Schema {
    if metadata == Metadata::default() {
        schema
    } else {
        let mut schema_obj = schema.into_object();
        schema_obj.metadata = Some(Box::new(metadata)).merge(schema_obj.metadata);
        Schema::Object(schema_obj)
    }
}

/// Hack to simulate specialization:
/// `MaybeSerializeWrapper(x).maybe_to_value()` will resolve to either
/// - The inherent method `MaybeSerializeWrapper::maybe_to_value(...)` if x is `Serialize`
/// - The trait method `NoSerialize::maybe_to_value(...)` from the blanket impl otherwise
#[doc(hidden)]
#[macro_export]
macro_rules! _schemars_maybe_to_value {
    ($expression:expr) => {{
        #[allow(unused_imports)]
        use $crate::_private::{MaybeSerializeWrapper, NoSerialize as _};

        MaybeSerializeWrapper($expression).maybe_to_value()
    }};
}

pub struct MaybeSerializeWrapper<T>(pub T);

pub trait NoSerialize: Sized {
    fn maybe_to_value(self) -> Option<Value> {
        None
    }
}

impl<T> NoSerialize for T {}

impl<T: Serialize> MaybeSerializeWrapper<T> {
    pub fn maybe_to_value(self) -> Option<Value> {
        serde_json::value::to_value(self.0).ok()
    }
}
