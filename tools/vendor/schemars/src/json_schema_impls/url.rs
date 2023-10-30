use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use url::Url;

impl JsonSchema for Url {
    no_ref_schema!();

    fn schema_name() -> String {
        "Url".to_owned()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            format: Some("uri".to_owned()),
            ..Default::default()
        }
        .into()
    }
}
