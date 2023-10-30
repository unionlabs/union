//! Since Borsh is not a self-descriptive format we have a way to describe types serialized with Borsh so that
//! we can deserialize serialized blobs without having Rust types available. Additionally, this can be used to
//! serialize content provided in a different format, e.g. JSON object `{"user": "alice", "message": "Message"}`
//! can be serialized by JS code into Borsh format such that it can be deserialized into `struct UserMessage {user: String, message: String}`
//! on Rust side.
//!
//! The important components are: `BorshSchema` trait, `Definition` and `Declaration` types, and `BorshSchemaContainer` struct.
//! * `BorshSchema` trait allows any type that implements it to be self-descriptive, i.e. generate it's own schema;
//! * `Declaration` is used to describe the type identifier, e.g. `HashMap<u64, String>`;
//! * `Definition` is used to describe the structure of the type;
//! * `BorshSchemaContainer` is used to store all declarations and defintions that are needed to work with a single type.

#![allow(dead_code)] // Unclear why rust check complains on fields of `Definition` variants.
use crate as borsh; // For `#[derive(BorshSerialize, BorshDeserialize)]`.
use crate::maybestd::{
    boxed::Box,
    collections::{hash_map::Entry, HashMap, HashSet},
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use crate::{BorshDeserialize, BorshSchema as BorshSchemaMacro, BorshSerialize};

/// The type that we use to represent the declaration of the Borsh type.
pub type Declaration = String;
/// The type that we use for the name of the variant.
pub type VariantName = String;
/// The name of the field in the struct (can be used to convert JSON to Borsh using the schema).
pub type FieldName = String;
/// The type that we use to represent the definition of the Borsh type.
#[derive(Clone, PartialEq, Eq, Debug, BorshSerialize, BorshDeserialize, BorshSchemaMacro)]
pub enum Definition {
    /// A fixed-size array with the length known at the compile time and the same-type elements.
    Array { length: u32, elements: Declaration },
    /// A sequence of elements of length known at the run time and the same-type elements.
    Sequence { elements: Declaration },
    /// A fixed-size tuple with the length known at the compile time and the elements of different
    /// types.
    Tuple { elements: Vec<Declaration> },
    /// A tagged union, a.k.a enum. Tagged-unions have variants with associated structures.
    Enum {
        variants: Vec<(VariantName, Declaration)>,
    },
    /// A structure, structurally similar to a tuple.
    Struct { fields: Fields },
}

/// The collection representing the fields of a struct.
#[derive(Clone, PartialEq, Eq, Debug, BorshSerialize, BorshDeserialize, BorshSchemaMacro)]
pub enum Fields {
    /// The struct with named fields.
    NamedFields(Vec<(FieldName, Declaration)>),
    /// The struct with unnamed fields, structurally identical to a tuple.
    UnnamedFields(Vec<Declaration>),
    /// The struct with no fields.
    Empty,
}

/// All schema information needed to deserialize a single type.
#[derive(Clone, PartialEq, Eq, Debug, BorshSerialize, BorshDeserialize, BorshSchemaMacro)]
pub struct BorshSchemaContainer {
    /// Declaration of the type.
    pub declaration: Declaration,
    /// All definitions needed to deserialize the given type.
    pub definitions: HashMap<Declaration, Definition>,
}

/// The declaration and the definition of the type that can be used to (de)serialize Borsh without
/// the Rust type that produced it.
pub trait BorshSchema {
    /// Recursively, using DFS, add type definitions required for this type. For primitive types
    /// this is an empty map. Type definition explains how to serialize/deserialize a type.
    fn add_definitions_recursively(definitions: &mut HashMap<Declaration, Definition>);

    /// Helper method to add a single type definition to the map.
    fn add_definition(
        declaration: Declaration,
        definition: Definition,
        definitions: &mut HashMap<Declaration, Definition>,
    ) {
        match definitions.entry(declaration) {
            Entry::Occupied(occ) => {
                let existing_def = occ.get();
                assert_eq!(existing_def, &definition, "Redefining type schema for the same type name. Types with the same names are not supported.");
            }
            Entry::Vacant(vac) => {
                vac.insert(definition);
            }
        }
    }
    /// Get the name of the type without brackets.
    fn declaration() -> Declaration;

    fn schema_container() -> BorshSchemaContainer {
        let mut definitions = HashMap::new();
        Self::add_definitions_recursively(&mut definitions);
        BorshSchemaContainer {
            declaration: Self::declaration(),
            definitions,
        }
    }
}

impl<T> BorshSchema for Box<T>
where
    T: BorshSchema + ?Sized,
{
    fn add_definitions_recursively(definitions: &mut HashMap<Declaration, Definition>) {
        T::add_definitions_recursively(definitions);
    }

    fn declaration() -> Declaration {
        T::declaration()
    }
}

impl BorshSchema for () {
    fn add_definitions_recursively(_definitions: &mut HashMap<Declaration, Definition>) {}

    fn declaration() -> Declaration {
        "nil".to_string()
    }
}

macro_rules! impl_for_renamed_primitives {
    ($($type: ident : $name: ident)+) => {
    $(
        impl BorshSchema for $type {
            fn add_definitions_recursively(_definitions: &mut HashMap<Declaration, Definition>) {}
            fn declaration() -> Declaration {
                stringify!($name).to_string()
            }
        }
    )+
    };
}

macro_rules! impl_for_primitives {
    ($($type: ident)+) => {
    impl_for_renamed_primitives!{$($type : $type)+}
    };
}

impl_for_primitives!(bool f32 f64 i8 i16 i32 i64 i128 u8 u16 u32 u64 u128);
impl_for_renamed_primitives!(String: string);
impl_for_renamed_primitives!(str: string);
impl_for_renamed_primitives!(isize: i64);
impl_for_renamed_primitives!(usize: u64);

impl<T, const N: usize> BorshSchema for [T; N]
where
    T: BorshSchema,
{
    fn add_definitions_recursively(definitions: &mut HashMap<Declaration, Definition>) {
        let definition = Definition::Array {
            length: N as u32,
            elements: T::declaration(),
        };
        Self::add_definition(Self::declaration(), definition, definitions);
        T::add_definitions_recursively(definitions);
    }
    fn declaration() -> Declaration {
        format!(r#"Array<{}, {}>"#, T::declaration(), N)
    }
}

impl<T> BorshSchema for Option<T>
where
    T: BorshSchema,
{
    fn add_definitions_recursively(definitions: &mut HashMap<Declaration, Definition>) {
        let definition = Definition::Enum {
            variants: vec![
                ("None".to_string(), <()>::declaration()),
                ("Some".to_string(), T::declaration()),
            ],
        };
        Self::add_definition(Self::declaration(), definition, definitions);
        T::add_definitions_recursively(definitions);
    }

    fn declaration() -> Declaration {
        format!(r#"Option<{}>"#, T::declaration())
    }
}

impl<T, E> BorshSchema for core::result::Result<T, E>
where
    T: BorshSchema,
    E: BorshSchema,
{
    fn add_definitions_recursively(definitions: &mut HashMap<Declaration, Definition>) {
        let definition = Definition::Enum {
            variants: vec![
                ("Ok".to_string(), T::declaration()),
                ("Err".to_string(), E::declaration()),
            ],
        };
        Self::add_definition(Self::declaration(), definition, definitions);
        T::add_definitions_recursively(definitions);
    }

    fn declaration() -> Declaration {
        format!(r#"Result<{}, {}>"#, T::declaration(), E::declaration())
    }
}

impl<T> BorshSchema for Vec<T>
where
    T: BorshSchema,
{
    fn add_definitions_recursively(definitions: &mut HashMap<Declaration, Definition>) {
        let definition = Definition::Sequence {
            elements: T::declaration(),
        };
        Self::add_definition(Self::declaration(), definition, definitions);
        T::add_definitions_recursively(definitions);
    }

    fn declaration() -> Declaration {
        format!(r#"Vec<{}>"#, T::declaration())
    }
}

impl<T> BorshSchema for [T]
where
    T: BorshSchema,
{
    fn add_definitions_recursively(definitions: &mut HashMap<Declaration, Definition>) {
        let definition = Definition::Sequence {
            elements: T::declaration(),
        };
        Self::add_definition(Self::declaration(), definition, definitions);
        T::add_definitions_recursively(definitions);
    }

    fn declaration() -> Declaration {
        format!(r#"Vec<{}>"#, T::declaration())
    }
}

impl<K, V> BorshSchema for HashMap<K, V>
where
    K: BorshSchema,
    V: BorshSchema,
{
    fn add_definitions_recursively(definitions: &mut HashMap<Declaration, Definition>) {
        let definition = Definition::Sequence {
            elements: <(K, V)>::declaration(),
        };
        Self::add_definition(Self::declaration(), definition, definitions);
        <(K, V)>::add_definitions_recursively(definitions);
    }

    fn declaration() -> Declaration {
        format!(r#"HashMap<{}, {}>"#, K::declaration(), V::declaration())
    }
}

impl<T> BorshSchema for HashSet<T>
where
    T: BorshSchema,
{
    fn add_definitions_recursively(definitions: &mut HashMap<Declaration, Definition>) {
        let definition = Definition::Sequence {
            elements: <T>::declaration(),
        };
        Self::add_definition(Self::declaration(), definition, definitions);
        <T>::add_definitions_recursively(definitions);
    }

    fn declaration() -> Declaration {
        format!(r#"HashSet<{}>"#, T::declaration())
    }
}

macro_rules! impl_tuple {
    ($($name:ident),+) => {
    impl<$($name),+> BorshSchema for ($($name,)+)
    where
        $($name: BorshSchema),+
    {
        fn add_definitions_recursively(definitions: &mut HashMap<Declaration, Definition>) {
            let elements = vec![$($name::declaration()),+];

            let definition = Definition::Tuple { elements };
            Self::add_definition(Self::declaration(), definition, definitions);
            $(
                $name::add_definitions_recursively(definitions);
            )+
        }

        fn declaration() -> Declaration {
            let params = vec![$($name::declaration()),+];
            format!(r#"Tuple<{}>"#, params.join(", "))
        }
    }
    };
}

impl_tuple!(T0);
impl_tuple!(T0, T1);
impl_tuple!(T0, T1, T2);
impl_tuple!(T0, T1, T2, T3);
impl_tuple!(T0, T1, T2, T3, T4);
impl_tuple!(T0, T1, T2, T3, T4, T5);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17);
impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18);
impl_tuple!(
    T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19
);
impl_tuple!(
    T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maybestd::collections::HashMap;

    macro_rules! map(
    () => { HashMap::new() };
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = HashMap::new();
            $(
                m.insert($key.to_string(), $value);
            )+
            m
        }
     };
    );

    #[test]
    fn simple_option() {
        let actual_name = Option::<u64>::declaration();
        let mut actual_defs = map!();
        Option::<u64>::add_definitions_recursively(&mut actual_defs);
        assert_eq!("Option<u64>", actual_name);
        assert_eq!(
            map! {"Option<u64>" =>
            Definition::Enum{ variants: vec![
                ("None".to_string(), "nil".to_string()),
                ("Some".to_string(), "u64".to_string()),
            ]}
            },
            actual_defs
        );
    }

    #[test]
    fn nested_option() {
        let actual_name = Option::<Option<u64>>::declaration();
        let mut actual_defs = map!();
        Option::<Option<u64>>::add_definitions_recursively(&mut actual_defs);
        assert_eq!("Option<Option<u64>>", actual_name);
        assert_eq!(
            map! {
            "Option<u64>" =>
                Definition::Enum {variants: vec![
                ("None".to_string(), "nil".to_string()),
                ("Some".to_string(), "u64".to_string()),
                ]},
            "Option<Option<u64>>" =>
                Definition::Enum {variants: vec![
                ("None".to_string(), "nil".to_string()),
                ("Some".to_string(), "Option<u64>".to_string()),
                ]}
            },
            actual_defs
        );
    }

    #[test]
    fn simple_vec() {
        let actual_name = Vec::<u64>::declaration();
        let mut actual_defs = map!();
        Vec::<u64>::add_definitions_recursively(&mut actual_defs);
        assert_eq!("Vec<u64>", actual_name);
        assert_eq!(
            map! {
            "Vec<u64>" => Definition::Sequence { elements: "u64".to_string() }
            },
            actual_defs
        );
    }

    #[test]
    fn nested_vec() {
        let actual_name = Vec::<Vec<u64>>::declaration();
        let mut actual_defs = map!();
        Vec::<Vec<u64>>::add_definitions_recursively(&mut actual_defs);
        assert_eq!("Vec<Vec<u64>>", actual_name);
        assert_eq!(
            map! {
            "Vec<u64>" => Definition::Sequence { elements: "u64".to_string() },
            "Vec<Vec<u64>>" => Definition::Sequence { elements: "Vec<u64>".to_string() }
            },
            actual_defs
        );
    }

    #[test]
    fn simple_tuple() {
        let actual_name = <(u64, String)>::declaration();
        let mut actual_defs = map!();
        <(u64, String)>::add_definitions_recursively(&mut actual_defs);
        assert_eq!("Tuple<u64, string>", actual_name);
        assert_eq!(
            map! {
                "Tuple<u64, string>" => Definition::Tuple { elements: vec![ "u64".to_string(), "string".to_string()]}
            },
            actual_defs
        );
    }

    #[test]
    fn nested_tuple() {
        let actual_name = <(u64, (u8, bool), String)>::declaration();
        let mut actual_defs = map!();
        <(u64, (u8, bool), String)>::add_definitions_recursively(&mut actual_defs);
        assert_eq!("Tuple<u64, Tuple<u8, bool>, string>", actual_name);
        assert_eq!(
            map! {
                "Tuple<u64, Tuple<u8, bool>, string>" => Definition::Tuple { elements: vec![
                    "u64".to_string(),
                    "Tuple<u8, bool>".to_string(),
                    "string".to_string(),
                ]},
                "Tuple<u8, bool>" => Definition::Tuple { elements: vec![ "u8".to_string(), "bool".to_string()]}
            },
            actual_defs
        );
    }

    #[test]
    fn simple_map() {
        let actual_name = HashMap::<u64, String>::declaration();
        let mut actual_defs = map!();
        HashMap::<u64, String>::add_definitions_recursively(&mut actual_defs);
        assert_eq!("HashMap<u64, string>", actual_name);
        assert_eq!(
            map! {
                "HashMap<u64, string>" => Definition::Sequence { elements: "Tuple<u64, string>".to_string()} ,
                "Tuple<u64, string>" => Definition::Tuple { elements: vec![ "u64".to_string(), "string".to_string()]}
            },
            actual_defs
        );
    }

    #[test]
    fn simple_set() {
        let actual_name = HashSet::<String>::declaration();
        let mut actual_defs = map!();
        HashSet::<String>::add_definitions_recursively(&mut actual_defs);
        assert_eq!("HashSet<string>", actual_name);
        assert_eq!(
            map! {
                "HashSet<string>" => Definition::Sequence { elements: "string".to_string()}
            },
            actual_defs
        );
    }

    #[test]
    fn simple_array() {
        let actual_name = <[u64; 32]>::declaration();
        let mut actual_defs = map!();
        <[u64; 32]>::add_definitions_recursively(&mut actual_defs);
        assert_eq!("Array<u64, 32>", actual_name);
        assert_eq!(
            map! {"Array<u64, 32>" => Definition::Array { length: 32, elements: "u64".to_string()}},
            actual_defs
        );
    }

    #[test]
    fn nested_array() {
        let actual_name = <[[[u64; 9]; 10]; 32]>::declaration();
        let mut actual_defs = map!();
        <[[[u64; 9]; 10]; 32]>::add_definitions_recursively(&mut actual_defs);
        assert_eq!("Array<Array<Array<u64, 9>, 10>, 32>", actual_name);
        assert_eq!(
            map! {
            "Array<u64, 9>" =>
                Definition::Array { length: 9, elements: "u64".to_string() },
            "Array<Array<u64, 9>, 10>" =>
                Definition::Array { length: 10, elements: "Array<u64, 9>".to_string() },
            "Array<Array<Array<u64, 9>, 10>, 32>" =>
                Definition::Array { length: 32, elements: "Array<Array<u64, 9>, 10>".to_string() }
            },
            actual_defs
        );
    }

    #[test]
    fn string() {
        let actual_name = str::declaration();
        assert_eq!("string", actual_name);
        let actual_name = String::declaration();
        assert_eq!("string", actual_name);
        let mut actual_defs = map!();
        String::add_definitions_recursively(&mut actual_defs);
        assert_eq!(map! {}, actual_defs);
    }

    #[test]
    fn boxed_schema() {
        let boxed_declaration = Box::<str>::declaration();
        assert_eq!("string", boxed_declaration);
        let boxed_declaration = Box::<[u8]>::declaration();
        assert_eq!("Vec<u8>", boxed_declaration);
    }
}
