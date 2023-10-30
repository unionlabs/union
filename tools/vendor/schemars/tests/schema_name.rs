mod util;
use schemars::JsonSchema;
use util::*;

#[allow(dead_code)]
#[derive(JsonSchema)]
struct MyStruct<T, U, V, W> {
    t: T,
    u: U,
    v: V,
    w: W,
    inner: MySimpleStruct,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
struct MySimpleStruct {
    foo: i32,
}

#[test]
fn default_name_multiple_type_params() -> TestResult {
    test_default_generated_schema::<MyStruct<i32, (), bool, Vec<String>>>("schema-name-default")
}

#[allow(dead_code)]
#[derive(JsonSchema)]
#[serde(rename = "a-new-name-{W}-{T}-{T}")]
#[schemars(rename_all = "camelCase")]
struct MyRenamedStruct<T, U, V, W> {
    t: T,
    u: U,
    v: V,
    w: W,
    inner: MySimpleRenamedStruct,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
#[serde(rename = "this-attribute-is-ignored")]
#[schemars(rename = "another-new-name")]
struct MySimpleRenamedStruct {
    foo: i32,
}

#[test]
fn overriden_with_rename_multiple_type_params() -> TestResult {
    test_default_generated_schema::<MyRenamedStruct<i32, (), bool, Vec<String>>>(
        "schema-name-custom",
    )
}
