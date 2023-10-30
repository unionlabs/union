use borsh::maybestd::collections::HashMap;
use borsh::schema::*;

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
pub fn unit_struct() {
    #[derive(borsh::BorshSchema)]
    struct A;
    assert_eq!("A".to_string(), A::declaration());
    let mut defs = Default::default();
    A::add_definitions_recursively(&mut defs);
    assert_eq!(
        map! {
        "A" => Definition::Struct {fields: Fields::Empty}
        },
        defs
    );
}

#[test]
pub fn simple_struct() {
    #[derive(borsh::BorshSchema)]
    struct A {
        _f1: u64,
        _f2: String,
    }
    assert_eq!("A".to_string(), A::declaration());
    let mut defs = Default::default();
    A::add_definitions_recursively(&mut defs);
    assert_eq!(
        map! {
        "A" => Definition::Struct{ fields: Fields::NamedFields(vec![
        ("_f1".to_string(), "u64".to_string()),
        ("_f2".to_string(), "string".to_string())
        ])}
        },
        defs
    );
}

#[test]
pub fn boxed() {
    #[derive(borsh::BorshSchema)]
    struct A {
        _f1: Box<u64>,
        _f2: Box<str>,
        _f3: Box<[u8]>,
    }
    assert_eq!("A".to_string(), A::declaration());
    let mut defs = Default::default();
    A::add_definitions_recursively(&mut defs);
    assert_eq!(
        map! {
        "Vec<u8>" => Definition::Sequence { elements: "u8".to_string() },
        "A" => Definition::Struct{ fields: Fields::NamedFields(vec![
        ("_f1".to_string(), "u64".to_string()),
        ("_f2".to_string(), "string".to_string()),
        ("_f3".to_string(), "Vec<u8>".to_string())
        ])}
        },
        defs
    );
}

#[test]
pub fn wrapper_struct() {
    #[derive(borsh::BorshSchema)]
    struct A<T>(T);
    assert_eq!("A<u64>".to_string(), <A<u64>>::declaration());
    let mut defs = Default::default();
    <A<u64>>::add_definitions_recursively(&mut defs);
    assert_eq!(
        map! {
        "A<u64>" => Definition::Struct {fields: Fields::UnnamedFields(vec!["u64".to_string()])}
        },
        defs
    );
}

#[test]
pub fn tuple_struct() {
    #[derive(borsh::BorshSchema)]
    struct A(u64, String);
    assert_eq!("A".to_string(), A::declaration());
    let mut defs = Default::default();
    A::add_definitions_recursively(&mut defs);
    assert_eq!(
        map! {
        "A" => Definition::Struct {fields: Fields::UnnamedFields(vec![
         "u64".to_string(), "string".to_string()
        ])}
        },
        defs
    );
}

#[test]
pub fn tuple_struct_params() {
    #[derive(borsh::BorshSchema)]
    struct A<K, V>(K, V);
    assert_eq!(
        "A<u64, string>".to_string(),
        <A<u64, String>>::declaration()
    );
    let mut defs = Default::default();
    <A<u64, String>>::add_definitions_recursively(&mut defs);
    assert_eq!(
        map! {
        "A<u64, string>" => Definition::Struct { fields: Fields::UnnamedFields(vec![
            "u64".to_string(), "string".to_string()
        ])}
        },
        defs
    );
}

#[test]
pub fn simple_generics() {
    #[derive(borsh::BorshSchema)]
    struct A<K, V> {
        _f1: HashMap<K, V>,
        _f2: String,
    }
    assert_eq!(
        "A<u64, string>".to_string(),
        <A<u64, String>>::declaration()
    );
    let mut defs = Default::default();
    <A<u64, String>>::add_definitions_recursively(&mut defs);
    assert_eq!(
        map! {
        "A<u64, string>" => Definition::Struct {
        fields: Fields::NamedFields(vec![
        ("_f1".to_string(), "HashMap<u64, string>".to_string()),
        ("_f2".to_string(), "string".to_string())
        ])
        },
        "HashMap<u64, string>" => Definition::Sequence {elements: "Tuple<u64, string>".to_string()},
        "Tuple<u64, string>" => Definition::Tuple{elements: vec!["u64".to_string(), "string".to_string()]}
        },
        defs
    );
}
