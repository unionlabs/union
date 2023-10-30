#![forbid(unsafe_code)]
/*!
Generate JSON Schema documents from Rust code

## Basic Usage

If you don't really care about the specifics, the easiest way to generate a JSON schema for your types is to `#[derive(JsonSchema)]` and use the `schema_for!` macro. All fields of the type must also implement `JsonSchema` - Schemars implements this for many standard library types.

```rust
use schemars::{schema_for, JsonSchema};

#[derive(JsonSchema)]
pub struct MyStruct {
    pub my_int: i32,
    pub my_bool: bool,
    pub my_nullable_enum: Option<MyEnum>,
}

#[derive(JsonSchema)]
pub enum MyEnum {
    StringNewType(String),
    StructVariant { floats: Vec<f32> },
}

let schema = schema_for!(MyStruct);
println!("{}", serde_json::to_string_pretty(&schema).unwrap());
```

<details>
<summary>Click to see the output JSON schema...</summary>

```json
{
    "$schema": "http://json-schema.org/draft-07/schema#",
    "title": "MyStruct",
    "type": "object",
    "required": [
        "my_bool",
        "my_int"
    ],
    "properties": {
        "my_bool": {
            "type": "boolean"
        },
        "my_int": {
            "type": "integer",
            "format": "int32"
        },
        "my_nullable_enum": {
            "anyOf": [
                {
                    "$ref": "#/definitions/MyEnum"
                },
                {
                    "type": "null"
                }
            ]
        }
    },
    "definitions": {
        "MyEnum": {
            "anyOf": [
                {
                    "type": "object",
                    "required": [
                        "StringNewType"
                    ],
                    "properties": {
                        "StringNewType": {
                            "type": "string"
                        }
                    },
                    "additionalProperties": false
                },
                {
                    "type": "object",
                    "required": [
                        "StructVariant"
                    ],
                    "properties": {
                        "StructVariant": {
                            "type": "object",
                            "required": [
                                "floats"
                            ],
                            "properties": {
                                "floats": {
                                    "type": "array",
                                    "items": {
                                        "type": "number",
                                        "format": "float"
                                    }
                                }
                            }
                        }
                    },
                    "additionalProperties": false
                }
            ]
        }
    }
}
```
</details>

### Serde Compatibility

One of the main aims of this library is compatibility with [Serde](https://github.com/serde-rs/serde). Any generated schema *should* match how [serde_json](https://github.com/serde-rs/json) would serialize/deserialize to/from JSON. To support this, Schemars will check for any `#[serde(...)]` attributes on types that derive `JsonSchema`, and adjust the generated schema accordingly.

```rust
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MyStruct {
    #[serde(rename = "myNumber")]
    pub my_int: i32,
    pub my_bool: bool,
    #[serde(default)]
    pub my_nullable_enum: Option<MyEnum>,
}

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum MyEnum {
    StringNewType(String),
    StructVariant { floats: Vec<f32> },
}

let schema = schema_for!(MyStruct);
println!("{}", serde_json::to_string_pretty(&schema).unwrap());
```

<details>
<summary>Click to see the output JSON schema...</summary>

```json
{
    "$schema": "http://json-schema.org/draft-07/schema#",
    "title": "MyStruct",
    "type": "object",
    "required": [
        "myBool",
        "myNumber"
    ],
    "properties": {
        "myBool": {
            "type": "boolean"
        },
        "myNullableEnum": {
            "default": null,
            "anyOf": [
                {
                    "$ref": "#/definitions/MyEnum"
                },
                {
                    "type": "null"
                }
            ]
        },
        "myNumber": {
            "type": "integer",
            "format": "int32"
        }
    },
    "additionalProperties": false,
    "definitions": {
        "MyEnum": {
            "anyOf": [
                {
                    "type": "string"
                },
                {
                    "type": "object",
                    "required": [
                        "floats"
                    ],
                    "properties": {
                        "floats": {
                            "type": "array",
                            "items": {
                                "type": "number",
                                "format": "float"
                            }
                        }
                    }
                }
            ]
        }
    }
}
```
</details>

`#[serde(...)]` attributes can be overriden using `#[schemars(...)]` attributes, which behave identically (e.g. `#[schemars(rename_all = "camelCase")]`). You may find this useful if you want to change the generated schema without affecting Serde's behaviour, or if you're just not using Serde.

### Schema from Example Value

If you want a schema for a type that can't/doesn't implement `JsonSchema`, but does implement `serde::Serialize`, then you can generate a JSON schema from a value of that type. However, this schema will generally be less precise than if the type implemented `JsonSchema` - particularly when it involves enums, since schemars will not make any assumptions about the structure of an enum based on a single variant.

```rust
use schemars::schema_for_value;
use serde::Serialize;

#[derive(Serialize)]
pub struct MyStruct {
    pub my_int: i32,
    pub my_bool: bool,
    pub my_nullable_enum: Option<MyEnum>,
}

#[derive(Serialize)]
pub enum MyEnum {
    StringNewType(String),
    StructVariant { floats: Vec<f32> },
}

let schema = schema_for_value!(MyStruct {
    my_int: 123,
    my_bool: true,
    my_nullable_enum: Some(MyEnum::StringNewType("foo".to_string()))
});
println!("{}", serde_json::to_string_pretty(&schema).unwrap());
```

<details>
<summary>Click to see the output JSON schema...</summary>

```json
{
    "$schema": "http://json-schema.org/draft-07/schema#",
    "title": "MyStruct",
    "examples": [
        {
            "my_bool": true,
            "my_int": 123,
            "my_nullable_enum": {
                "StringNewType": "foo"
            }
        }
    ],
    "type": "object",
    "properties": {
        "my_bool": {
            "type": "boolean"
        },
        "my_int": {
            "type": "integer"
        },
        "my_nullable_enum": true
    }
}
```
</details>

## Feature Flags
- `derive` (enabled by default) - provides `#[derive(JsonSchema)]` macro
- `impl_json_schema` - implements `JsonSchema` for Schemars types themselves
- `preserve_order` - keep the order of struct fields in `Schema` and `SchemaObject`

Schemars can implement `JsonSchema` on types from several popular crates, enabled via feature flags (dependency versions are shown in brackets):
- `chrono` - [chrono](https://crates.io/crates/chrono) (^0.4)
- `indexmap1` - [indexmap](https://crates.io/crates/indexmap) (^1.2)
- `either` - [either](https://crates.io/crates/either) (^1.3)
- `uuid08` - [uuid](https://crates.io/crates/uuid) (^0.8)
- `uuid1` - [uuid](https://crates.io/crates/uuid) (^1.0)
- `smallvec` - [smallvec](https://crates.io/crates/smallvec) (^1.0)
- `arrayvec05` - [arrayvec](https://crates.io/crates/arrayvec) (^0.5)
- `arrayvec07` - [arrayvec](https://crates.io/crates/arrayvec) (^0.7)
- `url` - [url](https://crates.io/crates/url) (^2.0)
- `bytes` - [bytes](https://crates.io/crates/bytes) (^1.0)
- `enumset` - [enumset](https://crates.io/crates/enumset) (^1.0)
- `rust_decimal` - [rust_decimal](https://crates.io/crates/rust_decimal) (^1.0)
- `bigdecimal` - [bigdecimal](https://crates.io/crates/bigdecimal) (^0.3)
- `smol_str` - [smol_str](https://crates.io/crates/smol_str) (^0.1.17)

For example, to implement `JsonSchema` on types from `chrono`, enable it as a feature in the `schemars` dependency in your `Cargo.toml` like so:

```toml
[dependencies]
schemars = { version = "0.8", features = ["chrono"] }
```
*/

/// The map type used by schemars types.
///
/// Currently a `BTreeMap` or `IndexMap` can be used, but this may change to a different implementation
/// with a similar interface in a future version of schemars.
/// The `IndexMap` will be used when the `preserve_order` feature flag is set.
#[cfg(not(feature = "preserve_order"))]
pub type Map<K, V> = std::collections::BTreeMap<K, V>;
#[cfg(feature = "preserve_order")]
pub type Map<K, V> = indexmap::IndexMap<K, V>;
/// The set type used by schemars types.
///
/// Currently a `BTreeSet`, but this may change to a different implementation
/// with a similar interface in a future version of schemars.
pub type Set<T> = std::collections::BTreeSet<T>;

/// A view into a single entry in a map, which may either be vacant or occupied.
//
/// This is constructed from the `entry` method on `BTreeMap` or `IndexMap`,
/// depending on whether the `preserve_order` feature flag is set.
#[cfg(not(feature = "preserve_order"))]
pub type MapEntry<'a, K, V> = std::collections::btree_map::Entry<'a, K, V>;
#[cfg(feature = "preserve_order")]
pub type MapEntry<'a, K, V> = indexmap::map::Entry<'a, K, V>;

mod flatten;
mod json_schema_impls;
mod ser;
#[macro_use]
mod macros;

/// This module is only public for use by `schemars_derive`. It should not need to be used by code
/// outside of `schemars`, and should not be considered part of the public API.
#[doc(hidden)]
pub mod _private;
pub mod gen;
pub mod schema;
pub mod visit;

#[cfg(feature = "schemars_derive")]
extern crate schemars_derive;
#[cfg(feature = "schemars_derive")]
pub use schemars_derive::*;

// Export serde_json so schemars_derive can use it
#[doc(hidden)]
pub use serde_json as _serde_json;

use schema::Schema;

/// A type which can be described as a JSON Schema document.
///
/// This is implemented for many Rust primitive and standard library types.
///
/// This can also be automatically derived on most custom types with `#[derive(JsonSchema)]`.
///
/// # Example
/// ```
/// use schemars::{schema_for, JsonSchema};
///
/// #[derive(JsonSchema)]
/// struct MyStruct {
///     foo: i32,
/// }
///
/// let my_schema = schema_for!(MyStruct);
/// ```
pub trait JsonSchema {
    /// Whether JSON Schemas generated for this type should be re-used where possible using the `$ref` keyword.
    ///
    /// For trivial types (such as primitives), this should return `false`. For more complex types, it should return `true`.
    /// For recursive types, this **must** return `true` to prevent infinite cycles when generating schemas.
    ///
    /// By default, this returns `true`.
    fn is_referenceable() -> bool {
        true
    }

    /// The name of the generated JSON Schema.
    ///
    /// This is used as the title for root schemas, and the key within the root's `definitions` property for subschemas.
    fn schema_name() -> String;

    /// Generates a JSON Schema for this type.
    ///
    /// If the returned schema depends on any [referenceable](JsonSchema::is_referenceable) schemas, then this method will
    /// add them to the [`SchemaGenerator`](gen::SchemaGenerator)'s schema definitions.
    ///
    /// This should not return a `$ref` schema.
    fn json_schema(gen: &mut gen::SchemaGenerator) -> Schema;

    // TODO document and bring into public API?
    #[doc(hidden)]
    fn _schemars_private_non_optional_json_schema(gen: &mut gen::SchemaGenerator) -> Schema {
        Self::json_schema(gen)
    }

    // TODO document and bring into public API?
    #[doc(hidden)]
    fn _schemars_private_is_option() -> bool {
        false
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn schema_object_for<T: JsonSchema>() -> schema::SchemaObject {
        schema_object(schema_for::<T>())
    }

    pub fn schema_for<T: JsonSchema>() -> schema::Schema {
        let mut gen = gen::SchemaGenerator::default();
        T::json_schema(&mut gen)
    }

    pub fn schema_object(schema: schema::Schema) -> schema::SchemaObject {
        match schema {
            schema::Schema::Object(o) => o,
            s => panic!("Schema was not an object: {:?}", s),
        }
    }
}
