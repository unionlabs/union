#![allow(dead_code)] // Local structures do not have their fields used.
use borsh::maybestd::collections::HashMap;
use borsh::schema::*;
use borsh::schema_helpers::{try_from_slice_with_schema, try_to_vec_with_schema};

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
pub fn simple_enum() {
    #[derive(borsh::BorshSchema)]
    enum A {
        Bacon,
        Eggs,
    }
    assert_eq!("A".to_string(), A::declaration());
    let mut defs = Default::default();
    A::add_definitions_recursively(&mut defs);
    assert_eq!(
        map! {
        "ABacon" => Definition::Struct{ fields: Fields::Empty },
        "AEggs" => Definition::Struct{ fields: Fields::Empty },
        "A" => Definition::Enum { variants: vec![("Bacon".to_string(), "ABacon".to_string()), ("Eggs".to_string(), "AEggs".to_string())]}
        },
        defs
    );
}

#[test]
pub fn single_field_enum() {
    #[derive(borsh::BorshSchema)]
    enum A {
        Bacon,
    }
    assert_eq!("A".to_string(), A::declaration());
    let mut defs = Default::default();
    A::add_definitions_recursively(&mut defs);
    assert_eq!(
        map! {
        "ABacon" => Definition::Struct {fields: Fields::Empty},
        "A" => Definition::Enum { variants: vec![("Bacon".to_string(), "ABacon".to_string())]}
        },
        defs
    );
}

#[test]
pub fn complex_enum_with_schema() {
    #[derive(
        borsh::BorshSchema,
        Default,
        borsh::BorshSerialize,
        borsh::BorshDeserialize,
        PartialEq,
        Debug,
    )]
    struct Tomatoes;
    #[derive(
        borsh::BorshSchema,
        Default,
        borsh::BorshSerialize,
        borsh::BorshDeserialize,
        PartialEq,
        Debug,
    )]
    struct Cucumber;
    #[derive(
        borsh::BorshSchema,
        Default,
        borsh::BorshSerialize,
        borsh::BorshDeserialize,
        PartialEq,
        Debug,
    )]
    struct Oil;
    #[derive(
        borsh::BorshSchema,
        Default,
        borsh::BorshSerialize,
        borsh::BorshDeserialize,
        PartialEq,
        Debug,
    )]
    struct Wrapper;
    #[derive(
        borsh::BorshSchema,
        Default,
        borsh::BorshSerialize,
        borsh::BorshDeserialize,
        PartialEq,
        Debug,
    )]
    struct Filling;
    #[derive(
        borsh::BorshSchema, borsh::BorshSerialize, borsh::BorshDeserialize, PartialEq, Debug,
    )]
    enum A {
        Bacon,
        Eggs,
        Salad(Tomatoes, Cucumber, Oil),
        Sausage { wrapper: Wrapper, filling: Filling },
    }

    impl Default for A {
        fn default() -> Self {
            A::Sausage {
                wrapper: Default::default(),
                filling: Default::default(),
            }
        }
    }
    // First check schema.
    assert_eq!("A".to_string(), A::declaration());
    let mut defs = Default::default();
    A::add_definitions_recursively(&mut defs);
    assert_eq!(
        map! {
        "Cucumber" => Definition::Struct {fields: Fields::Empty},
        "ASalad" => Definition::Struct{ fields: Fields::UnnamedFields(vec!["Tomatoes".to_string(), "Cucumber".to_string(), "Oil".to_string()])},
        "ABacon" => Definition::Struct {fields: Fields::Empty},
        "Oil" => Definition::Struct {fields: Fields::Empty},
        "A" => Definition::Enum{ variants: vec![
        ("Bacon".to_string(), "ABacon".to_string()),
        ("Eggs".to_string(), "AEggs".to_string()),
        ("Salad".to_string(), "ASalad".to_string()),
        ("Sausage".to_string(), "ASausage".to_string())]},
        "Wrapper" => Definition::Struct {fields: Fields::Empty},
        "Tomatoes" => Definition::Struct {fields: Fields::Empty},
        "ASausage" => Definition::Struct { fields: Fields::NamedFields(vec![
        ("wrapper".to_string(), "Wrapper".to_string()),
        ("filling".to_string(), "Filling".to_string())
        ])},
        "AEggs" => Definition::Struct {fields: Fields::Empty},
        "Filling" => Definition::Struct {fields: Fields::Empty}
        },
        defs
    );
    // Then check that we serialize and deserialize with schema.
    let obj = A::default();
    let data = try_to_vec_with_schema(&obj).unwrap();
    let obj2: A = try_from_slice_with_schema(&data).unwrap();
    assert_eq!(obj, obj2);
}

#[test]
pub fn complex_enum_generics() {
    #[derive(borsh::BorshSchema)]
    struct Tomatoes;
    #[derive(borsh::BorshSchema)]
    struct Cucumber;
    #[derive(borsh::BorshSchema)]
    struct Oil;
    #[derive(borsh::BorshSchema)]
    struct Wrapper;
    #[derive(borsh::BorshSchema)]
    struct Filling;
    #[derive(borsh::BorshSchema)]
    enum A<C, W> {
        Bacon,
        Eggs,
        Salad(Tomatoes, C, Oil),
        Sausage { wrapper: W, filling: Filling },
    }
    assert_eq!(
        "A<Cucumber, Wrapper>".to_string(),
        <A<Cucumber, Wrapper>>::declaration()
    );
    let mut defs = Default::default();
    <A<Cucumber, Wrapper>>::add_definitions_recursively(&mut defs);
    assert_eq!(
        map! {
        "Cucumber" => Definition::Struct {fields: Fields::Empty},
        "ASalad<Cucumber, Wrapper>" => Definition::Struct{
            fields: Fields::UnnamedFields(vec!["Tomatoes".to_string(), "Cucumber".to_string(), "Oil".to_string()])
        },
        "ABacon<Cucumber, Wrapper>" => Definition::Struct {fields: Fields::Empty},
        "Oil" => Definition::Struct {fields: Fields::Empty},
        "A<Cucumber, Wrapper>" => Definition::Enum{
            variants: vec![
            ("Bacon".to_string(), "ABacon<Cucumber, Wrapper>".to_string()),
            ("Eggs".to_string(), "AEggs<Cucumber, Wrapper>".to_string()),
            ("Salad".to_string(), "ASalad<Cucumber, Wrapper>".to_string()),
            ("Sausage".to_string(), "ASausage<Cucumber, Wrapper>".to_string())
            ]
        },
        "Wrapper" => Definition::Struct {fields: Fields::Empty},
        "Tomatoes" => Definition::Struct {fields: Fields::Empty},
        "ASausage<Cucumber, Wrapper>" => Definition::Struct {
            fields: Fields::NamedFields(vec![
            ("wrapper".to_string(), "Wrapper".to_string()),
            ("filling".to_string(), "Filling".to_string())
            ])
        },
        "AEggs<Cucumber, Wrapper>" => Definition::Struct {fields: Fields::Empty},
        "Filling" => Definition::Struct {fields: Fields::Empty}
        },
        defs
    );
}
