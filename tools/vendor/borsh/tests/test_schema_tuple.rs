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
fn test_unary_tuple_schema() {
    assert_eq!("Tuple<bool>", <(bool,)>::declaration());
    let mut defs = Default::default();
    <(bool,)>::add_definitions_recursively(&mut defs);
    assert_eq!(
        map! {
        "Tuple<bool>" => Definition::Tuple { elements: vec!["bool".to_string()] }
        },
        defs
    );
}
