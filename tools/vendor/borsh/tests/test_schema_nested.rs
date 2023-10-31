#![allow(dead_code)] // Local structures do not have their fields used.
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

// Checks that recursive definitions work. Also checks that re-instantiations of templated types work.
#[test]
pub fn duplicated_instantiations() {
    #[derive(borsh::BorshSchema)]
    struct Tomatoes;
    #[derive(borsh::BorshSchema)]
    struct Cucumber;
    #[derive(borsh::BorshSchema)]
    struct Oil<K, V> {
        seeds: HashMap<K, V>,
        liquid: Option<K>,
    }
    #[derive(borsh::BorshSchema)]
    struct Wrapper<T> {
        foo: Option<T>,
        bar: Box<A<T, T>>,
    }
    #[derive(borsh::BorshSchema)]
    struct Filling;
    #[derive(borsh::BorshSchema)]
    enum A<C, W> {
        Bacon,
        Eggs,
        Salad(Tomatoes, C, Oil<u64, String>),
        Sausage { wrapper: W, filling: Filling },
    }
    assert_eq!(
        "A<Cucumber, Wrapper<string>>".to_string(),
        <A<Cucumber, Wrapper<String>>>::declaration()
    );
    let mut defs = Default::default();
    <A<Cucumber, Wrapper<String>>>::add_definitions_recursively(&mut defs);
    assert_eq!(
        map! {
        "A<Cucumber, Wrapper<string>>" => Definition::Enum {variants: vec![
         ("Bacon".to_string(), "ABacon<Cucumber, Wrapper<string>>".to_string()),
         ("Eggs".to_string(), "AEggs<Cucumber, Wrapper<string>>".to_string()),
         ("Salad".to_string(), "ASalad<Cucumber, Wrapper<string>>".to_string()),
         ("Sausage".to_string(), "ASausage<Cucumber, Wrapper<string>>".to_string())
        ]},
        "A<string, string>" => Definition::Enum {variants: vec![
            ("Bacon".to_string(), "ABacon<string, string>".to_string()),
            ("Eggs".to_string(), "AEggs<string, string>".to_string()),
            ("Salad".to_string(), "ASalad<string, string>".to_string()),
            ("Sausage".to_string(), "ASausage<string, string>".to_string())]},
        "ABacon<Cucumber, Wrapper<string>>" => Definition::Struct {fields: Fields::Empty},
        "ABacon<string, string>" => Definition::Struct {fields: Fields::Empty},
        "AEggs<Cucumber, Wrapper<string>>" => Definition::Struct {fields: Fields::Empty},
        "AEggs<string, string>" => Definition::Struct {fields: Fields::Empty},
        "ASalad<Cucumber, Wrapper<string>>" => Definition::Struct {fields: Fields::UnnamedFields(vec!["Tomatoes".to_string(), "Cucumber".to_string(), "Oil<u64, string>".to_string()])},
        "ASalad<string, string>" => Definition::Struct { fields: Fields::UnnamedFields( vec!["Tomatoes".to_string(), "string".to_string(), "Oil<u64, string>".to_string() ])},
        "ASausage<Cucumber, Wrapper<string>>" => Definition::Struct {fields: Fields::NamedFields(vec![("wrapper".to_string(), "Wrapper<string>".to_string()), ("filling".to_string(), "Filling".to_string())])},
        "ASausage<string, string>" => Definition::Struct{ fields: Fields::NamedFields(vec![("wrapper".to_string(), "string".to_string()), ("filling".to_string(), "Filling".to_string())])},
        "Cucumber" => Definition::Struct {fields: Fields::Empty},
        "Filling" => Definition::Struct {fields: Fields::Empty},
        "HashMap<u64, string>" => Definition::Sequence { elements: "Tuple<u64, string>".to_string()},
        "Oil<u64, string>" => Definition::Struct { fields: Fields::NamedFields(vec![("seeds".to_string(), "HashMap<u64, string>".to_string()), ("liquid".to_string(), "Option<u64>".to_string())])},
        "Option<string>" => Definition::Enum {variants: vec![("None".to_string(), "nil".to_string()), ("Some".to_string(), "string".to_string())]},
        "Option<u64>" => Definition::Enum { variants: vec![("None".to_string(), "nil".to_string()), ("Some".to_string(), "u64".to_string())]},
        "Tomatoes" => Definition::Struct {fields: Fields::Empty},
        "Tuple<u64, string>" => Definition::Tuple {elements: vec!["u64".to_string(), "string".to_string()]},
        "Wrapper<string>" => Definition::Struct{ fields: Fields::NamedFields(vec![("foo".to_string(), "Option<string>".to_string()), ("bar".to_string(), "A<string, string>".to_string())])}
        },
        defs
    );
}
