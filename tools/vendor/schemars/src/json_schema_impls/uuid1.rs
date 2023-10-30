use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use uuid1::Uuid;

impl JsonSchema for Uuid {
    no_ref_schema!();

    fn schema_name() -> String {
        "Uuid".to_string()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            format: Some("uuid".to_string()),
            ..Default::default()
        }
        .into()
    }
}
