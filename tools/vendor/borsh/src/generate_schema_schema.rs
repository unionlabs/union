//! Generate `BorshSchemaCointainer` for `BorshSchemaContainer` and save it into a file.
// TODO: re-enable this lint when we bump msrv to 1.58
#![allow(clippy::uninlined_format_args)]
use borsh::schema::BorshSchema;
use borsh::BorshSerialize;
use std::fs::File;
use std::io::Write;

fn main() {
    let container = borsh::schema::BorshSchemaContainer::schema_container();
    println!("{:?}", container);
    let data = container
        .try_to_vec()
        .expect("Failed to serialize BorshSchemaContainer");
    let mut file = File::create("schema_schema.dat").expect("Failed to create file");
    file.write_all(&data).expect("Failed to write file");
}
