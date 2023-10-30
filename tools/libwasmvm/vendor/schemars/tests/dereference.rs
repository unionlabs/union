use schemars::{gen::SchemaGenerator, JsonSchema};
use std::ptr;

#[allow(dead_code)]
#[derive(JsonSchema)]
struct Struct {
    foo: i32,
    bar: bool,
}

#[test]
fn dereference_struct() {
    let mut gen = SchemaGenerator::default();
    let struct_ref_schema = gen.subschema_for::<Struct>();
    let struct_schema = gen.definitions().get(&<Struct>::schema_name()).unwrap();

    assert!(struct_ref_schema.is_ref());
    assert!(!struct_schema.is_ref());

    let dereferenced = gen.dereference(&struct_ref_schema);
    assert!(dereferenced.is_some());
    assert!(ptr::eq(dereferenced.unwrap(), struct_schema));
}
